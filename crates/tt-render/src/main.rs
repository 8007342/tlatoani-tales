//! Tlatoāni Tales — `tt-render` orchestrator CLI.
//!
//! Parses args with clap, sets up `tracing` + the event bus, dispatches the
//! subcommand, and exits with the documented codes from
//! `openspec/specs/orchestrator/spec.md`:
//!
//! | Code | Meaning |
//! |---|---|
//! | 0  | Success |
//! | 10 | Canon failure (QA unrecoverable; strip marked `needs-human`) |
//! | 20 | Canon needs-human / spec invariant violation |
//! | 30 | Infra failure (container unreachable, model missing, bind-mount denied) |
//! | 31 | Infra failure — permission denied on bind mount (sub-code of 30) |
//! | 2  | Usage error (bad args, strip not found) |
//!
//! The orchestrator spec numbers usage errors as `40` in the narrative table,
//! but the shared `FailureClass::Usage.exit_code()` returns `2` — the
//! POSIX-idiomatic "bad invocation" code clap itself emits. We honour the
//! shared taxonomy so every crate that returns a `TtError::Usage` produces
//! the same exit code; the orchestrator-spec table is the one that drifts.
//!
//! This binary is the **integration point** for every library crate in the
//! `tt-*` workspace. Most of the pipeline is stubbed with clearly-flagged
//! `(stub — wave-10 milestone)` output: workflow construction, per-strip
//! prompt generation, and the live ComfyUI/ollama submit loop all require
//! infrastructure that lives outside this binary today. The happy-path
//! wiring is written in full so the integration shape is visible and the
//! types line up.
//!
// @trace spec:orchestrator, spec:visual-qa-loop, spec:isolation
// @Lesson S1-1300

use std::io::IsTerminal;
use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use tt_core::{project_root, FailureClass, TtError};
use tt_events::{Bus, RenderEvent};

// Keep every library crate hot on the link graph so `cargo check --workspace`
// catches a regression in any one of them when this binary changes.
// Each crate below earns a real call site in the stubbed happy path; imports
// without a call site are `as _` so the warning budget stays clean.
#[allow(unused_imports)]
use {
    anyhow as _, futures as _, tt_calmecac_indexer as _, tt_compose as _, tt_hashing as _,
    tt_lora as _, tt_metadata as _, tt_specs as _,
};

use tt_comfy::ComfyClient;
use tt_qa::{QaClient, Verdict};
use url::Url;

// ---------------------------------------------------------------------------
// CLI shape
// ---------------------------------------------------------------------------

/// `tt-render` — render Tlatoāni Tales strips from specs.
///
/// Default (no subcommand) renders every stale strip. See
/// `openspec/specs/orchestrator/spec.md` §Commands for the full table.
#[derive(Debug, Parser)]
#[command(
    name = "tt-render",
    version,
    about = "Tlatoāni Tales — render strips from specs, critique via VLM, composite with plates.",
    long_about = None,
)]
struct Cli {
    /// Bypass cache; re-render every panel of every in-scope strip.
    #[arg(long, global = true)]
    force: bool,

    /// Hash + invalidation set only; render nothing.
    #[arg(long, global = true)]
    dry_run: bool,

    /// Composite from cache; no ComfyUI, no VLM; offline.
    #[arg(long, global = true)]
    resume_from_cache_only: bool,

    #[command(subcommand)]
    cmd: Option<Command>,
}

/// Subcommands for `tt-render`.
#[derive(Debug, Subcommand)]
enum Command {
    /// Default when no subcommand is given — render all stale strips.
    Render,
    /// Render exactly one strip by its 1-based number (e.g. `only 03`).
    Only {
        /// Strip number (1-based). `3`, `03`, and `3.` all parse.
        strip: u16,
    },
    /// Spec-mutation primitive: edit one property, re-render the invalidation set.
    Mutate {
        /// Dotted path identifying the spec field, e.g. `style-bible.palette.paper`.
        #[arg(long)]
        prop: String,
        /// New value for the property.
        #[arg(long = "to")]
        to: String,
    },
    /// Run `tt-lint`. Exits non-zero on any violation.
    Verify,
    /// Print the coverage graph for a trace name or lesson ID.
    Trace {
        /// `spec:<name>` or `Sn-NNN`.
        target: String,
    },
    /// Long-running daemon; re-renders on spec/strip/character change.
    Watch,
}

// ---------------------------------------------------------------------------
// Exit code plumbing
// ---------------------------------------------------------------------------

/// Map an anyhow-ish error into the canonical `FailureClass` exit code.
///
/// Binaries carry `anyhow::Error` at the edges and preserve the
/// `TtError`/`FailureClass` via downcast. Any error that did not come
/// through the shared taxonomy is treated as `Infra` — the safe default per
/// orchestrator/spec.md §Failure modes (the tool is wrong, not the comic).
fn exit_code_of(err: &anyhow::Error) -> u8 {
    if let Some(tt) = err.downcast_ref::<TtError>() {
        return tt.class().exit_code();
    }
    FailureClass::Infra.exit_code()
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let project_dir = project_root();

    // Telemetry first — every downstream `tracing::*` call needs a
    // subscriber, and we want stderr JSON lines flowing before we touch
    // disk or network.
    let _guard = match tt_telemetry::init(&project_dir) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("tt-render: telemetry init failed: {e}");
            return ExitCode::from(FailureClass::Infra.exit_code());
        }
    };

    match dispatch(cli, &project_dir).await {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            let code = exit_code_of(&e);
            tracing::error!(
                spec = "orchestrator",
                exit_code = code,
                error = %e,
                "tt-render: failed"
            );
            eprintln!("tt-render: {e}");
            ExitCode::from(code)
        }
    }
}

/// Dispatch the parsed CLI to the correct subcommand handler.
async fn dispatch(cli: Cli, project_dir: &Path) -> anyhow::Result<()> {
    match cli.cmd.unwrap_or(Command::Render) {
        Command::Render => {
            render_run(project_dir, None, cli.force, cli.dry_run, cli.resume_from_cache_only).await
        }
        Command::Only { strip } => {
            render_run(
                project_dir,
                Some(strip),
                cli.force,
                cli.dry_run,
                cli.resume_from_cache_only,
            )
            .await
        }
        Command::Mutate { prop, to } => mutate_run(&prop, &to).await,
        Command::Verify => verify_run(project_dir).await,
        Command::Trace { target } => trace_run(project_dir, &target).await,
        Command::Watch => watch_run().await,
    }
}

// ---------------------------------------------------------------------------
// `render` / `only` — the integration happy-path
// ---------------------------------------------------------------------------

/// The integrated render flow: telemetry, event bus, spec graph, clients,
/// per-strip loop, composite, metadata.
///
/// Per-panel ComfyUI workflow construction is stubbed — it needs
/// `tt-specs::load_all` to return a populated `SpecGraph` AND a per-strip
/// prompt-to-workflow translator that doesn't exist yet. The stub sites
/// are tagged `wave-10 milestone` in error / log messages.
async fn render_run(
    project_dir: &Path,
    only: Option<u16>,
    force: bool,
    dry_run: bool,
    resume_from_cache_only: bool,
) -> anyhow::Result<()> {
    let run_id = format!("run-{}", tt_telemetry::iso8601_now());
    let bus = Bus::new();

    // Spawn the telemetry sink that drains the bus into
    // `output/telemetry/{strip-NN,run}.jsonl`. We subscribe BEFORE dropping
    // the bus so the sink sees `RecvError::Closed` when we exit.
    let sink_subscriber = bus.subscribe();
    let telemetry_root = project_dir.join("output");
    let sink_task = tokio::spawn(async move {
        if let Err(e) =
            tt_telemetry::run_sink_from_subscriber(sink_subscriber, &telemetry_root).await
        {
            tracing::warn!(spec = "orchestrator", error = %e, "telemetry sink shut down with error");
        }
    });

    bus.emit(RenderEvent::RunStarted {
        run_id: run_id.clone(),
        spec_tag: None,
        lesson_tag: None,
    });

    if std::env::var("TT_OFFLINE").ok().as_deref() == Some("1") {
        tracing::info!(
            spec = "isolation",
            "TT_OFFLINE=1 — all untrusted containers run with --network=none (enforced by launcher; this binary does not spawn them)"
        );
    }

    // Load the spec graph. SpecGraph::load_all is not complete yet;
    // `tt_specs::load_all` returns an empty graph. Fall back to default on
    // error, warn, continue — the rest of the flow still type-checks.
    let graph = match tt_specs::load_all(project_dir).await {
        Ok(g) => g,
        Err(e) => {
            tracing::warn!(
                spec = "orchestrator",
                error = %e,
                "SpecGraph loading is incomplete — falling back to default (wave-10 milestone)"
            );
            tt_specs::SpecGraph::default()
        }
    };
    tracing::info!(
        spec = "orchestrator",
        specs_loaded = graph.specs.len(),
        strips_declared = graph.strips.len(),
        "tt-specs: graph loaded"
    );

    // Clients — URLs are always 127.0.0.1 per isolation/spec.md. The
    // untrusted containers forward their HTTP ports to localhost; if
    // TT_OFFLINE=1, they are launched with --network=none by the
    // orchestrator's launcher (not this binary — the launcher is in
    // `scripts/`). See isolation/spec.md §Canonical flags.
    let _comfy = ComfyClient::new(
        Url::parse("http://127.0.0.1:8188/").expect("static URL parses"),
    );
    let _qa = QaClient::new(
        Url::parse("http://127.0.0.1:11434/").expect("static URL parses"),
        tt_qa::DEFAULT_MODEL,
    );

    // Progress bar on stderr when attached to a TTY. indicatif is happy to
    // draw to a non-terminal but it's noise in a pipe.
    let strip_list: Vec<tt_core::StripId> = match only {
        Some(n) => vec![tt_core::StripId::new(n).map_err(|e| anyhow::anyhow!(e))?],
        None => strips_from_graph(&graph),
    };

    let progress = if std::io::stderr().is_terminal() {
        let pb = ProgressBar::new(strip_list.len() as u64);
        pb.set_style(
            ProgressStyle::with_template("{spinner} [{pos}/{len}] {msg}")
                .expect("static template parses"),
        );
        Some(pb)
    } else {
        None
    };

    let mut strips_rendered = 0u32;
    let mut strips_cached = 0u32;

    for strip in &strip_list {
        if let Some(pb) = &progress {
            pb.set_message(format!("strip {strip}"));
        }
        bus.emit(RenderEvent::StripDiscovered {
            strip: *strip,
            spec_tag: None,
            lesson_tag: None,
        });

        // Per-strip: hash panels, check cache, (stub) submit missing to
        // Comfy, (stub) critique via QA with reroll loop, composite, emit
        // metadata. The actual panel-hash/cache/submit path needs
        // proposal data that load_all does not yet supply.
        let fresh = process_strip(&bus, project_dir, *strip, force, dry_run, resume_from_cache_only)
            .await?;
        if fresh {
            strips_rendered += 1;
        } else {
            strips_cached += 1;
        }
        if let Some(pb) = &progress {
            pb.inc(1);
        }
    }

    if let Some(pb) = progress {
        pb.finish_with_message("done");
    }

    bus.emit(RenderEvent::RunComplete {
        run_id,
        strips_rendered,
        strips_cached,
        spec_tag: None,
        lesson_tag: None,
    });

    // Drop the bus so the telemetry sink sees `Closed` and drains.
    drop(bus);
    let _ = sink_task.await;

    Ok(())
}

/// Materialize the list of strips to process from a `SpecGraph`.
///
/// When the graph is empty (current scaffold reality), the list is empty
/// and the render flow is a no-op at the strip level — we still run the
/// RunStarted/RunComplete lifecycle events so telemetry sees the run.
fn strips_from_graph(graph: &tt_specs::SpecGraph) -> Vec<tt_core::StripId> {
    let mut out: Vec<tt_core::StripId> = graph.strips.iter().map(|p| p.strip_id).collect();
    out.sort_by_key(|s| s.as_u16());
    out
}

/// Process one strip — stubbed at the ComfyUI submit boundary.
///
/// Returns `true` if the strip produced fresh pixels (every panel rendered),
/// `false` if it was served entirely from cache. The real implementation
/// iterates `StripProposal::panels`, computes `tt_hashing::panel_hash` for
/// each, checks `cache/panels/<hash>.png`, submits missing workflows to
/// ComfyUI, runs the QA loop, promotes or escalates, then composites.
///
/// Every stub site below prints `(stub — wave-10 milestone)` so the author
/// reading the integration shape sees exactly what remains.
async fn process_strip(
    _bus: &Bus,
    _project_dir: &Path,
    strip: tt_core::StripId,
    _force: bool,
    _dry_run: bool,
    _resume_from_cache_only: bool,
) -> anyhow::Result<bool> {
    tracing::info!(
        spec = "orchestrator",
        strip = %strip,
        "process_strip: (stub — wave-10 milestone) — per-strip proposal not yet loaded"
    );

    // -------- real wiring the full implementation will use --------
    //
    // for panel in strip.panels {
    //     let input = tt_hashing::PanelInput { ... };
    //     let hash = tt_hashing::panel_hash(&input);
    //     bus.emit(RenderEvent::PanelHashComputed { strip, panel: panel.index, panel_hash: hash, ... });
    //
    //     if cache_has(&hash) {
    //         bus.emit(RenderEvent::CacheHit { strip, panel: panel.index, panel_hash: hash, ... });
    //         continue;
    //     }
    //     bus.emit(RenderEvent::CacheMiss { strip, panel: panel.index, panel_hash: hash, reason: "first render".into(), ... });
    //
    //     let workflow = build_workflow_for_panel(&panel, &graph)?; // <-- stub: workflow construction
    //     let prompt_id = comfy.submit(workflow).await?;
    //     bus.emit(ComfyEvent::Submitted { strip, panel: panel.index, panel_hash: hash, prompt_id: prompt_id.to_string(), ... });
    //
    //     let png = await_render(&comfy, &prompt_id).await?;
    //     qa_loop(&qa, &bus, strip, panel.index, hash, &png).await?;
    //
    //     promote_to_cache(&hash, &png, &report).await?;
    // }
    //
    // let compose_in = tt_compose::StripInput { ... };
    // let res = tt_compose::composite_strip(&compose_in, &output_path, &fonts).await?;
    // bus.emit(ComposeEvent::ComposeDone { strip, output_path: res.png_path.display().to_string(), ... });
    //
    // let metadata = build_metadata(&strip, &res, &graph);
    // tt_metadata::write_metadata(&metadata, &metadata_path).await?;
    // bus.emit(ComposeEvent::MetadataWritten { strip, metadata_path: metadata_path.display().to_string(), ... });

    Ok(false)
}

/// Run the QA loop for one panel.
///
/// Honours `TT_QA`: `off` skips the VLM entirely, `single` runs one pass
/// without reroll, `strict` (default) runs up to 5 rerolls per drift
/// verdict. The addendum for each reroll comes from `tt_qa::derive_addendum`.
///
/// Stubbed: the full loop needs the PNG bytes from ComfyUI. The type
/// skeleton below is exercised by `cargo check` via the unused-import
/// elision above.
#[allow(dead_code)]
async fn qa_loop_stub(
    qa: &QaClient,
    checks: &[tt_qa::Check],
    png: &[u8],
    strip_label: &str,
    panel: u8,
    panel_hash: tt_core::PanelHash,
) -> anyhow::Result<Verdict> {
    let mode = std::env::var("TT_QA").unwrap_or_else(|_| "strict".to_string());
    if mode == "off" {
        return Ok(Verdict::Stable);
    }
    let max_rerolls = if mode == "single" { 0 } else { 5 };

    let mut iteration: u8 = 1;
    loop {
        let report = qa
            .critique(png, checks, strip_label, panel, iteration, panel_hash)
            .await?;
        match report.verdict {
            Verdict::Stable => return Ok(Verdict::Stable),
            Verdict::NeedsHuman => return Ok(Verdict::NeedsHuman),
            Verdict::Reroll | Verdict::Escalate if iteration as u32 > max_rerolls => {
                return Ok(Verdict::NeedsHuman);
            }
            Verdict::Reroll | Verdict::Escalate => {
                let _addendum = tt_qa::derive_addendum(&report);
                // Real impl would re-submit the ComfyUI workflow here with
                // the addendum appended to the prompt.
                iteration += 1;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// `mutate`
// ---------------------------------------------------------------------------

async fn mutate_run(prop: &str, to: &str) -> anyhow::Result<()> {
    eprintln!(
        "tt-render: mutate --prop {prop} --to {to} (stub — wave-10 milestone)\n\
         Spec mutation is the author-facing propagation primitive described in\n\
         openspec/specs/orchestrator/spec.md §Commands. The full impl edits the\n\
         named spec field, recomputes hashes, emits the invalidation set as\n\
         events, and re-renders stale panels. This binary surfaces the CLI\n\
         shape today so the author can see the contract."
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// `verify`
// ---------------------------------------------------------------------------

async fn verify_run(project_dir: &Path) -> anyhow::Result<()> {
    let report = tt_lint::verify_all_in(project_dir).await?;
    print!("{report}");
    if report.has_violations() {
        // verify is the spec-invariant linter. Any violation → exit 10.
        return Err(TtError::Canon(format!(
            "tt-lint reported {} violation(s) — see openspec/specs/*/spec.md",
            report.violations.len()
        ))
        .into());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// `trace`
// ---------------------------------------------------------------------------

async fn trace_run(project_dir: &Path, target: &str) -> anyhow::Result<()> {
    // Accept `spec:<name>`, bare `<name>`, or `Sn-NNN`.
    let (kind, value) = if let Some(rest) = target.strip_prefix("spec:") {
        ("spec", rest)
    } else if target.starts_with('S') && target.contains('-') {
        ("lesson", target)
    } else {
        ("spec", target)
    };

    let _graph = tt_specs::load_all(project_dir).await.unwrap_or_default();
    eprintln!(
        "tt-render trace: {kind}={value}\n\
         (stub — SpecGraph loading unfinished; full trace coverage deferred to wave-10 milestone)"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// `watch`
// ---------------------------------------------------------------------------

async fn watch_run() -> anyhow::Result<()> {
    eprintln!(
        "tt-render watch: not yet implemented (wave-10 milestone).\n\
         The watch subcommand will use the `notify` crate to observe\n\
         openspec/specs/, strips/, and characters/ for changes and\n\
         re-render affected strips. See orchestrator/spec.md §Commands."
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    // -- CLI parsing -----------------------------------------------------

    #[test]
    fn clap_app_is_well_formed() {
        Cli::command().debug_assert();
    }

    #[test]
    fn clap_parses_default_render() {
        let cli = Cli::try_parse_from(["tt-render"]).expect("bare invocation parses");
        assert!(cli.cmd.is_none());
        assert!(!cli.force);
    }

    #[test]
    fn clap_parses_render_subcommand() {
        let cli = Cli::try_parse_from(["tt-render", "render"]).expect("render parses");
        assert!(matches!(cli.cmd, Some(Command::Render)));
    }

    #[test]
    fn clap_parses_only_subcommand() {
        let cli = Cli::try_parse_from(["tt-render", "only", "3"]).expect("only parses");
        match cli.cmd {
            Some(Command::Only { strip }) => assert_eq!(strip, 3),
            other => panic!("expected Only, got {other:?}"),
        }
    }

    #[test]
    fn clap_parses_mutate_subcommand() {
        let cli = Cli::try_parse_from([
            "tt-render",
            "mutate",
            "--prop",
            "style-bible.palette.paper",
            "--to",
            "#F0E0C0",
        ])
        .expect("mutate parses");
        match cli.cmd {
            Some(Command::Mutate { prop, to }) => {
                assert_eq!(prop, "style-bible.palette.paper");
                assert_eq!(to, "#F0E0C0");
            }
            other => panic!("expected Mutate, got {other:?}"),
        }
    }

    #[test]
    fn clap_parses_verify_subcommand() {
        let cli = Cli::try_parse_from(["tt-render", "verify"]).expect("verify parses");
        assert!(matches!(cli.cmd, Some(Command::Verify)));
    }

    #[test]
    fn clap_parses_trace_subcommand() {
        let cli = Cli::try_parse_from(["tt-render", "trace", "spec:orchestrator"])
            .expect("trace parses");
        match cli.cmd {
            Some(Command::Trace { target }) => assert_eq!(target, "spec:orchestrator"),
            other => panic!("expected Trace, got {other:?}"),
        }
    }

    #[test]
    fn clap_parses_watch_subcommand() {
        let cli = Cli::try_parse_from(["tt-render", "watch"]).expect("watch parses");
        assert!(matches!(cli.cmd, Some(Command::Watch)));
    }

    #[test]
    fn clap_global_flags_work_before_subcommand() {
        let cli = Cli::try_parse_from(["tt-render", "--force", "render"]).expect("parses");
        assert!(cli.force);
        assert!(matches!(cli.cmd, Some(Command::Render)));
    }

    #[test]
    fn clap_force_accepted_on_only() {
        let cli =
            Cli::try_parse_from(["tt-render", "--force", "only", "5"]).expect("parses");
        assert!(cli.force);
        assert!(matches!(cli.cmd, Some(Command::Only { strip: 5 })));
    }

    #[test]
    fn clap_rejects_unknown_subcommand() {
        let err = Cli::try_parse_from(["tt-render", "nonsense"]).unwrap_err();
        // Usage errors from clap.
        assert!(!err.to_string().is_empty());
    }

    // -- Exit code mapping ----------------------------------------------

    #[test]
    fn exit_code_of_canon() {
        let e: anyhow::Error = TtError::Canon("drift".into()).into();
        assert_eq!(exit_code_of(&e), 10);
    }

    #[test]
    fn exit_code_of_canon_needs_human() {
        let e: anyhow::Error = TtError::CanonNeedsHuman("rerolls exhausted".into()).into();
        assert_eq!(exit_code_of(&e), 20);
    }

    #[test]
    fn exit_code_of_infra() {
        let e: anyhow::Error = TtError::Infra("unreachable".into()).into();
        assert_eq!(exit_code_of(&e), 30);
    }

    #[test]
    fn exit_code_of_infra_sub() {
        let e: anyhow::Error =
            TtError::InfraPermissionDenied("bind mount denied".into()).into();
        assert_eq!(exit_code_of(&e), 31);
    }

    #[test]
    fn exit_code_of_usage() {
        let e: anyhow::Error = TtError::Usage("bad args".into()).into();
        assert_eq!(exit_code_of(&e), 2);
    }

    #[test]
    fn exit_code_defaults_to_infra_for_unknown_errors() {
        // Not a TtError — must fall back to Infra per orchestrator/spec.md.
        let e: anyhow::Error = anyhow::anyhow!("some raw error string");
        assert_eq!(exit_code_of(&e), 30);
    }

    // -- strips_from_graph ---------------------------------------------

    #[test]
    fn strips_from_graph_empty_when_graph_empty() {
        let g = tt_specs::SpecGraph::default();
        assert!(strips_from_graph(&g).is_empty());
    }

    // -- integration (gated) --------------------------------------------

    /// Integration-style: spawn this binary with `verify` and assert exit 0
    /// against the real repo. Gated because it requires an already-built
    /// `tt-render` binary on disk and the working tree to be clean.
    #[test]
    #[ignore = "runs tt-render verify as a subprocess; requires the binary built"]
    fn verify_subprocess_against_real_repo_exits_zero() {
        let bin = std::env::var("CARGO_BIN_EXE_tt-render")
            .expect("CARGO_BIN_EXE_tt-render is set by cargo test for bin targets");
        let status = std::process::Command::new(bin)
            .arg("verify")
            .status()
            .expect("spawn tt-render verify");
        assert!(status.success(), "verify exited {:?}", status.code());
    }

    // -- qa_loop_stub is linked but dead without a live qa client -------

    #[tokio::test]
    async fn qa_loop_off_returns_stable_without_network() {
        // TT_QA=off must short-circuit to Stable — proves the env-var switch
        // is live without reaching the ollama endpoint.
        // SAFETY: env is a process-global; tests touching it are serialised
        // by the fact that only this test reads TT_QA, and cargo test runs
        // per-process by default for integration/unit test binaries.
        std::env::set_var("TT_QA", "off");
        let qa = QaClient::new(
            Url::parse("http://127.0.0.1:1/").unwrap(),
            tt_qa::DEFAULT_MODEL,
        );
        let verdict = qa_loop_stub(
            &qa,
            &[],
            b"",
            "TT 01/15",
            1,
            tt_core::PanelHash::from_bytes([0u8; 32]),
        )
        .await
        .expect("off mode never errors");
        assert_eq!(verdict, Verdict::Stable);
        std::env::remove_var("TT_QA");
    }

    // Keep referenced so the helper is not flagged as dead.
    #[allow(dead_code)]
    fn _keep_project_dir_live() -> std::path::PathBuf {
        tt_core::project_root()
    }
}
