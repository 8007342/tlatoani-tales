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
//! | 20 | Spec invariant violation (`verify` failed) |
//! | 30 | Infra failure (container unreachable, model missing, bind-mount denied) |
//! | 31 | Infra failure — permission denied on bind mount (sub-code of 30) |
//! | 40 | Usage error (bad args, strip not found) |
//!
// @trace spec:orchestrator
// @Lesson S1-1300

use clap::{Parser, Subcommand};
use tt_events::Bus;

// Keep the dependency graph honest during scaffolding: each `tt-*` crate
// below earns a call site as its implementation fills in, but we need them
// all linked today so `cargo check --workspace` verifies the shape.
#[allow(unused_imports)]
use {
    anyhow as _, futures as _, indicatif as _, tt_calmecac_indexer as _, tt_comfy as _,
    tt_compose as _, tt_core as _, tt_hashing as _, tt_lora as _, tt_metadata as _,
    tt_qa as _, tt_specs as _,
};

/// `tt-render` — render Tlatoāni Tales strips from specs.
#[derive(Debug, Parser)]
#[command(name = "tt-render", version, about, long_about = None)]
struct Cli {
    /// Scope to a single strip (e.g. `--only 03`).
    #[arg(long, global = true)]
    only: Option<String>,

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
    /// Spec-mutation primitive: edit one property, re-render the invalidation set.
    Mutate {
        #[arg(long)]
        prop: String,
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

/// Documented exit codes. Return one of these from `main` through `process::exit`.
#[derive(Debug, Clone, Copy)]
enum ExitCode {
    Success = 0,
    CanonFailure = 10,
    SpecInvariantViolation = 20,
    InfraFailure = 30,
    #[allow(dead_code)]
    InfraPermissionDenied = 31,
    #[allow(dead_code)]
    UsageError = 40,
}

#[tokio::main]
async fn main() {
    let code = run().await;
    std::process::exit(code as i32);
}

/// Dispatch entry point. Returns a documented exit code rather than
/// bubbling `anyhow` errors — the code IS the interface CI reads.
async fn run() -> ExitCode {
    let cli = Cli::parse();

    // Telemetry first — every downstream `tracing::*` call needs it.
    let _guard = match tt_telemetry::init_default() {
        Ok(g) => g,
        Err(_) => return ExitCode::InfraFailure,
    };

    // Shared event bus for the duration of the run.
    let _bus = Bus::new();

    match cli.cmd.unwrap_or(Command::Render) {
        Command::Render => {
            tracing::info!(spec = "orchestrator", "tt-render: render default");
            let _ = (cli.only, cli.force, cli.dry_run, cli.resume_from_cache_only);
            ExitCode::Success
        }
        Command::Mutate { prop: _, to: _ } => {
            tracing::info!(spec = "orchestrator", "tt-render: mutate");
            ExitCode::Success
        }
        Command::Verify => match tt_lint::verify_all_in(std::path::Path::new(".")).await {
            Ok(report) if report.is_clean() => ExitCode::Success,
            Ok(_) => ExitCode::SpecInvariantViolation,
            Err(_) => {
                let _ = ExitCode::CanonFailure; // placeholder reference so the variant stays live
                ExitCode::InfraFailure
            }
        },
        Command::Trace { target: _ } => {
            tracing::info!(spec = "orchestrator", "tt-render: trace");
            ExitCode::Success
        }
        Command::Watch => {
            tracing::info!(spec = "orchestrator", "tt-render: watch");
            ExitCode::Success
        }
    }
}
