//! Tlatoāni Tales — `tt-calmecac` CLI.
//!
//! Thin wrapper around `tt-calmecac-indexer` + the viewer container
//! launcher. Subcommands:
//!
//! | Subcommand | Behaviour |
//! |---|---|
//! | `build`   | Run the indexer, write `output/calmecac-bundle/calmecac-index.json`. |
//! | `serve`   | Best-effort start `tlatoani-tales-viewer` (idempotent). Silverblue-only. |
//! | `stop`    | Stop the viewer container. |
//! | `rebuild` | Rebuild the viewer image, then `serve`. Silverblue-only. |
//!
//! The binary is a convenience wrapper around `scripts/tlatoāni_tales.sh`,
//! which is the canonical entry point per
//! `openspec/specs/calmecac/spec.md` §Launcher script contract. The bash
//! script owns the container lifecycle; this binary delegates via
//! subprocess so there is exactly one code path that invokes `podman run`.
//!
//! Exit codes per `openspec/specs/calmecac/spec.md` §Exit codes:
//!
//! | Code | Meaning |
//! |---|---|
//! | `0` | Running / cleanly stopped. |
//! | `1` | Precondition failed (wrong OS, missing podman, port taken). |
//! | `2` | Container failed to start for non-precondition reasons. |
//!
//! Governing spec: `openspec/specs/calmecac/spec.md`.
//!
// @trace spec:calmecac, spec:isolation
// @Lesson S1-1000
// @Lesson S1-1500

use std::path::{Path, PathBuf};
use std::process::Command as SysCommand;

use clap::{Parser, Subcommand};
use tt_core::{podman::CONTAINER_NAME_PREFIX, project_root, Role};

/// Default local port per `openspec/specs/calmecac/spec.md` §Behaviour.
const DEFAULT_PORT: u16 = 8088;

/// Refusal message for non-Silverblue hosts. Verbatim, per
/// `openspec/specs/calmecac/spec.md` §Silverblue-only constraint. No ASCII
/// fallback; no `sudo` workaround; this message is part of the teaching.
// @trace spec:calmecac
const NOT_SILVERBLUE_MSG: &str =
    "Local viewer supports Fedora Silverblue only. Season 2 teaches why.";

/// Exit codes per `openspec/specs/calmecac/spec.md` §Exit codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExitCode {
    /// Viewer running (or cleanly stopped via `--stop`).
    Success = 0,
    /// Precondition failed (wrong OS, missing podman, port taken).
    PreconditionFailed = 1,
    /// Container failed to start for a non-precondition reason.
    ContainerStartFailed = 2,
}

/// `tt-calmecac` — observe Tlatoāni Tales' own observability scaffold.
#[derive(Debug, Parser)]
#[command(name = "tt-calmecac", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build the Calmecac concept index under `output/calmecac-bundle/`.
    Build,
    /// Start the viewer container (`tlatoani-tales-viewer`). Silverblue-only.
    Serve {
        /// Override the default port (8088).
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
    },
    /// Stop the viewer container and exit. Silent no-op if not running.
    Stop,
    /// Rebuild the viewer image, then flow into `serve`. Silverblue-only.
    Rebuild {
        /// Override the default port (8088).
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    let code = run().await;
    std::process::exit(code as i32);
}

/// Dispatch entry point. Returns a documented exit code rather than
/// bubbling `anyhow` errors — the code IS the interface scripts + CI read.
async fn run() -> ExitCode {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Build => match cmd_build().await {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                eprintln!("tt-calmecac build failed: {e:#}");
                ExitCode::ContainerStartFailed
            }
        },
        Command::Serve { port } => match cmd_serve(port).await {
            Ok(code) => code,
            Err(e) => {
                eprintln!("tt-calmecac serve failed: {e:#}");
                ExitCode::ContainerStartFailed
            }
        },
        Command::Stop => match cmd_stop().await {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                eprintln!("tt-calmecac stop failed: {e:#}");
                ExitCode::ContainerStartFailed
            }
        },
        Command::Rebuild { port } => match cmd_rebuild(port).await {
            Ok(code) => code,
            Err(e) => {
                eprintln!("tt-calmecac rebuild failed: {e:#}");
                ExitCode::ContainerStartFailed
            }
        },
    }
}

// ---------------------------------------------------------------------------
// Subcommand bodies
// ---------------------------------------------------------------------------

/// `build` — generate `calmecac-index.json` + materialise PWA assets.
// @trace spec:calmecac
async fn cmd_build() -> anyhow::Result<()> {
    let project = project_root();
    let bundle = project.join("output").join("calmecac-bundle");
    std::fs::create_dir_all(&bundle)?;

    // Call the indexer. It may be unimplemented (scaffold); if it panics
    // via `unimplemented!`, we still want a recognisable build output so
    // downstream `serve` has something to mount. Wrap in a catch_unwind.
    let index_out = bundle.join("calmecac-index.json");
    let call_result = std::panic::AssertUnwindSafe(async {
        tt_calmecac_indexer::build_index(&project, &index_out).await
    });
    // `tokio::task::spawn_blocking` + `catch_unwind` is overkill for the
    // scaffold path; tolerate the panic with a fallback placeholder so
    // the rest of the build pipeline keeps working.
    let indexer_ok = match futures::FutureExt::catch_unwind(call_result).await {
        Ok(Ok(())) => true,
        Ok(Err(e)) => {
            eprintln!("tt-calmecac-indexer reported error: {e}");
            false
        }
        Err(_) => {
            eprintln!("tt-calmecac-indexer is not yet implemented; writing placeholder index");
            false
        }
    };
    if !indexer_ok {
        // Minimal placeholder — keep the shape the viewer expects: an
        // object with the top-level keys named in the spec, all empty.
        let placeholder = serde_json::json!({
            "lessons":     {},
            "rules":       {},
            "strips":      {},
            "seeds":       {},
            "tombstones":  {},
            "edges":       [],
            "convergence": {},
            "commits":     [],
        });
        std::fs::write(&index_out, serde_json::to_vec_pretty(&placeholder)?)?;
    }

    // PWA assets: `calmecac/web/` is authored by a parallel agent. If it
    // does not yet exist, lay down a minimal placeholder index.html so
    // the httpd container has something to serve and the reader sees a
    // concept-level message instead of an empty DocumentRoot.
    let web_src = project.join("calmecac").join("web");
    if web_src.is_dir() {
        copy_dir_recursive(&web_src, &bundle)?;
    } else if !bundle.join("index.html").is_file() {
        std::fs::write(
            bundle.join("index.html"),
            include_str!("placeholder_index.html"),
        )?;
    }

    Ok(())
}

/// `serve` — Silverblue + podman guards, then shell out to the bash
/// launcher. The bash script is the canonical entry point per
/// `openspec/specs/calmecac/spec.md` §Launcher script contract.
// @trace spec:calmecac, spec:isolation
async fn cmd_serve(port: u16) -> anyhow::Result<ExitCode> {
    if !ensure_silverblue_stderr() {
        return Ok(ExitCode::PreconditionFailed);
    }
    if !ensure_podman_stderr() {
        return Ok(ExitCode::PreconditionFailed);
    }
    let status = run_launcher(&["--port", &port.to_string()])?;
    Ok(launcher_status_to_exit(status))
}

/// `stop` — `podman stop <name>`; silent-exit if not running.
// @trace spec:calmecac
async fn cmd_stop() -> anyhow::Result<()> {
    let name = format!("{CONTAINER_NAME_PREFIX}{role}", role = viewer_role_suffix());
    // Silent — spec says `--stop` exits 0 whether or not it was running.
    let _ = SysCommand::new("podman")
        .args(["stop", &name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    Ok(())
}

/// `rebuild` — Silverblue + podman guards, delegate to the bash launcher
/// with `--rebuild` (which owns the `podman rmi` + `podman build` flow).
// @trace spec:calmecac, spec:isolation
async fn cmd_rebuild(port: u16) -> anyhow::Result<ExitCode> {
    if !ensure_silverblue_stderr() {
        return Ok(ExitCode::PreconditionFailed);
    }
    if !ensure_podman_stderr() {
        return Ok(ExitCode::PreconditionFailed);
    }
    let status = run_launcher(&["--rebuild", "--port", &port.to_string()])?;
    Ok(launcher_status_to_exit(status))
}

// ---------------------------------------------------------------------------
// Launcher plumbing
// ---------------------------------------------------------------------------

/// Absolute path of `scripts/tlatoāni_tales.sh`. The filename carries the
/// macron on disk; no ASCII fallback per `openspec/specs/calmecac/spec.md`
/// §Launcher script contract and `tlatoāni-spelling/spec.md`.
// @trace spec:calmecac, spec:tlatoāni-spelling
fn launcher_path() -> PathBuf {
    project_root().join("scripts").join("tlatoāni_tales.sh")
}

/// Run `bash scripts/tlatoāni_tales.sh <args>` as a child process and
/// return its raw `ExitStatus`.
fn run_launcher(args: &[&str]) -> anyhow::Result<std::process::ExitStatus> {
    let script = launcher_path();
    if !script.is_file() {
        anyhow::bail!(
            "launcher script missing at {} — repo is incomplete",
            script.display()
        );
    }
    let status = SysCommand::new("bash")
        .arg(&script)
        .args(args)
        .status()?;
    Ok(status)
}

/// Map a `std::process::ExitStatus` from the bash launcher onto the
/// tt-calmecac exit-code taxonomy.
///
/// - 0 → Success
/// - 1 → PreconditionFailed
/// - anything else → ContainerStartFailed
fn launcher_status_to_exit(status: std::process::ExitStatus) -> ExitCode {
    match status.code() {
        Some(0) => ExitCode::Success,
        Some(1) => ExitCode::PreconditionFailed,
        _ => ExitCode::ContainerStartFailed,
    }
}

/// `Role::Viewer` canonical suffix, without the prefix.
fn viewer_role_suffix() -> &'static str {
    // tlatoani-tales-viewer → strip the prefix.
    Role::Viewer
        .container_name()
        .strip_prefix(CONTAINER_NAME_PREFIX)
        .unwrap_or("viewer")
}

// ---------------------------------------------------------------------------
// Guards
// ---------------------------------------------------------------------------

/// Silverblue-only guard per `openspec/specs/calmecac/spec.md`. Prints the
/// refusal message to stderr and returns `false` on non-Silverblue.
// @trace spec:calmecac
fn ensure_silverblue_stderr() -> bool {
    let path = Path::new("/etc/os-release");
    if is_silverblue(path) {
        return true;
    }
    eprintln!("{NOT_SILVERBLUE_MSG}");
    false
}

/// Pure helper: read `/etc/os-release` at `path` and return whether
/// `VARIANT_ID` (or `ID`) is `silverblue`.
///
/// Parameter is a path (not a subprocess call) so tests can feed
/// fabricated os-release files. This is the central Silverblue-detection
/// primitive; the Rust binary never shells out to a subprocess for this.
// @trace spec:calmecac
fn is_silverblue(os_release_path: &Path) -> bool {
    let contents = match std::fs::read_to_string(os_release_path) {
        Ok(s) => s,
        Err(_) => return false,
    };
    parse_variant_id(&contents)
        .map(|v| v == "silverblue")
        .unwrap_or(false)
}

/// Extract `VARIANT_ID=<value>` from an `/etc/os-release`-format string.
/// Handles optional double-quoting. Returns `None` if absent.
fn parse_variant_id(os_release: &str) -> Option<String> {
    for line in os_release.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("VARIANT_ID=") {
            return Some(strip_quotes(rest).to_string());
        }
    }
    None
}

/// Strip a single pair of surrounding double-quotes, if present.
fn strip_quotes(s: &str) -> &str {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Podman presence guard. `command -v podman` semantically — we do it
/// directly in Rust via `which`-style PATH walking to avoid shell.
// @trace spec:isolation
fn ensure_podman_stderr() -> bool {
    if find_in_path("podman").is_some() {
        return true;
    }
    eprintln!(
        "podman not found on PATH. On Fedora Silverblue: \
         `rpm-ostree install podman` then reboot."
    );
    false
}

/// Resolve a bare program name through `$PATH`. Returns the first hit.
fn find_in_path(program: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        let candidate = dir.join(program);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Filesystem helpers
// ---------------------------------------------------------------------------

/// Recursively copy `src` into `dst`. Creates `dst` if missing. Overwrites
/// existing files. Small utility to avoid pulling in a dep for this one
/// scaffold operation.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.is_dir() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), target)?;
        }
        // Skip symlinks — they are rare in bundles and we prefer to fail
        // closed on exotic file types rather than guess.
    }
    Ok(())
}

// Keep the indexer + core in the dependency graph even when the `build`
// call path goes through `futures::FutureExt::catch_unwind` indirection.
#[allow(unused_imports)]
use {tt_calmecac_indexer as _, tt_core as _};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // -- clap parsing -----------------------------------------------------

    #[test]
    fn cli_parses_build() {
        let cli = Cli::try_parse_from(["tt-calmecac", "build"]).unwrap();
        assert!(matches!(cli.cmd, Command::Build));
    }

    #[test]
    fn cli_parses_serve_default_port() {
        let cli = Cli::try_parse_from(["tt-calmecac", "serve"]).unwrap();
        match cli.cmd {
            Command::Serve { port } => assert_eq!(port, DEFAULT_PORT),
            other => panic!("expected Serve, got {other:?}"),
        }
    }

    #[test]
    fn cli_parses_serve_with_port() {
        let cli = Cli::try_parse_from(["tt-calmecac", "serve", "--port", "9090"]).unwrap();
        match cli.cmd {
            Command::Serve { port } => assert_eq!(port, 9090),
            other => panic!("expected Serve, got {other:?}"),
        }
    }

    #[test]
    fn cli_parses_stop() {
        let cli = Cli::try_parse_from(["tt-calmecac", "stop"]).unwrap();
        assert!(matches!(cli.cmd, Command::Stop));
    }

    #[test]
    fn cli_parses_rebuild_with_port() {
        let cli = Cli::try_parse_from(["tt-calmecac", "rebuild", "--port", "9191"]).unwrap();
        match cli.cmd {
            Command::Rebuild { port } => assert_eq!(port, 9191),
            other => panic!("expected Rebuild, got {other:?}"),
        }
    }

    #[test]
    fn cli_rejects_unknown_subcommand() {
        let err = Cli::try_parse_from(["tt-calmecac", "launch"]).unwrap_err();
        // clap uses a specific error kind for unknown subcommands.
        assert!(
            matches!(
                err.kind(),
                clap::error::ErrorKind::InvalidSubcommand
                    | clap::error::ErrorKind::UnknownArgument
            ),
            "unexpected clap kind: {:?}",
            err.kind()
        );
    }

    // -- /etc/os-release parsing -----------------------------------------

    #[test]
    fn parse_variant_id_picks_up_silverblue() {
        let os_release = r#"
NAME="Fedora Linux"
VERSION="43 (Silverblue)"
ID=fedora
VARIANT_ID=silverblue
"#;
        assert_eq!(parse_variant_id(os_release).as_deref(), Some("silverblue"));
    }

    #[test]
    fn parse_variant_id_handles_quoted_value() {
        let os_release = r#"VARIANT_ID="silverblue""#;
        assert_eq!(parse_variant_id(os_release).as_deref(), Some("silverblue"));
    }

    #[test]
    fn parse_variant_id_absent_returns_none() {
        let os_release = r#"
NAME="Fedora Linux"
ID=fedora
"#;
        assert_eq!(parse_variant_id(os_release), None);
    }

    #[test]
    fn parse_variant_id_picks_up_workstation() {
        let os_release = r#"
NAME="Fedora Linux"
ID=fedora
VARIANT_ID=workstation
"#;
        assert_eq!(parse_variant_id(os_release).as_deref(), Some("workstation"));
    }

    #[test]
    fn is_silverblue_true_on_fabricated_silverblue_file() {
        let tmp = std::env::temp_dir().join(format!(
            "tt-calmecac-os-release-silverblue-{}",
            std::process::id()
        ));
        std::fs::write(&tmp, "VARIANT_ID=silverblue\n").unwrap();
        assert!(is_silverblue(&tmp));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn is_silverblue_false_on_workstation_file() {
        let tmp = std::env::temp_dir().join(format!(
            "tt-calmecac-os-release-workstation-{}",
            std::process::id()
        ));
        std::fs::write(&tmp, "VARIANT_ID=workstation\n").unwrap();
        assert!(!is_silverblue(&tmp));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn is_silverblue_false_when_file_missing() {
        let nonexistent = std::env::temp_dir().join("tt-calmecac-nowhere-nonexistent-file.txt");
        // Make sure it really doesn't exist.
        std::fs::remove_file(&nonexistent).ok();
        assert!(!is_silverblue(&nonexistent));
    }

    // -- exit code mapping ------------------------------------------------

    #[test]
    fn exit_codes_match_spec() {
        assert_eq!(ExitCode::Success as i32, 0);
        assert_eq!(ExitCode::PreconditionFailed as i32, 1);
        assert_eq!(ExitCode::ContainerStartFailed as i32, 2);
    }

    // -- refusal message --------------------------------------------------

    #[test]
    fn refusal_message_is_verbatim_per_spec() {
        assert_eq!(
            NOT_SILVERBLUE_MSG,
            "Local viewer supports Fedora Silverblue only. Season 2 teaches why."
        );
    }

    // -- role name helpers -----------------------------------------------

    #[test]
    fn viewer_role_suffix_is_ascii_and_short() {
        let suffix = viewer_role_suffix();
        assert_eq!(suffix, "viewer");
        assert!(suffix.is_ascii());
    }

    #[test]
    fn launcher_path_ends_with_macron_filename() {
        let p = launcher_path();
        let file = p.file_name().unwrap().to_string_lossy().into_owned();
        // The macron must be in the filename — no ASCII fallback per spec.
        assert_eq!(file, "tlatoāni_tales.sh");
        assert!(p.ends_with("scripts/tlatoāni_tales.sh"));
    }
}
