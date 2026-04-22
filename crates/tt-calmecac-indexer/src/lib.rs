//! Tlatoāni Tales — Calmecac concept-index builder.
//!
//! Reads substrate (markdown, paths, dates, hashes) and emits concepts
//! (lessons, rules, changes). The substrate-erasing step is load-bearing:
//! filenames and hashes are parsed and *thrown away* before the index is
//! written, so the bundle the untrusted httpd container serves never
//! contains path-shaped strings or commit hashes. The UI does not need to
//! remember to hide substrate — the substrate never reaches it.
//!
//! Governing spec: `openspec/specs/calmecac/spec.md` §Concept index
//! generation.
//!
// @trace spec:calmecac, spec:orchestrator
// @Lesson S1-1000
// @Lesson S1-1500

use std::path::Path;
use tt_core::TtError;

/// Build the concept index for a project checkout and write it to `out` as
/// JSON. Runs at build time, emits once, exits — the indexer is NOT a web
/// service and the httpd container never invokes it.
pub async fn build_index(_project_dir: &Path, _out: &Path) -> Result<(), TtError> {
    unimplemented!("tt-calmecac-indexer build_index is scaffolded; real walk lands in a later change")
}
