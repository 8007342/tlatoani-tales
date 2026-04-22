//! Tlatoāni Tales — typed event bus.
//!
//! The orchestrator is "a typed event bus where every state transition is
//! observable" (governing spec: `openspec/specs/orchestrator/spec.md`). This
//! crate declares the bus and the per-domain event enums. Every variant
//! carries an optional `spec_tag` and `lesson_tag` — the Rust equivalent of
//! `@trace spec:<name>` and `@Lesson Sn-NNN`.
//!
//! Subscribers are cheap: `Bus::subscribe()` returns a broadcast receiver
//! that can be adapted into a `futures::Stream` via tokio's stream helpers.
//!
// @trace spec:orchestrator
// @Lesson S1-800
// @Lesson S1-1300

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tt_core::{FailureClass, LessonId, PanelHash, SpecName, StripId};

/// Every event in the workspace is one of these variants. Subscribers that
/// only care about one domain filter on the outer variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Render(RenderEvent),
    Comfy(ComfyEvent),
    Qa(QaEvent),
    Compose(ComposeEvent),
    Cache(CacheEvent),
    Lint(LintEvent),
    Lora(LoraEvent),
}

/// Orchestrator-level lifecycle events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderEvent {
    RunStarted {
        run_id: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    StripDiscovered {
        strip: StripId,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    SpecLoaded {
        spec_name: SpecName,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    PanelHashComputed {
        strip: StripId,
        panel: u8,
        panel_hash: PanelHash,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    CacheHit {
        strip: StripId,
        panel: u8,
        panel_hash: PanelHash,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    CacheMiss {
        strip: StripId,
        panel: u8,
        panel_hash: PanelHash,
        reason: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    RunComplete {
        run_id: String,
        strips_rendered: u32,
        strips_cached: u32,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    RunFailed {
        run_id: String,
        class: FailureClass,
        detail: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
}

/// Events from the ComfyUI HTTP client (`tt-comfy`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComfyEvent {
    Submitted {
        strip: StripId,
        panel: u8,
        panel_hash: PanelHash,
        prompt_id: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    Progress {
        prompt_id: String,
        step: u32,
        total: u32,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    Rendered {
        prompt_id: String,
        output_path: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    Failed {
        prompt_id: String,
        error_kind: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    Timeout {
        prompt_id: String,
        elapsed_ms: u64,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
}

/// Events from the VLM critique client (`tt-qa`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QaEvent {
    Submitted {
        strip: StripId,
        panel: u8,
        iteration: u32,
        model: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    CheckResult {
        strip: StripId,
        panel: u8,
        iteration: u32,
        check_id: String,
        pass: bool,
        confidence: f32,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    Verdict {
        strip: StripId,
        panel: u8,
        iteration: u32,
        drift_score: f32,
        verdict: QaVerdict,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    RerollScheduled {
        strip: StripId,
        panel: u8,
        iteration_next: u32,
        addendum: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
}

/// VLM verdict — mirrors `openspec/specs/visual-qa-loop/spec.md` §Thresholds.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QaVerdict {
    Stable,
    Reroll,
    Escalate,
    NeedsHuman,
}

/// Events from the composer (`tt-compose`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComposeEvent {
    PanelsLoaded {
        strip: StripId,
        hashes: Vec<PanelHash>,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    PlatesRendered {
        strip: StripId,
        plate_kinds: Vec<String>,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    TitleComposited {
        strip: StripId,
        title_display: String,
        source: String, // "Qwen-Image"
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    ComposeDone {
        strip: StripId,
        output_path: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    MetadataWritten {
        strip: StripId,
        metadata_path: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
}

/// Events from the content-addressed panel cache.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheEvent {
    HashComputed {
        panel_hash: PanelHash,
        inputs: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    Hit {
        panel_hash: PanelHash,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    Miss {
        panel_hash: PanelHash,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    Promoted {
        panel_hash: PanelHash,
        png_path: String,
        report_path: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    Evicted {
        panel_hash: PanelHash,
        reason: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
}

/// Events from the `verify` / lint subcommand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LintEvent {
    Started {
        rules_scope: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    RuleViolated {
        rule_id: String,
        path: String,
        detail: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    Passed {
        rules_checked: u32,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    Failed {
        violations: u32,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
}

/// Events from the LoRA trainer wrapper (`tt-lora`).
///
/// See `openspec/specs/character-loras/spec.md` §Event emission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoraEvent {
    TrainStarted {
        character: String,
        version: u32,
        config_hash: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    StepProgress {
        character: String,
        step: u32,
        total_steps: u32,
        loss: f32,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    SanityRenderDone {
        character: String,
        prompt: String,
        drift_score: f32,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    Trained {
        character: String,
        manifest_path: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
    Failed {
        character: String,
        reason: String,
        spec_tag: Option<SpecName>,
        lesson_tag: Option<LessonId>,
    },
}

/// Broadcast bus shared by every crate that emits or observes events.
///
/// Subscribers get their own `Receiver`; if they lag, broadcast's backlog
/// fills and `recv()` returns `RecvError::Lagged(n)` — fail loudly, don't
/// paper over missed events.
#[derive(Clone)]
pub struct Bus {
    sender: broadcast::Sender<Event>,
}

impl Bus {
    /// Create a bus with the given channel capacity. Capacity tuning is an
    /// operational concern; 1024 is the reasonable default the orchestrator
    /// uses today.
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Create a bus with the default capacity.
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Subscribe to every event emitted from now on.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    /// Emit an event. Returns the number of active subscribers that will see
    /// the event, or 0 if nobody is listening (which is fine).
    pub fn emit(&self, event: Event) -> usize {
        self.sender.send(event).unwrap_or(0)
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}
