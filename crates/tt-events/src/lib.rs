//! Tlatoāni Tales — typed event bus.
//!
//! The orchestrator is "a typed event bus where every state transition is
//! observable" (governing spec: `openspec/specs/orchestrator/spec.md`). This
//! crate declares the bus and the per-domain event enums. Every variant
//! carries an optional `spec_tag` and `lesson_tag` — the Rust equivalent of
//! `@trace spec:<name>` and `@Lesson Sn-NNN`.
//!
//! The bus itself is the substrate of `@Lesson S1-1300` (*the loop closes*):
//! telemetry from iteration `i` re-enters the system as input to iteration
//! `i+1`. Subscribers compose as `futures::Stream`s and may filter by spec,
//! lesson, or an arbitrary predicate — `openspec/specs/orchestrator/spec.md`
//! §Observable streams.
//!
//! # Channel capacity
//!
//! The default broadcast capacity is **1024**. Rationale: the busiest
//! observable run (a full 15-strip rebuild with QA enabled) emits on the
//! order of a few hundred events per strip — Submit/Progress/Rendered from
//! `tt-comfy`, one CheckResult per check from `tt-qa`, Hit/Miss/Promoted
//! from the cache, plus Compose/Metadata events. 1024 gives every
//! reasonable subscriber (CLI, JSONL sink, cache manager, optional
//! Calmecac SSE bridge) headroom for a per-strip burst without dropping
//! events. Operators with exotic fan-out may use [`Bus::with_capacity`].
//!
//! # Lag policy
//!
//! `tokio::sync::broadcast` drops the oldest messages for a slow receiver
//! and surfaces the drop as `RecvError::Lagged(n)`. We **propagate the lag
//! as a typed error** rather than silently skipping — the orchestrator
//! invariant is "no hidden state mutations" and a dropped event is a
//! hidden state mutation. Subscribers decide how to react (telemetry
//! sinks may treat Lagged as fatal; a live CLI may log and keep going).
//!
//! @trace spec:orchestrator
//! @Lesson S1-800
//! @Lesson S1-1300

use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tt_core::{FailureClass, LessonId, PanelHash, SpecName, StripId};

// ---------------------------------------------------------------------------
// Top-level event enum
// ---------------------------------------------------------------------------

/// Every event in the workspace is one of these variants. Subscribers that
/// only care about one domain filter on the outer variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// Orchestrator-level lifecycle transitions.
    Render(RenderEvent),
    /// Events from the ComfyUI HTTP client.
    Comfy(ComfyEvent),
    /// Events from the VLM critique client.
    Qa(QaEvent),
    /// Events from the composer.
    Compose(ComposeEvent),
    /// Events from the content-addressed panel cache.
    Cache(CacheEvent),
    /// Events from the `verify` / lint subcommand.
    Lint(LintEvent),
    /// Events from the LoRA trainer wrapper.
    Lora(LoraEvent),
}

impl Event {
    /// The `@trace spec:<name>` citation carried by this event, if any.
    pub fn spec_tag(&self) -> Option<&SpecName> {
        match self {
            Event::Render(e) => e.spec_tag(),
            Event::Comfy(e) => e.spec_tag(),
            Event::Qa(e) => e.spec_tag(),
            Event::Compose(e) => e.spec_tag(),
            Event::Cache(e) => e.spec_tag(),
            Event::Lint(e) => e.spec_tag(),
            Event::Lora(e) => e.spec_tag(),
        }
    }

    /// The `@Lesson Sn-NNN` citation carried by this event, if any.
    pub fn lesson_tag(&self) -> Option<&LessonId> {
        match self {
            Event::Render(e) => e.lesson_tag(),
            Event::Comfy(e) => e.lesson_tag(),
            Event::Qa(e) => e.lesson_tag(),
            Event::Compose(e) => e.lesson_tag(),
            Event::Cache(e) => e.lesson_tag(),
            Event::Lint(e) => e.lesson_tag(),
            Event::Lora(e) => e.lesson_tag(),
        }
    }
}

// ---------------------------------------------------------------------------
// Domain events
// ---------------------------------------------------------------------------

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

impl RenderEvent {
    /// Spec citation carried by this variant.
    pub fn spec_tag(&self) -> Option<&SpecName> {
        match self {
            RenderEvent::RunStarted { spec_tag, .. }
            | RenderEvent::StripDiscovered { spec_tag, .. }
            | RenderEvent::SpecLoaded { spec_tag, .. }
            | RenderEvent::PanelHashComputed { spec_tag, .. }
            | RenderEvent::CacheHit { spec_tag, .. }
            | RenderEvent::CacheMiss { spec_tag, .. }
            | RenderEvent::RunComplete { spec_tag, .. }
            | RenderEvent::RunFailed { spec_tag, .. } => spec_tag.as_ref(),
        }
    }

    /// Lesson citation carried by this variant.
    pub fn lesson_tag(&self) -> Option<&LessonId> {
        match self {
            RenderEvent::RunStarted { lesson_tag, .. }
            | RenderEvent::StripDiscovered { lesson_tag, .. }
            | RenderEvent::SpecLoaded { lesson_tag, .. }
            | RenderEvent::PanelHashComputed { lesson_tag, .. }
            | RenderEvent::CacheHit { lesson_tag, .. }
            | RenderEvent::CacheMiss { lesson_tag, .. }
            | RenderEvent::RunComplete { lesson_tag, .. }
            | RenderEvent::RunFailed { lesson_tag, .. } => lesson_tag.as_ref(),
        }
    }
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

impl ComfyEvent {
    pub fn spec_tag(&self) -> Option<&SpecName> {
        match self {
            ComfyEvent::Submitted { spec_tag, .. }
            | ComfyEvent::Progress { spec_tag, .. }
            | ComfyEvent::Rendered { spec_tag, .. }
            | ComfyEvent::Failed { spec_tag, .. }
            | ComfyEvent::Timeout { spec_tag, .. } => spec_tag.as_ref(),
        }
    }

    pub fn lesson_tag(&self) -> Option<&LessonId> {
        match self {
            ComfyEvent::Submitted { lesson_tag, .. }
            | ComfyEvent::Progress { lesson_tag, .. }
            | ComfyEvent::Rendered { lesson_tag, .. }
            | ComfyEvent::Failed { lesson_tag, .. }
            | ComfyEvent::Timeout { lesson_tag, .. } => lesson_tag.as_ref(),
        }
    }
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

impl QaEvent {
    pub fn spec_tag(&self) -> Option<&SpecName> {
        match self {
            QaEvent::Submitted { spec_tag, .. }
            | QaEvent::CheckResult { spec_tag, .. }
            | QaEvent::Verdict { spec_tag, .. }
            | QaEvent::RerollScheduled { spec_tag, .. } => spec_tag.as_ref(),
        }
    }

    pub fn lesson_tag(&self) -> Option<&LessonId> {
        match self {
            QaEvent::Submitted { lesson_tag, .. }
            | QaEvent::CheckResult { lesson_tag, .. }
            | QaEvent::Verdict { lesson_tag, .. }
            | QaEvent::RerollScheduled { lesson_tag, .. } => lesson_tag.as_ref(),
        }
    }
}

/// VLM verdict — mirrors `openspec/specs/visual-qa-loop/spec.md` §Thresholds.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

impl ComposeEvent {
    pub fn spec_tag(&self) -> Option<&SpecName> {
        match self {
            ComposeEvent::PanelsLoaded { spec_tag, .. }
            | ComposeEvent::PlatesRendered { spec_tag, .. }
            | ComposeEvent::TitleComposited { spec_tag, .. }
            | ComposeEvent::ComposeDone { spec_tag, .. }
            | ComposeEvent::MetadataWritten { spec_tag, .. } => spec_tag.as_ref(),
        }
    }

    pub fn lesson_tag(&self) -> Option<&LessonId> {
        match self {
            ComposeEvent::PanelsLoaded { lesson_tag, .. }
            | ComposeEvent::PlatesRendered { lesson_tag, .. }
            | ComposeEvent::TitleComposited { lesson_tag, .. }
            | ComposeEvent::ComposeDone { lesson_tag, .. }
            | ComposeEvent::MetadataWritten { lesson_tag, .. } => lesson_tag.as_ref(),
        }
    }
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

impl CacheEvent {
    pub fn spec_tag(&self) -> Option<&SpecName> {
        match self {
            CacheEvent::HashComputed { spec_tag, .. }
            | CacheEvent::Hit { spec_tag, .. }
            | CacheEvent::Miss { spec_tag, .. }
            | CacheEvent::Promoted { spec_tag, .. }
            | CacheEvent::Evicted { spec_tag, .. } => spec_tag.as_ref(),
        }
    }

    pub fn lesson_tag(&self) -> Option<&LessonId> {
        match self {
            CacheEvent::HashComputed { lesson_tag, .. }
            | CacheEvent::Hit { lesson_tag, .. }
            | CacheEvent::Miss { lesson_tag, .. }
            | CacheEvent::Promoted { lesson_tag, .. }
            | CacheEvent::Evicted { lesson_tag, .. } => lesson_tag.as_ref(),
        }
    }
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

impl LintEvent {
    pub fn spec_tag(&self) -> Option<&SpecName> {
        match self {
            LintEvent::Started { spec_tag, .. }
            | LintEvent::RuleViolated { spec_tag, .. }
            | LintEvent::Passed { spec_tag, .. }
            | LintEvent::Failed { spec_tag, .. } => spec_tag.as_ref(),
        }
    }

    pub fn lesson_tag(&self) -> Option<&LessonId> {
        match self {
            LintEvent::Started { lesson_tag, .. }
            | LintEvent::RuleViolated { lesson_tag, .. }
            | LintEvent::Passed { lesson_tag, .. }
            | LintEvent::Failed { lesson_tag, .. } => lesson_tag.as_ref(),
        }
    }
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

impl LoraEvent {
    pub fn spec_tag(&self) -> Option<&SpecName> {
        match self {
            LoraEvent::TrainStarted { spec_tag, .. }
            | LoraEvent::StepProgress { spec_tag, .. }
            | LoraEvent::SanityRenderDone { spec_tag, .. }
            | LoraEvent::Trained { spec_tag, .. }
            | LoraEvent::Failed { spec_tag, .. } => spec_tag.as_ref(),
        }
    }

    pub fn lesson_tag(&self) -> Option<&LessonId> {
        match self {
            LoraEvent::TrainStarted { lesson_tag, .. }
            | LoraEvent::StepProgress { lesson_tag, .. }
            | LoraEvent::SanityRenderDone { lesson_tag, .. }
            | LoraEvent::Trained { lesson_tag, .. }
            | LoraEvent::Failed { lesson_tag, .. } => lesson_tag.as_ref(),
        }
    }
}

// ---------------------------------------------------------------------------
// From impls: lift a domain event into the top-level Event
// ---------------------------------------------------------------------------

impl From<RenderEvent> for Event {
    fn from(e: RenderEvent) -> Self {
        Event::Render(e)
    }
}
impl From<ComfyEvent> for Event {
    fn from(e: ComfyEvent) -> Self {
        Event::Comfy(e)
    }
}
impl From<QaEvent> for Event {
    fn from(e: QaEvent) -> Self {
        Event::Qa(e)
    }
}
impl From<ComposeEvent> for Event {
    fn from(e: ComposeEvent) -> Self {
        Event::Compose(e)
    }
}
impl From<CacheEvent> for Event {
    fn from(e: CacheEvent) -> Self {
        Event::Cache(e)
    }
}
impl From<LintEvent> for Event {
    fn from(e: LintEvent) -> Self {
        Event::Lint(e)
    }
}
impl From<LoraEvent> for Event {
    fn from(e: LoraEvent) -> Self {
        Event::Lora(e)
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Error returned when a [`Subscriber`] cannot yield the next event.
///
/// - `Closed` — every sender has been dropped; the stream is terminal.
/// - `Lagged(n)` — the subscriber was slow and the broadcast channel
///   dropped `n` events rather than stall the bus. Lag is surfaced rather
///   than silently swallowed; see the module docstring §Lag policy.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RecvError {
    /// The bus has been dropped; no more events will arrive.
    #[error("event bus closed")]
    Closed,
    /// The subscriber fell behind; `n` events were dropped.
    #[error("subscriber lagged — {0} events dropped")]
    Lagged(u64),
}

impl From<broadcast::error::RecvError> for RecvError {
    fn from(e: broadcast::error::RecvError) -> Self {
        match e {
            broadcast::error::RecvError::Closed => RecvError::Closed,
            broadcast::error::RecvError::Lagged(n) => RecvError::Lagged(n),
        }
    }
}

/// Error returned for the rare pathological emit case.
///
/// `tokio::sync::broadcast::Sender::send` does not fail when the channel is
/// "full" — it drops the oldest value for any slow receiver and keeps going.
/// It only errors when there are no receivers at all, which we treat as a
/// no-op rather than an error (emitting into the void is legal). This type
/// exists for future symmetry with fallible-emit transports (e.g. an SSE
/// bridge) and is currently unused by [`Bus::emit`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EmitError {
    /// Every subscriber had been dropped before the emit call.
    #[error("no active subscribers — event dropped")]
    NoSubscribers,
}

// ---------------------------------------------------------------------------
// Bus
// ---------------------------------------------------------------------------

/// Broadcast bus shared by every crate that emits or observes events.
///
/// Cloning the bus is cheap — the clone shares the same underlying
/// `Sender`. Subscribers get their own `Receiver`.
#[derive(Clone)]
pub struct Bus {
    sender: broadcast::Sender<Event>,
    capacity: usize,
}

impl Bus {
    /// Create a bus with the default channel capacity (1024). See the
    /// module docstring for the rationale.
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Create a bus with a custom channel capacity. Operators tuning for
    /// very bursty or very slow subscribers may need to override the
    /// default.
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(capacity > 0, "Bus capacity must be > 0");
        let (sender, _) = broadcast::channel(capacity);
        Self { sender, capacity }
    }

    /// Emit an event. Non-blocking. Accepts any domain event via the
    /// [`From<_> for Event`] impls, so callers write
    /// `bus.emit(RenderEvent::RunStarted { .. })`.
    ///
    /// If the broadcast ring buffer is full for some subscriber, that
    /// subscriber will observe a `RecvError::Lagged(n)` on its next
    /// `recv()` — the bus logs a `tracing::warn!` when we detect the
    /// ring sits at or beyond capacity to make the condition visible.
    pub fn emit(&self, event: impl Into<Event>) {
        let event = event.into();
        // `len()` reports the number of queued messages; when it reaches
        // capacity the next send will displace an unread message for the
        // slowest subscriber. We warn once here so operators know the
        // condition is live even if no subscriber has yet called recv().
        let queued = self.sender.len();
        if queued >= self.capacity {
            tracing::warn!(
                capacity = self.capacity,
                queued,
                "tt-events bus at capacity — a slow subscriber will see RecvError::Lagged"
            );
        }
        // `send` returns Err only when there are zero receivers, which is
        // an expected state (no observers attached) — not an error.
        let _ = self.sender.send(event);
    }

    /// Subscribe to every event emitted from now on. Returns a typed
    /// [`Subscriber`] wrapper; late subscribers do not see historical
    /// events (the broadcast channel is a ring, not a log).
    pub fn subscribe(&self) -> Subscriber {
        Subscriber {
            inner: self.sender.subscribe(),
        }
    }

    /// Number of live subscribers attached to this bus right now.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }

    /// The configured channel capacity. Surfaced for telemetry and tests.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Subscriber
// ---------------------------------------------------------------------------

/// Typed wrapper around `tokio::sync::broadcast::Receiver<Event>`.
///
/// Exposes an async `recv()` and conversions into `futures::Stream`s with
/// built-in spec/lesson/predicate filtering.
pub struct Subscriber {
    inner: broadcast::Receiver<Event>,
}

impl Subscriber {
    /// Await the next event from the bus.
    pub async fn recv(&mut self) -> Result<Event, RecvError> {
        self.inner.recv().await.map_err(RecvError::from)
    }

    /// Consume this subscriber and expose it as a `futures::Stream`.
    ///
    /// The stream terminates when the bus is dropped (`RecvError::Closed`)
    /// and yields whatever events arrive in between. `Lagged` events are
    /// **skipped at the stream layer** (a warning is logged) — a stream
    /// cannot yield an error, and we still want the loop to close.
    /// Callers that need strict lag accounting should use [`Self::recv`]
    /// directly.
    pub fn into_stream(self) -> impl Stream<Item = Event> + Send + Unpin {
        BusStream { inner: self.inner }
    }

    /// Stream of events whose `spec_tag` equals `Some(spec)`.
    pub fn filter_spec(self, spec: SpecName) -> impl Stream<Item = Event> + Send + Unpin {
        FilteredStream {
            inner: self.inner,
            predicate: Box::new(move |e: &Event| e.spec_tag() == Some(&spec)),
        }
    }

    /// Stream of events whose `lesson_tag` equals `Some(lesson)`.
    pub fn filter_lesson(self, lesson: LessonId) -> impl Stream<Item = Event> + Send + Unpin {
        FilteredStream {
            inner: self.inner,
            predicate: Box::new(move |e: &Event| e.lesson_tag() == Some(&lesson)),
        }
    }

    /// Stream of events matching an arbitrary predicate.
    pub fn filter_domain<F>(self, predicate: F) -> impl Stream<Item = Event> + Send + Unpin
    where
        F: Fn(&Event) -> bool + Send + 'static,
    {
        FilteredStream {
            inner: self.inner,
            predicate: Box::new(predicate),
        }
    }
}

// ---------------------------------------------------------------------------
// Stream adapters
// ---------------------------------------------------------------------------
//
// We implement two tiny state machines by hand rather than pulling in
// `tokio-stream` or `async-stream`: both would work, but the hand-rolled
// version keeps the dep surface minimal and is ~20 lines each. The key
// trick is `recv()` returning a boxed future we stash in the struct so
// poll_next can drive it to completion across wakeups.

struct BusStream {
    inner: broadcast::Receiver<Event>,
}

impl Stream for BusStream {
    type Item = Event;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        // We can't keep the receiver borrowed across await points in a
        // poll method; we build a transient future per poll instead.
        // `try_recv` first avoids allocating a future in the hot path.
        match this.inner.try_recv() {
            Ok(event) => Poll::Ready(Some(event)),
            Err(broadcast::error::TryRecvError::Closed) => Poll::Ready(None),
            Err(broadcast::error::TryRecvError::Lagged(n)) => {
                tracing::warn!(
                    dropped = n,
                    "tt-events stream dropped {} event(s) — subscriber lagged",
                    n
                );
                // Re-poll immediately; the receiver has advanced past the
                // lag and the next try_recv will either return a real
                // event or Empty.
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(broadcast::error::TryRecvError::Empty) => {
                // Nothing ready; register for wakeup via a real recv
                // future we drive once.
                poll_via_recv(&mut this.inner, cx)
            }
        }
    }
}

struct FilteredStream {
    inner: broadcast::Receiver<Event>,
    predicate: Box<dyn Fn(&Event) -> bool + Send>,
}

impl Stream for FilteredStream {
    type Item = Event;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            match this.inner.try_recv() {
                Ok(event) => {
                    if (this.predicate)(&event) {
                        return Poll::Ready(Some(event));
                    }
                    // Not a match — discard and keep draining.
                    continue;
                }
                Err(broadcast::error::TryRecvError::Closed) => return Poll::Ready(None),
                Err(broadcast::error::TryRecvError::Lagged(n)) => {
                    tracing::warn!(
                        dropped = n,
                        "tt-events filtered stream dropped {} event(s) — subscriber lagged",
                        n
                    );
                    continue;
                }
                Err(broadcast::error::TryRecvError::Empty) => {
                    return poll_via_recv(&mut this.inner, cx);
                }
            }
        }
    }
}

/// Helper: drive `broadcast::Receiver::recv` once to register a waker and
/// return `Poll::Pending`, or resolve synchronously if the future
/// happens to be ready immediately.
fn poll_via_recv(
    rx: &mut broadcast::Receiver<Event>,
    cx: &mut Context<'_>,
) -> Poll<Option<Event>> {
    // Construct a transient recv future each poll. `recv()` is cancel-safe
    // and the broadcast channel's wakeup list lives on the Receiver itself,
    // not on the future, so dropping the future after a Pending poll still
    // correctly registers us for wakeup when a new event arrives.
    // (Lifetime is local — no type alias; let inference bind the borrow.)
    let mut fut = Box::pin(rx.recv());
    match fut.as_mut().poll(cx) {
        Poll::Ready(Ok(event)) => Poll::Ready(Some(event)),
        Poll::Ready(Err(broadcast::error::RecvError::Closed)) => Poll::Ready(None),
        Poll::Ready(Err(broadcast::error::RecvError::Lagged(n))) => {
            tracing::warn!(
                dropped = n,
                "tt-events stream dropped {} event(s) — subscriber lagged",
                n
            );
            cx.waker().wake_by_ref();
            Poll::Pending
        }
        Poll::Pending => Poll::Pending,
    }
}

// Bring `Future` into scope for the boxed-future type alias above.
use std::future::Future;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use std::time::Duration;
    use tokio::time::timeout;
    use tt_core::{LessonId, SpecName, StripId};

    /// Short helper — every `recv` should resolve quickly in tests.
    const RECV_TIMEOUT: Duration = Duration::from_secs(2);

    fn run_started(spec: Option<&str>, lesson: Option<&str>) -> RenderEvent {
        RenderEvent::RunStarted {
            run_id: "test-run".into(),
            spec_tag: spec.map(|s| SpecName::new(s).unwrap()),
            lesson_tag: lesson.map(|l| LessonId::new(l).unwrap()),
        }
    }

    fn strip_discovered(strip: u16, spec: Option<&str>, lesson: Option<&str>) -> RenderEvent {
        RenderEvent::StripDiscovered {
            strip: StripId::new(strip).unwrap(),
            spec_tag: spec.map(|s| SpecName::new(s).unwrap()),
            lesson_tag: lesson.map(|l| LessonId::new(l).unwrap()),
        }
    }

    #[tokio::test]
    async fn single_subscriber_receives_event() {
        let bus = Bus::new();
        let mut sub = bus.subscribe();
        bus.emit(run_started(None, None));
        let evt = timeout(RECV_TIMEOUT, sub.recv())
            .await
            .expect("recv timed out")
            .expect("recv errored");
        match evt {
            Event::Render(RenderEvent::RunStarted { run_id, .. }) => {
                assert_eq!(run_id, "test-run");
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[tokio::test]
    async fn fan_out_to_multiple_subscribers() {
        let bus = Bus::new();
        let mut a = bus.subscribe();
        let mut b = bus.subscribe();
        let mut c = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 3);

        bus.emit(run_started(Some("orchestrator"), None));

        for sub in [&mut a, &mut b, &mut c] {
            let evt = timeout(RECV_TIMEOUT, sub.recv())
                .await
                .expect("recv timed out")
                .expect("recv errored");
            assert!(matches!(evt, Event::Render(RenderEvent::RunStarted { .. })));
        }
    }

    #[tokio::test]
    async fn late_subscriber_sees_only_future_events() {
        let bus = Bus::new();
        bus.emit(run_started(None, None)); // dropped — no subscribers yet
        let mut late = bus.subscribe();
        bus.emit(strip_discovered(7, None, None));

        let evt = timeout(RECV_TIMEOUT, late.recv())
            .await
            .expect("recv timed out")
            .expect("recv errored");
        match evt {
            Event::Render(RenderEvent::StripDiscovered { strip, .. }) => {
                assert_eq!(strip, StripId::new(7).unwrap());
            }
            other => panic!("late subscriber saw unexpected event: {other:?}"),
        }
    }

    #[tokio::test]
    async fn filter_spec_matches_only_tagged_events() {
        let bus = Bus::new();
        let sub = bus.subscribe();
        let mut stream = sub.filter_spec(SpecName::new("visual-qa-loop").unwrap());

        bus.emit(run_started(Some("orchestrator"), None));
        bus.emit(strip_discovered(1, Some("visual-qa-loop"), None));
        bus.emit(strip_discovered(2, None, None));
        bus.emit(strip_discovered(3, Some("visual-qa-loop"), None));
        drop(bus); // close the channel so the stream terminates.

        let collected: Vec<_> = timeout(RECV_TIMEOUT, stream.by_ref().collect::<Vec<_>>())
            .await
            .expect("stream drain timed out");
        assert_eq!(collected.len(), 2);
        for evt in &collected {
            assert_eq!(
                evt.spec_tag().map(|s| s.as_str()),
                Some("visual-qa-loop")
            );
        }
    }

    #[tokio::test]
    async fn filter_lesson_matches_only_tagged_events() {
        let bus = Bus::new();
        let sub = bus.subscribe();
        let mut stream = sub.filter_lesson(LessonId::new("S1-1300-loop-closes").unwrap());

        bus.emit(run_started(None, Some("S1-100-volatile-is-dangerous")));
        bus.emit(strip_discovered(4, None, Some("S1-1300-loop-closes")));
        bus.emit(strip_discovered(5, None, None));
        bus.emit(strip_discovered(6, None, Some("S1-1300-loop-closes")));
        drop(bus);

        let collected: Vec<_> = timeout(RECV_TIMEOUT, stream.by_ref().collect::<Vec<_>>())
            .await
            .expect("stream drain timed out");
        assert_eq!(collected.len(), 2);
        for evt in &collected {
            assert_eq!(evt.lesson_tag().map(|l| l.as_str()), Some("S1-1300-loop-closes"));
        }
    }

    #[tokio::test]
    async fn filter_domain_arbitrary_predicate() {
        let bus = Bus::new();
        let sub = bus.subscribe();
        let mut stream =
            sub.filter_domain(|e| matches!(e, Event::Render(RenderEvent::StripDiscovered { .. })));

        bus.emit(run_started(None, None));
        bus.emit(strip_discovered(1, None, None));
        bus.emit(strip_discovered(2, None, None));
        drop(bus);

        let collected: Vec<_> = timeout(RECV_TIMEOUT, stream.by_ref().collect::<Vec<_>>())
            .await
            .expect("stream drain timed out");
        assert_eq!(collected.len(), 2);
    }

    #[tokio::test]
    async fn into_stream_terminates_when_bus_dropped() {
        let bus = Bus::new();
        let sub = bus.subscribe();
        let mut stream = sub.into_stream();

        bus.emit(run_started(None, None));
        drop(bus);

        let collected: Vec<_> = timeout(RECV_TIMEOUT, stream.by_ref().collect::<Vec<_>>())
            .await
            .expect("stream drain timed out");
        assert_eq!(collected.len(), 1);
    }

    #[tokio::test]
    async fn qa_check_result_round_trip() {
        let bus = Bus::new();
        let mut sub = bus.subscribe();
        let check = QaEvent::CheckResult {
            strip: StripId::new(12).unwrap(),
            panel: 2,
            iteration: 3,
            check_id: "codex-bound-glyph".into(),
            pass: false,
            confidence: 0.42,
            spec_tag: Some(SpecName::new("visual-qa-loop").unwrap()),
            lesson_tag: Some(LessonId::new("S1-1300-loop-closes").unwrap()),
        };
        bus.emit(check);

        let evt = timeout(RECV_TIMEOUT, sub.recv())
            .await
            .expect("recv timed out")
            .expect("recv errored");

        match evt {
            Event::Qa(QaEvent::CheckResult {
                strip,
                panel,
                iteration,
                check_id,
                pass,
                confidence,
                spec_tag,
                lesson_tag,
            }) => {
                assert_eq!(strip, StripId::new(12).unwrap());
                assert_eq!(panel, 2);
                assert_eq!(iteration, 3);
                assert_eq!(check_id, "codex-bound-glyph");
                assert!(!pass);
                assert!((confidence - 0.42).abs() < 1e-6);
                assert_eq!(spec_tag.unwrap().as_str(), "visual-qa-loop");
                assert_eq!(lesson_tag.unwrap().as_str(), "S1-1300-loop-closes");
            }
            other => panic!("expected QaEvent::CheckResult, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn subscriber_count_tracks_drops() {
        let bus = Bus::new();
        let a = bus.subscribe();
        let b = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);
        drop(a);
        assert_eq!(bus.subscriber_count(), 1);
        drop(b);
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn default_capacity_is_1024() {
        let bus = Bus::new();
        assert_eq!(bus.capacity(), 1024);
        let small = Bus::with_capacity(8);
        assert_eq!(small.capacity(), 8);
    }

    #[tokio::test]
    async fn recv_error_closed_after_bus_drop() {
        let bus = Bus::new();
        let mut sub = bus.subscribe();
        drop(bus);
        let err = timeout(RECV_TIMEOUT, sub.recv())
            .await
            .expect("recv timed out")
            .unwrap_err();
        assert_eq!(err, RecvError::Closed);
    }

    #[tokio::test]
    async fn lagged_error_surfaces_via_recv() {
        let bus = Bus::with_capacity(2);
        let mut sub = bus.subscribe();
        // Overflow: emit 5 events into a capacity-2 channel; the first 3
        // are dropped for this (slow) subscriber.
        for i in 1..=5 {
            bus.emit(strip_discovered(i, None, None));
        }
        let err = timeout(RECV_TIMEOUT, sub.recv())
            .await
            .expect("recv timed out")
            .unwrap_err();
        match err {
            RecvError::Lagged(n) => assert!(n > 0, "Lagged should report > 0, got {n}"),
            other => panic!("expected Lagged, got {other:?}"),
        }
    }
}
