//! Tlatoāni Tales — visual QA loop client.
//!
//! Wraps the ollama VLM HTTP API. Produces a [`DriftReport`] per panel,
//! derives a reroll addendum from the failed checks, and exposes both to the
//! orchestrator's event bus. The check catalogue is the canonical list from
//! `openspec/specs/visual-qa-loop/spec.md` — the spec is the source of truth
//! for *which* checks exist; this crate is the source of truth for *how* to
//! ask the VLM about them.
//!
//! # Boundary
//!
//! Ollama lives across the isolation boundary — the `tlatoani-tales-inference`
//! container speaks HTTP on `127.0.0.1:11434` and nowhere else. This crate is
//! the **only** path out of the trusted Rust zone to the VLM. Governing specs:
//! `openspec/specs/visual-qa-loop/spec.md`, `openspec/specs/isolation/spec.md`.
//!
//! # Ollama protocol
//!
//! We call `/api/chat` with:
//!
//! - `model` — e.g. `moondream:2b`
//! - `messages` — a single user turn carrying the natural-language question
//!   bundle and the panel PNG (base64-encoded in the `images` array)
//! - `format: "json"` — asks ollama to constrain the response to JSON. We
//!   further pin the schema in-prompt by telling the VLM exactly which keys
//!   to emit. This is the "structured-output / JSON-schema" path documented
//!   in ollama's README; the strict JSON-schema mode is available on newer
//!   builds only, so we use the portable `format: "json"` + prompt-schema
//!   pattern to maximise compatibility across ollama releases.
//! - `stream: false` — we want the whole answer in one response.
//!
//! The response's `message.content` is a JSON string; we parse it back into a
//! `Vec<CheckAnswer>` keyed by check id.
//!
//! @trace spec:visual-qa-loop, spec:isolation
//! @Lesson S1-800
//! @Lesson S1-1000

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use tt_core::{PanelHash, SpecName, TtError};
use url::Url;

/// Default VLM model — see `visual-qa-loop/spec.md` §Runtime. Small enough
/// to run alongside FLUX on consumer hardware, sharp enough for
/// attribute-presence checks (tails, crown, plate position).
pub const DEFAULT_MODEL: &str = "moondream:2b";

/// One check definition — which `id` we track, which `spec` it enforces, and
/// the natural-language `question` we ask the VLM.
///
/// See the canonical list in `openspec/specs/visual-qa-loop/spec.md` §Drift
/// score schema; [`builtin_checks`] returns exactly that list.
///
/// @trace spec:visual-qa-loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
    pub id: String,
    pub spec: SpecName,
    pub question: String,
}

impl Check {
    fn new(id: &str, spec: &str, question: &str) -> Self {
        Self {
            id: id.into(),
            spec: SpecName::new(spec).expect("builtin check spec name is static + valid"),
            question: question.into(),
        }
    }
}

/// Canonical check catalogue from `openspec/specs/visual-qa-loop/spec.md`.
///
/// The question text is hand-crafted — one sentence per check, phrased so a
/// small VLM like `moondream:2b` can answer yes/no with a confidence. The
/// question text is a **judgment call** in this crate; the id/spec pair is
/// pulled verbatim from the spec.
///
/// @trace spec:visual-qa-loop, spec:character-canon, spec:style-bible, spec:trace-plate, spec:lessons
pub fn builtin_checks() -> Vec<Check> {
    vec![
        Check::new(
            "tlatoani.single-tail",
            "character-canon",
            "Does the axolotl character Tlatoāni have exactly one tail (not two, not zero)?",
        ),
        Check::new(
            "tlatoani.crown-present",
            "character-canon",
            "Is a small crown visible on Tlatoāni's head?",
        ),
        Check::new(
            "covi.ambiguous-white",
            "character-canon",
            "Is the character Covi rendered in ambiguous / off-white colouring (not pure white, not a distinct colour)?",
        ),
        Check::new(
            "covi.good-mood",
            "character-canon",
            "Does Covi's facial expression read as positive or neutral rather than dejected or angry?",
        ),
        Check::new(
            "palette.paper-bg",
            "style-bible",
            "Does the panel background read as warm paper tones rather than pure white or saturated colour?",
        ),
        Check::new(
            "plate.episode.position",
            "style-bible",
            "Is the episode plate positioned at the top of the panel as required by the style bible?",
        ),
        Check::new(
            "plate.trace-present",
            "trace-plate",
            "Is a trace plate (the `@trace spec:...` citation band) visible in the panel?",
        ),
        Check::new(
            "plate.trace-legible",
            "trace-plate",
            "Is the text of the trace plate sharp and readable at normal viewing size?",
        ),
        Check::new(
            "plate.trace-content",
            "trace-plate",
            "Does the trace plate's text match the declared `trace_spec` for this strip (no typos, no omissions)?",
        ),
        Check::new(
            "plate.lesson-present",
            "trace-plate",
            "Is a lesson plate (the `@Lesson S<n>-<NNN>` citation) visible in the panel?",
        ),
        Check::new(
            "plate.lesson-legible",
            "trace-plate",
            "Is the text of the lesson plate sharp and readable at normal viewing size?",
        ),
        Check::new(
            "plate.lesson-id-valid",
            "lessons",
            "Does the lesson plate reference a lesson id in the canonical `S<n>-<NNN>-<slug>` form?",
        ),
        Check::new(
            "plate.lesson-spec-aligned",
            "lessons",
            "Is the lesson plate's declared governing spec listed in that lesson's own coverage?",
        ),
        Check::new(
            "plate.title-present",
            "trace-plate",
            "Is a title plate visible in the top-left region of the panel by default?",
        ),
        Check::new(
            "plate.title-legible",
            "trace-plate",
            "Is the title plate's text sharp and readable?",
        ),
        Check::new(
            "plate.title-matches-declared",
            "trace-plate",
            "Does the title plate's text match the strip's declared `title` field from proposal.md?",
        ),
        Check::new(
            "plate.title-position-valid",
            "trace-plate",
            "Is the title plate in the top-left region, or if right-floated, declared as such in proposal.md?",
        ),
        Check::new(
            "plate.symmetry",
            "trace-plate",
            "Are the plates (title, trace, lesson, episode) laid out with symmetric / coherent alignment rather than haphazardly?",
        ),
        Check::new(
            "plate.episode-total-format",
            "trace-plate",
            "Does the episode plate read `Tlatoāni Tales NN/15` with the `/15` denominator present (not a bare `#NN`)?",
        ),
    ]
}

// ---------------------------------------------------------------------------
// Evaluated result types
// ---------------------------------------------------------------------------

/// One evaluated check inside a [`DriftReport`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub spec: SpecName,
    pub pass: bool,
    pub confidence: f32,
    pub note: Option<String>,
}

/// VLM verdict — mirrors [`tt_events::QaVerdict`] so we can emit without
/// translating types. Re-exported for convenience.
pub use tt_events::QaVerdict as Verdict;

/// Per-panel drift report — the telemetry primary artefact (`@Lesson S1-1000`
/// — dashboards must add observability).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub panel_hash: PanelHash,
    pub strip: String,
    pub panel: u8,
    pub iteration: u8,
    pub model: String,
    pub checks: Vec<CheckResult>,
    pub drift_score: f32,
    pub verdict: Verdict,
}

// ---------------------------------------------------------------------------
// Wire types — the VLM's JSON answer
// ---------------------------------------------------------------------------

/// One answer the VLM emits per check — what we ask it to produce in its
/// JSON response.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct CheckAnswer {
    id: String,
    pass: bool,
    confidence: f32,
    #[serde(default)]
    note: Option<String>,
}

/// Shape of the JSON body the VLM returns (inside ollama's `message.content`).
#[derive(Debug, Deserialize)]
struct VlmAnswer {
    checks: Vec<CheckAnswer>,
}

/// Subset of ollama's `/api/chat` response we care about.
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Handle to the ollama VLM.
///
/// @trace spec:visual-qa-loop, spec:isolation
pub struct QaClient {
    ollama_url: Url,
    model: String,
    client: reqwest::Client,
}

impl QaClient {
    /// Build a client pointing at the given ollama endpoint (e.g.
    /// `http://127.0.0.1:11434/`). Use [`DEFAULT_MODEL`] for the
    /// `moondream:2b` default documented in `visual-qa-loop/spec.md`.
    pub fn new(ollama_url: Url, model: impl Into<String>) -> Self {
        Self {
            ollama_url,
            model: model.into(),
            client: reqwest::Client::new(),
        }
    }

    /// The model this client will ask.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Critique a rendered panel against the provided check list.
    ///
    /// Calls ollama's `/api/chat` with the panel PNG base64-encoded in the
    /// `images` field and a prompt that bundles every check's id + question.
    /// The VLM is instructed to return a JSON object with a `checks` array;
    /// we parse that back into a [`DriftReport`] with computed drift score
    /// and verdict.
    ///
    /// `strip` is the strip label (e.g. `"TT 03/15"`), `panel` is the panel
    /// number, `iteration` is the 1-based reroll count, and `panel_hash` is
    /// the content-address of the PNG being critiqued.
    pub async fn critique(
        &self,
        panel_png: &[u8],
        checks: &[Check],
        strip: impl Into<String>,
        panel: u8,
        iteration: u8,
        panel_hash: PanelHash,
    ) -> Result<DriftReport, TtError> {
        if checks.is_empty() {
            return Err(TtError::Usage(
                "tt-qa critique: check list is empty — nothing to ask the VLM".into(),
            ));
        }

        let prompt = build_prompt(checks);
        let image_b64 = base64::engine::general_purpose::STANDARD.encode(panel_png);

        let endpoint = self
            .ollama_url
            .join("api/chat")
            .map_err(|e| TtError::Infra(format!("ollama url join failed: {e}")))?;

        let body = serde_json::json!({
            "model": self.model,
            "stream": false,
            "format": "json",
            "messages": [{
                "role": "user",
                "content": prompt,
                "images": [image_b64],
            }]
        });

        let resp = self
            .client
            .post(endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| TtError::Infra(format!("ollama request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(TtError::Infra(format!(
                "ollama returned {status}: {body}"
            )));
        }

        let chat: OllamaChatResponse = resp
            .json()
            .await
            .map_err(|e| TtError::Infra(format!("ollama response not JSON: {e}")))?;

        let answer: VlmAnswer = serde_json::from_str(&chat.message.content).map_err(|e| {
            TtError::Parse(format!(
                "VLM `message.content` is not a valid JSON CheckAnswer bundle: {e}; raw: {}",
                chat.message.content
            ))
        })?;

        let results = align_answers(checks, &answer.checks);
        let drift = score(&results);
        let verdict = verdict_from_drift(drift);

        Ok(DriftReport {
            panel_hash,
            strip: strip.into(),
            panel,
            iteration,
            model: self.model.clone(),
            checks: results,
            drift_score: drift,
            verdict,
        })
    }
}

// ---------------------------------------------------------------------------
// Prompt assembly
// ---------------------------------------------------------------------------

/// Assemble the user prompt: a schema preamble + per-check question list.
///
/// The format is deliberately verbose — `moondream:2b` sometimes drops the
/// `checks` envelope if the request is terse, so we repeat "return a JSON
/// object with a `checks` array" both in natural language and in an example.
fn build_prompt(checks: &[Check]) -> String {
    let mut s = String::with_capacity(1024 + checks.len() * 200);
    s.push_str(
        "You are a visual-QA critic. Look at the image and answer every check.\n\
         Respond with a single JSON object of the form:\n\
         {\"checks\":[{\"id\":\"<id>\",\"pass\":true|false,\"confidence\":0.0-1.0,\"note\":\"optional short reason\"},...]}\n\
         Do not include any text outside the JSON. `confidence` is how sure you are of the `pass` verdict.\n\n\
         Checks:\n",
    );
    for c in checks {
        s.push_str("- id: ");
        s.push_str(&c.id);
        s.push_str("\n  question: ");
        s.push_str(&c.question);
        s.push('\n');
    }
    s
}

/// Align the VLM's answers with the requested check list, filling any
/// missing answer with a low-confidence failure so drift scoring stays safe.
fn align_answers(checks: &[Check], answers: &[CheckAnswer]) -> Vec<CheckResult> {
    checks
        .iter()
        .map(|c| match answers.iter().find(|a| a.id == c.id) {
            Some(a) => CheckResult {
                id: c.id.clone(),
                spec: c.spec.clone(),
                pass: a.pass,
                confidence: a.confidence.clamp(0.0, 1.0),
                note: a.note.clone(),
            },
            None => CheckResult {
                id: c.id.clone(),
                spec: c.spec.clone(),
                pass: false,
                confidence: 0.0,
                note: Some("VLM did not return this check — treated as failure".into()),
            },
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Scoring & verdict
// ---------------------------------------------------------------------------

/// Drift score: for each failed check, accumulate `1.0 - confidence`, then
/// normalise by the number of checks. Range 0.0..=1.0.
///
/// Matches `openspec/specs/visual-qa-loop/spec.md` §Drift score schema.
pub fn score(checks: &[CheckResult]) -> f32 {
    if checks.is_empty() {
        return 0.0;
    }
    let raw: f32 = checks
        .iter()
        .filter(|c| !c.pass)
        .map(|c| (1.0 - c.confidence).max(0.0))
        .sum();
    (raw / checks.len() as f32).clamp(0.0, 1.0)
}

/// Verdict from a drift score, matching the task's thresholds:
/// `< 0.05` → Stable, `0.05..0.20` → Reroll, `>= 0.20` → Escalate.
pub fn verdict_from_drift(drift: f32) -> Verdict {
    if drift < 0.05 {
        Verdict::Stable
    } else if drift < 0.20 {
        Verdict::Reroll
    } else {
        Verdict::Escalate
    }
}

// ---------------------------------------------------------------------------
// Addendum composer — C13 (the loop closes) made mechanical
// ---------------------------------------------------------------------------

/// Compose a terse negative-prompt-style addendum from a drift report's
/// failed checks. The output is fed back into the next render prompt so the
/// loop literally closes (`@Lesson S1-1300` — loop closes, mirrored here in
/// the QA domain).
///
/// Format: `avoid: <id> (<note>); <id> (<note>); ...`
/// — each failing check contributes one clause. If a check has no `note`,
/// the id alone is emitted. Empty output if no checks failed.
pub fn derive_addendum(report: &DriftReport) -> String {
    let clauses: Vec<String> = report
        .checks
        .iter()
        .filter(|c| !c.pass)
        .map(|c| match &c.note {
            Some(n) if !n.is_empty() => format!("{} ({})", c.id, n),
            _ => c.id.clone(),
        })
        .collect();
    if clauses.is_empty() {
        String::new()
    } else {
        format!("avoid: {}", clauses.join("; "))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn cr(id: &str, pass: bool, conf: f32, note: Option<&str>) -> CheckResult {
        CheckResult {
            id: id.into(),
            spec: SpecName::new("visual-qa-loop").unwrap(),
            pass,
            confidence: conf,
            note: note.map(|s| s.into()),
        }
    }

    fn sample_report(checks: Vec<CheckResult>) -> DriftReport {
        let drift = score(&checks);
        DriftReport {
            panel_hash: PanelHash::from_bytes([0u8; 32]),
            strip: "TT 03/15".into(),
            panel: 2,
            iteration: 1,
            model: "moondream:2b".into(),
            checks,
            drift_score: drift,
            verdict: verdict_from_drift(drift),
        }
    }

    // -- score -------------------------------------------------------------

    #[test]
    fn score_all_passing_is_zero() {
        let checks = vec![
            cr("a", true, 0.9, None),
            cr("b", true, 0.8, None),
            cr("c", true, 1.0, None),
        ];
        assert_eq!(score(&checks), 0.0);
    }

    #[test]
    fn score_single_full_fail_normalised() {
        // 1 check, failed, confidence 0 → raw = 1.0 → normalised = 1.0.
        let checks = vec![cr("a", false, 0.0, None)];
        assert!((score(&checks) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn score_mix_normalises_by_count() {
        // 4 checks: two fail at confidence 0.5 → raw = 0.5 + 0.5 = 1.0.
        // normalised = 1.0 / 4 = 0.25.
        let checks = vec![
            cr("a", true, 0.9, None),
            cr("b", false, 0.5, None),
            cr("c", false, 0.5, None),
            cr("d", true, 1.0, None),
        ];
        assert!((score(&checks) - 0.25).abs() < 1e-6);
    }

    #[test]
    fn score_confidence_only_counted_for_failed() {
        // A passing check with low confidence contributes nothing.
        let checks = vec![cr("a", true, 0.1, None), cr("b", false, 0.9, None)];
        // raw = 1.0 - 0.9 = 0.1, normalised over 2 checks = 0.05.
        assert!((score(&checks) - 0.05).abs() < 1e-6);
    }

    #[test]
    fn score_empty_is_zero() {
        assert_eq!(score(&[]), 0.0);
    }

    // -- verdict thresholds ------------------------------------------------

    #[test]
    fn verdict_stable_below_005() {
        assert_eq!(verdict_from_drift(0.0), Verdict::Stable);
        assert_eq!(verdict_from_drift(0.049), Verdict::Stable);
    }

    #[test]
    fn verdict_reroll_between_005_and_020() {
        assert_eq!(verdict_from_drift(0.05), Verdict::Reroll);
        assert_eq!(verdict_from_drift(0.1), Verdict::Reroll);
        assert_eq!(verdict_from_drift(0.199), Verdict::Reroll);
    }

    #[test]
    fn verdict_escalate_at_or_above_020() {
        assert_eq!(verdict_from_drift(0.20), Verdict::Escalate);
        assert_eq!(verdict_from_drift(0.3), Verdict::Escalate);
        assert_eq!(verdict_from_drift(1.0), Verdict::Escalate);
    }

    // -- derive_addendum ---------------------------------------------------

    #[test]
    fn addendum_empty_when_no_failures() {
        let rep = sample_report(vec![cr("a", true, 0.9, None), cr("b", true, 0.8, None)]);
        assert_eq!(derive_addendum(&rep), "");
    }

    #[test]
    fn addendum_emits_failed_ids_and_notes() {
        let rep = sample_report(vec![
            cr("covi.good-mood", false, 0.7, Some("expression reads dejected")),
            cr("plate.trace-legible", false, 0.3, None),
            cr("palette.paper-bg", true, 0.9, None),
        ]);
        let add = derive_addendum(&rep);
        assert!(add.starts_with("avoid: "));
        assert!(add.contains("covi.good-mood (expression reads dejected)"));
        assert!(add.contains("plate.trace-legible"));
        // passing check must not appear.
        assert!(!add.contains("palette.paper-bg"));
        // two clauses → exactly one "; " separator.
        assert_eq!(add.matches("; ").count(), 1);
    }

    // -- builtin_checks ----------------------------------------------------

    #[test]
    fn builtin_checks_match_spec_list() {
        let checks = builtin_checks();
        let ids: Vec<&str> = checks.iter().map(|c| c.id.as_str()).collect();
        // Spot-check the canonical ids from visual-qa-loop/spec.md.
        for expected in [
            "tlatoani.single-tail",
            "tlatoani.crown-present",
            "covi.ambiguous-white",
            "covi.good-mood",
            "palette.paper-bg",
            "plate.episode.position",
            "plate.trace-present",
            "plate.trace-legible",
            "plate.trace-content",
            "plate.lesson-present",
            "plate.lesson-legible",
            "plate.lesson-id-valid",
            "plate.lesson-spec-aligned",
            "plate.title-present",
            "plate.title-legible",
            "plate.title-matches-declared",
            "plate.title-position-valid",
            "plate.symmetry",
            "plate.episode-total-format",
        ] {
            assert!(
                ids.contains(&expected),
                "builtin_checks missing id {expected}"
            );
        }
        // Every question must be non-empty.
        for c in builtin_checks() {
            assert!(!c.question.is_empty(), "check {} has empty question", c.id);
        }
    }

    // -- align_answers -----------------------------------------------------

    #[test]
    fn align_answers_fills_missing_as_failure() {
        let checks = vec![
            Check::new("a", "visual-qa-loop", "q-a"),
            Check::new("b", "visual-qa-loop", "q-b"),
        ];
        let answers = vec![CheckAnswer {
            id: "a".into(),
            pass: true,
            confidence: 0.9,
            note: None,
        }];
        let results = align_answers(&checks, &answers);
        assert_eq!(results.len(), 2);
        assert!(results[0].pass);
        assert!(!results[1].pass); // missing → failure
        assert_eq!(results[1].confidence, 0.0);
        assert!(results[1].note.is_some());
    }

    #[test]
    fn align_answers_clamps_confidence() {
        let checks = vec![Check::new("a", "visual-qa-loop", "q")];
        let answers = vec![CheckAnswer {
            id: "a".into(),
            pass: true,
            confidence: 2.5, // out of range
            note: None,
        }];
        let results = align_answers(&checks, &answers);
        assert_eq!(results[0].confidence, 1.0);
    }

    // -- prompt assembly ---------------------------------------------------

    #[test]
    fn build_prompt_includes_every_check_id_and_question() {
        let checks = builtin_checks();
        let p = build_prompt(&checks);
        for c in &checks {
            assert!(p.contains(&c.id), "prompt missing id {}", c.id);
            assert!(p.contains(&c.question), "prompt missing question for {}", c.id);
        }
        assert!(p.contains("\"checks\""));
    }

    // -- critique end-to-end via wiremock ---------------------------------

    #[tokio::test]
    async fn critique_round_trip_against_mock_ollama() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;

        // The VLM answers 3 checks: 2 pass, 1 fails with a note.
        let vlm_body = serde_json::json!({
            "checks": [
                {"id": "alpha", "pass": true, "confidence": 0.92},
                {"id": "beta", "pass": false, "confidence": 0.7, "note": "mood dejected"},
                {"id": "gamma", "pass": true, "confidence": 0.88},
            ]
        });
        let ollama_response = serde_json::json!({
            "message": {
                "role": "assistant",
                "content": vlm_body.to_string(),
            }
        });

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ollama_response))
            .mount(&server)
            .await;

        let url = Url::parse(&format!("{}/", server.uri())).unwrap();
        let client = QaClient::new(url, "moondream:2b");

        let checks = vec![
            Check::new("alpha", "visual-qa-loop", "q-alpha"),
            Check::new("beta", "visual-qa-loop", "q-beta"),
            Check::new("gamma", "visual-qa-loop", "q-gamma"),
        ];
        let fake_png: Vec<u8> = b"\x89PNG\r\n\x1a\n".to_vec();

        let report = client
            .critique(
                &fake_png,
                &checks,
                "TT 03/15",
                2,
                1,
                PanelHash::from_bytes([0u8; 32]),
            )
            .await
            .expect("critique should succeed against mock");

        assert_eq!(report.strip, "TT 03/15");
        assert_eq!(report.panel, 2);
        assert_eq!(report.iteration, 1);
        assert_eq!(report.model, "moondream:2b");
        assert_eq!(report.checks.len(), 3);
        assert!(report.checks[0].pass);
        assert!(!report.checks[1].pass);
        assert_eq!(
            report.checks[1].note.as_deref(),
            Some("mood dejected"),
        );
        assert!(report.checks[2].pass);

        // Drift: 1 failed at conf 0.7 → raw 0.3, normalised over 3 → 0.1.
        assert!((report.drift_score - 0.1).abs() < 1e-6);
        assert_eq!(report.verdict, Verdict::Reroll);

        // Addendum surfaces the failed check and its note.
        let add = derive_addendum(&report);
        assert!(add.contains("beta (mood dejected)"));
        assert!(!add.contains("alpha"));
    }

    #[tokio::test]
    async fn critique_errors_when_ollama_returns_non_json_content() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let ollama_response = serde_json::json!({
            "message": {
                "role": "assistant",
                "content": "sorry I cannot comply",
            }
        });
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ollama_response))
            .mount(&server)
            .await;

        let url = Url::parse(&format!("{}/", server.uri())).unwrap();
        let client = QaClient::new(url, DEFAULT_MODEL);

        let err = client
            .critique(
                b"x",
                &[Check::new("a", "visual-qa-loop", "q")],
                "TT 01/15",
                1,
                1,
                PanelHash::from_bytes([0u8; 32]),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, TtError::Parse(_)));
    }

    #[tokio::test]
    async fn critique_rejects_empty_check_list() {
        let url = Url::parse("http://127.0.0.1:11434/").unwrap();
        let client = QaClient::new(url, DEFAULT_MODEL);
        let err = client
            .critique(b"x", &[], "TT 01/15", 1, 1, PanelHash::from_bytes([0u8; 32]))
            .await
            .unwrap_err();
        assert!(matches!(err, TtError::Usage(_)));
    }
}
