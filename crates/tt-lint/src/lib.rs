//! Tlatoāni Tales — spec-invariant linter.
//!
//! Fast, pure, pre-commit-friendly. Walks the project, applies one rule per
//! governing spec, and returns a structured `LintReport`. Any violation is a
//! canon failure (exit 20) per `openspec/specs/orchestrator/spec.md`
//! §Failure modes.
//!
//! Implemented rule families (scaffolded; bodies land in a later change):
//!
//! - **License coverage** — every committed file matches an R## rule in
//!   `openspec/specs/licensing/spec.md`.
//! - **Trace presence** — source, specs, and scripts carry the declared
//!   `@trace spec:<name>` citations.
//! - **Tlatoāni spelling** — `\bTlatoani\b` appears only in catalogued
//!   teachable-break allowlist locations (TB01..TBNN).
//! - **Plate declaration** — every strip proposal declares title, lesson,
//!   trace_spec, and plate fields per `trace-plate/spec.md`.
//! - **Slug in registry** — every `@Lesson Sn-NNN` resolves to a lesson in
//!   `lessons/spec.md`.
//! - **Spec in lesson coverage** — a strip's `trace_spec` appears in the
//!   declared lesson's coverage list.
//! - **Isolation violations** — every `podman run` in source matches
//!   `tt-core::podman::DEFAULT_FLAGS` + no forbidden flags.
//!
// @trace spec:licensing, spec:trace-plate, spec:lessons, spec:tlatoāni-spelling, spec:isolation
// @Lesson S1-500

use serde::{Deserialize, Serialize};
use std::path::Path;
use tt_core::TtError;

/// Rule family — each variant carries the violations it produced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LintRule {
    LicenseCoverage(Vec<Violation>),
    TracePresence(Vec<Violation>),
    TlatoaniSpelling(Vec<Violation>),
    PlateDeclaration(Vec<Violation>),
    SlugInRegistry(Vec<Violation>),
    SpecInLessonCoverage(Vec<Violation>),
    IsolationViolations(Vec<Violation>),
}

/// A single rule violation — identifies where and why.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub rule_id: String,
    pub path: String,
    pub line: Option<u32>,
    pub detail: String,
}

/// Aggregate report from a `verify_all` run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LintReport {
    pub rules: Vec<LintRule>,
    pub total_violations: u32,
}

impl LintReport {
    /// `true` if no rule produced a violation.
    pub fn is_clean(&self) -> bool {
        self.total_violations == 0
    }
}

/// Run every rule against `project_dir`. Returns a structured `LintReport`
/// (the binary decides whether to exit 20 based on `is_clean`).
pub async fn verify_all(_project_dir: &Path) -> Result<LintReport, TtError> {
    // Scaffold: return a clean report so the workspace compiles and downstream
    // callers can wire the flow. Real implementations for each rule land in
    // subsequent changes, one per rule family.
    Ok(LintReport::default())
}
