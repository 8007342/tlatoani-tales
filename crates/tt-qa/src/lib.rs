//! Tlatoāni Tales — visual QA loop client.
//!
//! Wraps the ollama VLM HTTP API. Loads check definitions from
//! `openspec/specs/visual-qa-loop/spec.md` at startup (it does NOT hard-code
//! the check list — the spec is the source of truth). Produces a
//! `DriftReport` per panel, derives a reroll addendum from the failed
//! checks, and exposes both to the orchestrator's event bus.
//!
//! Governing spec: `openspec/specs/visual-qa-loop/spec.md`,
//! `openspec/specs/trace-plate/spec.md`.
//!
// @trace spec:visual-qa-loop, spec:trace-plate
// @Lesson S1-800
// @Lesson S1-1300

use serde::{Deserialize, Serialize};
use std::path::Path;
use tt_core::{SpecName, TtError};
use tt_events::QaVerdict;
use url::Url;

/// One check from `visual-qa-loop/spec.md` — e.g. `tlatoāni.single-tail`,
/// `plate.title-matches-declared`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
    pub id: String,
    pub spec: SpecName,
    pub note: Option<String>,
}

/// One evaluated check inside a `DriftReport`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub spec: SpecName,
    pub pass: bool,
    pub confidence: f32,
    pub note: Option<String>,
}

/// Per-panel drift report — the telemetry primary artefact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub panel_hash: String,
    pub strip: String,
    pub panel: u8,
    pub iteration: u32,
    pub model: String,
    pub checks: Vec<CheckResult>,
    pub drift_score: f32,
    pub verdict: QaVerdict,
}

/// Handle to the ollama VLM.
pub struct QaClient {
    #[allow(dead_code)]
    ollama_url: Url,
}

impl QaClient {
    /// Build a client pointing at the given ollama endpoint (e.g.
    /// `http://127.0.0.1:11434/`).
    pub fn new(ollama_url: Url) -> Self {
        Self { ollama_url }
    }

    /// Critique a rendered panel against the provided check list.
    pub async fn critique(
        &self,
        _panel_png: &Path,
        _checks: &[Check],
    ) -> Result<DriftReport, TtError> {
        unimplemented!("tt-qa critique is scaffolded; real VLM loop lands in a later change")
    }

    /// Derive a prompt addendum from a drift report's failed checks.
    ///
    /// Format: a terse negative-direction string like
    /// `"avoid: covi.good-mood (expression reads dejected); ..."`. Mirrors
    /// `openspec/specs/visual-qa-loop/spec.md` §The loop.
    pub fn derive_addendum(_report: &DriftReport) -> String {
        String::new()
    }
}
