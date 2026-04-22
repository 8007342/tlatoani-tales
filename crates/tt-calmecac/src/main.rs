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
//! Governing spec: `openspec/specs/calmecac/spec.md`.
//!
// @trace spec:calmecac, spec:isolation
// @Lesson S1-1000

use clap::{Parser, Subcommand};

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
        /// Override the default port.
        #[arg(long)]
        port: Option<u16>,
    },
    /// Stop the viewer container and exit.
    Stop,
    /// Rebuild the viewer image, then flow into `serve`.
    Rebuild,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Build => {
            tracing::info!(spec = "calmecac", "tt-calmecac: build");
            // Scaffold: the indexer stub panics with `unimplemented!`; don't
            // call it yet. When the real implementation lands, replace this
            // log with the actual invocation.
            Ok(())
        }
        Command::Serve { port: _ } => {
            ensure_silverblue()?;
            tracing::info!(spec = "calmecac", "tt-calmecac: serve");
            Ok(())
        }
        Command::Stop => {
            tracing::info!(spec = "calmecac", "tt-calmecac: stop");
            Ok(())
        }
        Command::Rebuild => {
            ensure_silverblue()?;
            tracing::info!(spec = "calmecac", "tt-calmecac: rebuild");
            Ok(())
        }
    }
}

/// Silverblue-only guard per `openspec/specs/calmecac/spec.md`.
///
/// Reads `/etc/os-release`; accepts only `VARIANT_ID=silverblue`. Scaffold:
/// returns `Ok(())` — the actual enforcement lands alongside the viewer
/// launcher in a later change. Leaving the function in place so every
/// viewer-managing subcommand already threads through it.
// @trace spec:calmecac
fn ensure_silverblue() -> anyhow::Result<()> {
    Ok(())
}

// Keep the indexer in the dependency graph. Real call site lives inside
// `Command::Build` once `build_index` is implemented.
#[allow(unused_imports)]
use {tt_calmecac_indexer as _, tt_core as _};
