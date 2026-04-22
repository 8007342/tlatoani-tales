//! Tlatoāni Tales — telemetry.
//!
//! Initialises the `tracing` subscriber, wires the JSONL sink at
//! `output/telemetry/<strip>.jsonl`, and tags every event with `spec` and
//! `lesson` fields. The same event stream feeds Calmecac's convergence
//! dashboards — the viewer observes the orchestrator through the telemetry
//! this crate emits.
//!
//! Governing spec: `openspec/specs/orchestrator/spec.md` §Observability of
//! the orchestrator itself.
//!
// @trace spec:orchestrator, spec:calmecac
// @Lesson S1-900
// @Lesson S1-1500

use tt_core::TtError;

/// Guard returned from `init`. Drop it (via `.stop()` or at program exit) to
/// flush buffered JSONL events to disk.
pub struct TelemetryGuard {
    _priv: (),
}

impl TelemetryGuard {
    /// Flush any buffered telemetry events. Idempotent.
    pub fn stop(self) {
        // Scaffold: no-op. Real impl flushes the JSONL sink's buffered writer.
    }
}

/// Initialise the process-wide tracing subscriber + JSONL sink. Call once
/// from `main()`, hold the returned guard for the program's lifetime.
pub fn init() -> Result<TelemetryGuard, TtError> {
    // Scaffold: install a minimal subscriber that writes to stderr so
    // `tracing::info!` works from call sites during development. Real impl
    // adds the JSONL sink and the spec/lesson field enrichment.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();
    Ok(TelemetryGuard { _priv: () })
}
