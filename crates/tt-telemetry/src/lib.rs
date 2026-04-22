//! Tlatoāni Tales — telemetry.
//!
//! This crate wires three related pieces of observability:
//!
//! 1. **Tracing init** — a single `init()` entry point that installs a
//!    `tracing_subscriber::fmt()` JSON layer over `EnvFilter` (`RUST_LOG`).
//!    Every span/event is emitted as structured JSON to stderr, carrying the
//!    `spec` and `lesson` fields each accountability-tagged call site adds.
//! 2. **JSONL sink per strip** — [`StripTelemetrySink`] writes one JSON line
//!    per [`Event`] to `output/telemetry/strip-NN.jsonl`. The on-disk shape is
//!    `{ "ts": "<iso8601>", "event": <Event> }`. This is the grep-first
//!    debugging surface and the input to Calmecac's convergence dashboards.
//! 3. **Convergence metrics** — [`ConvergenceMetric`] + [`append_metric`]
//!    append to a single global `output/telemetry/metrics.jsonl`. The
//!    Calmecac indexer reads this file to populate the *boring dashboards*
//!    (the non-visual rule convergence tab; `calmecac/spec.md` §Boring
//!    dashboards).
//!
//! The bus-to-sink bridge ([`run_sink_from_bus`]) subscribes to the typed
//! event bus and dispatches events to the correct strip file — for events
//! that carry a strip field it writes to `strip-NN.jsonl`; for events without
//! a natural strip (run lifecycle, lint, lora, cache, comfy progress by
//! `prompt_id`) it writes to `run.jsonl`. This is the loop that closes
//! (`@Lesson S1-1300`): telemetry from iteration `i` re-enters as readable
//! artefacts for iteration `i+1`.
//!
//! Governing specs: `openspec/specs/orchestrator/spec.md` §Observability,
//! `openspec/specs/calmecac/spec.md` §Boring dashboards,
//! `openspec/specs/visual-qa-loop/spec.md` §Drift score aggregation.
//!
// @trace spec:orchestrator, spec:calmecac, spec:visual-qa-loop
// @Lesson S1-900
// @Lesson S1-1000
// @Lesson S1-1300
// @Lesson S1-1500

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tt_core::{LessonId, SpecName, StripId, TtError};
use tt_events::{ComfyEvent, ComposeEvent, Event, QaEvent, RenderEvent};

// ---------------------------------------------------------------------------
// Tracing init
// ---------------------------------------------------------------------------

/// Guard returned from [`init`]. Holding it keeps the subscriber alive for
/// the lifetime of the program. Dropping it is a no-op (the global
/// subscriber stays installed for the process), but callers should treat it
/// as RAII: keep it until shutdown and then let it fall out of scope.
///
/// The guard exists chiefly so that the signature of `init` reads as
/// "install + hand back a resource to hold" — matching the idiom used by
/// `tracing_appender::non_blocking`.
pub struct TelemetryGuard {
    _priv: (),
}

/// Initialise the process-wide tracing subscriber. Call once from `main()`.
///
/// The subscriber is configured with:
/// - `EnvFilter` honouring `RUST_LOG` (default `info`).
/// - `fmt().json()` layer writing structured JSON to **stderr** — every
///   `tracing::info!(spec = "orchestrator", lesson = "S1-1300", ...)` call
///   site renders as one JSON object per line, with the `spec` / `lesson`
///   fields as first-class keys so downstream tools can filter on them.
///
/// `project_dir` is accepted for future use (e.g. a non-blocking file layer
/// rooted at `<project_dir>/output/telemetry/`); right now the JSONL surface
/// lives on [`StripTelemetrySink`] rather than inside the tracing layer.
/// The argument exists so the public signature matches the orchestrator
/// spec and does not churn later.
///
/// Calling `init` a second time is a no-op — `try_init()` will quietly fail
/// and the existing subscriber stays active.
// @trace spec:orchestrator
// @Lesson S1-900
pub fn init(project_dir: &Path) -> Result<TelemetryGuard, TtError> {
    // Keep the argument — it's part of the public contract. We may later
    // mount a file-based layer at <project_dir>/output/telemetry/trace.log.
    let _ = project_dir;
    init_subscriber()
}

/// Zero-arg variant retained for backwards-compatible call sites in the
/// workspace (notably `tt-render`'s `main`). Equivalent to
/// [`init`]`(std::path::Path::new("."))`.
pub fn init_default() -> Result<TelemetryGuard, TtError> {
    init_subscriber()
}

fn init_subscriber() -> Result<TelemetryGuard, TtError> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(std::io::stderr)
        .with_current_span(true)
        .with_span_list(false);

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init();

    Ok(TelemetryGuard { _priv: () })
}

// ---------------------------------------------------------------------------
// Strip JSONL sink
// ---------------------------------------------------------------------------

/// Writes one JSON line per [`Event`] to `output/telemetry/strip-NN.jsonl`.
///
/// Each line is `{ "ts": "<iso8601>", "event": <Event> }` — a valid JSON
/// object terminated by `\n`. The file is opened in append mode so multiple
/// runs on the same strip accrete history; Calmecac's indexer reads the
/// whole file to plot convergence series.
///
/// # Flush cadence
///
/// Writes are `append + flush` per event. This is deliberately simple: the
/// orchestrator's event rate is on the order of tens per second worst-case
/// (full 15-strip run), nowhere near the cost where batching matters, and
/// the grep-first debugging workflow benefits from seeing events land on
/// disk in real time. If flushing ever becomes a bottleneck, revisit.
// @trace spec:orchestrator, spec:calmecac
// @Lesson S1-900
pub struct StripTelemetrySink {
    file: File,
    #[allow(dead_code)]
    strip: StripKey,
    #[allow(dead_code)]
    path: PathBuf,
}

/// Sink file key — either a specific strip or the run-level sink.
///
/// Strip-less events (run lifecycle, cache, lint, lora, and any comfy
/// events that reference a `prompt_id` without a strip) land in
/// `run.jsonl`. See [`strip_of`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StripKey {
    Strip(StripId),
    Run,
}

impl StripKey {
    fn file_name(&self) -> String {
        match self {
            StripKey::Strip(s) => format!("strip-{}.jsonl", s),
            StripKey::Run => "run.jsonl".to_string(),
        }
    }
}

impl StripTelemetrySink {
    /// Open (or create) the strip's JSONL file under
    /// `<output_dir>/telemetry/strip-NN.jsonl`. Creates the telemetry
    /// subdirectory if absent.
    pub async fn open(output_dir: &Path, strip: StripId) -> Result<Self, TtError> {
        Self::open_keyed(output_dir, StripKey::Strip(strip)).await
    }

    /// Open the run-level sink at `<output_dir>/telemetry/run.jsonl`. Used
    /// by the bus bridge for events that do not carry a strip.
    pub async fn open_run(output_dir: &Path) -> Result<Self, TtError> {
        Self::open_keyed(output_dir, StripKey::Run).await
    }

    async fn open_keyed(output_dir: &Path, key: StripKey) -> Result<Self, TtError> {
        let dir = output_dir.join("telemetry");
        tokio::fs::create_dir_all(&dir).await?;
        let path = dir.join(key.file_name());
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await?;
        Ok(Self {
            file,
            strip: key,
            path,
        })
    }

    /// Append one JSON line for `event`. Flushes before returning so the
    /// line is visible to `tail -f` and grep.
    pub async fn write_event(&mut self, event: &Event) -> Result<(), TtError> {
        let line = SinkLine {
            ts: iso8601_now(),
            event,
        };
        let mut buf = serde_json::to_vec(&line)
            .map_err(|e| TtError::Parse(format!("telemetry serialisation failed: {e}")))?;
        buf.push(b'\n');
        self.file.write_all(&buf).await?;
        self.file.flush().await?;
        Ok(())
    }

    /// Flush the underlying file handle. Usually unnecessary because
    /// [`write_event`](Self::write_event) flushes, but exposed for the
    /// bus bridge shutdown path.
    pub async fn flush(&mut self) -> Result<(), TtError> {
        self.file.flush().await?;
        Ok(())
    }
}

#[derive(Serialize)]
struct SinkLine<'a> {
    ts: String,
    event: &'a Event,
}

// ---------------------------------------------------------------------------
// Convergence metrics (Calmecac's "boring dashboards" input)
// ---------------------------------------------------------------------------

/// A single metric data point appended to `output/telemetry/metrics.jsonl`.
///
/// The Calmecac indexer (`tt-calmecac-indexer`) reads this file at build
/// time and materialises per-rule time series into the convergence tab.
/// See `calmecac/spec.md` §Boring dashboards and §Concept index generation.
///
/// This is the literal data stream demanded by `@Lesson S1-1000` —
/// *dashboards-must-add-observability*. The dashboard is boring by design;
/// its audience is the feedback loop, not the reader.
// @trace spec:calmecac, spec:orchestrator
// @Lesson S1-1000
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceMetric {
    /// Metric name, e.g. `"drift_score.mean"`, `"cache.hit_ratio"`,
    /// `"rerolls.count"`. Convention: dotted-path namespaces so the
    /// indexer can group by prefix.
    pub name: String,

    /// Numeric value. f32 — the whole project's error budget is in this
    /// range (drift scores, ratios, counts under a few thousand).
    pub value: f32,

    /// ISO-8601 timestamp. Filled by callers via [`iso8601_now`] or a
    /// fixed timestamp for deterministic tests.
    pub ts: String,

    /// Optional strip this metric is about. A metric may be cross-strip
    /// (e.g. a per-run hit ratio) in which case this is `None`.
    pub strip: Option<StripId>,

    /// Optional governing spec tag.
    pub spec_tag: Option<SpecName>,

    /// Optional governing lesson tag.
    pub lesson_tag: Option<LessonId>,
}

/// Append a single metric to `metrics.jsonl`, creating the file (and the
/// `telemetry/` subdirectory) if absent.
///
/// The path is the full file path (e.g.
/// `<output>/telemetry/metrics.jsonl`) — callers choose the location so
/// tests can point at a tempdir. One JSON object per line; flushed before
/// return.
// @trace spec:calmecac
// @Lesson S1-1000
pub async fn append_metric(
    metrics_path: &Path,
    metric: &ConvergenceMetric,
) -> Result<(), TtError> {
    if let Some(parent) = metrics_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(metrics_path)
        .await?;
    let mut buf = serde_json::to_vec(metric)
        .map_err(|e| TtError::Parse(format!("metric serialisation failed: {e}")))?;
    buf.push(b'\n');
    file.write_all(&buf).await?;
    file.flush().await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Drift-score aggregator
// ---------------------------------------------------------------------------

/// Aggregate of QA drift scores over a slice of events — the shape consumed
/// by the visual-qa-loop's convergence dashboard.
///
/// Feeds the mean/max/count triple per `openspec/specs/visual-qa-loop/spec.md`
/// §Drift score aggregation. A panel is stable when `mean < 0.05` across
/// its most recent iterations; the `max` traps a single worst-case check.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DriftAggregate {
    pub mean: f32,
    pub max: f32,
    pub count: u32,
}

/// Compute `(mean, max, count)` over the `Qa::CheckResult` events in
/// `events`, where each `CheckResult`'s contribution is `1 - confidence`
/// when `pass == false`, else `0.0`.
///
/// Returns `None` when the slice contains no CheckResult events — the
/// caller should treat that as "no signal yet" rather than "all clean".
// @trace spec:visual-qa-loop
// @Lesson S1-1000
pub fn aggregate_drift_scores(events: &[Event]) -> Option<DriftAggregate> {
    let mut sum = 0.0f32;
    let mut max = 0.0f32;
    let mut count = 0u32;
    for e in events {
        if let Event::Qa(QaEvent::CheckResult {
            pass, confidence, ..
        }) = e
        {
            let contribution = if *pass { 0.0 } else { 1.0 - confidence };
            sum += contribution;
            if contribution > max {
                max = contribution;
            }
            count += 1;
        }
    }
    if count == 0 {
        None
    } else {
        Some(DriftAggregate {
            mean: sum / count as f32,
            max,
            count,
        })
    }
}

// ---------------------------------------------------------------------------
// Bus-to-sink bridge
// ---------------------------------------------------------------------------

/// Strip-of dispatch for an [`Event`].
///
/// Returns the event's natural strip when the variant carries one. For
/// variants with no strip field — run lifecycle, cache, lint, lora, and
/// the prompt_id-keyed comfy variants (Progress / Rendered / Failed /
/// Timeout) — returns `None`, signalling the bridge to route to the
/// run-level file.
///
/// # Judgment call
///
/// `ComfyEvent::Submitted` carries a strip, but `Progress` / `Rendered` /
/// `Failed` / `Timeout` reference the submission only by `prompt_id`. We
/// could correlate prompt_id → strip inside the bridge (stash a map at
/// Submitted time) but that would leak state into the bridge that the bus
/// already canonicalises elsewhere (the event producer knows the strip
/// and is free to include it). Keeping the bridge stateless — prompt_id
/// events land in `run.jsonl` — preserves the "no hidden state mutations"
/// invariant from `orchestrator/spec.md`.
fn strip_of(event: &Event) -> Option<StripId> {
    match event {
        Event::Render(e) => match e {
            RenderEvent::StripDiscovered { strip, .. }
            | RenderEvent::PanelHashComputed { strip, .. }
            | RenderEvent::CacheHit { strip, .. }
            | RenderEvent::CacheMiss { strip, .. } => Some(*strip),
            RenderEvent::RunStarted { .. }
            | RenderEvent::SpecLoaded { .. }
            | RenderEvent::RunComplete { .. }
            | RenderEvent::RunFailed { .. } => None,
        },
        Event::Comfy(e) => match e {
            ComfyEvent::Submitted { strip, .. } => Some(*strip),
            ComfyEvent::Progress { .. }
            | ComfyEvent::Rendered { .. }
            | ComfyEvent::Failed { .. }
            | ComfyEvent::Timeout { .. } => None,
        },
        Event::Qa(e) => match e {
            QaEvent::Submitted { strip, .. }
            | QaEvent::CheckResult { strip, .. }
            | QaEvent::Verdict { strip, .. }
            | QaEvent::RerollScheduled { strip, .. } => Some(*strip),
        },
        Event::Compose(e) => match e {
            ComposeEvent::PanelsLoaded { strip, .. }
            | ComposeEvent::PlatesRendered { strip, .. }
            | ComposeEvent::TitleComposited { strip, .. }
            | ComposeEvent::ComposeDone { strip, .. }
            | ComposeEvent::MetadataWritten { strip, .. } => Some(*strip),
        },
        Event::Cache(_) | Event::Lint(_) | Event::Lora(_) => None,
    }
}

/// Drain the bus into per-strip JSONL files until the bus is closed.
///
/// For each event:
/// - Compute the owning strip via [`strip_of`].
/// - Lazily open (and memoise) the matching [`StripTelemetrySink`].
/// - Write the event + flush.
///
/// Returns when the bus is dropped (`RecvError::Closed`). Lagged events are
/// treated the way the bus does: logged and skipped. An I/O failure on a
/// sink surfaces as a `TtError` and terminates the bridge so a broken
/// telemetry disk never silently swallows events.
///
/// The bridge owns its own subscriber — callers pass the bus by reference
/// and the bridge calls `bus.subscribe()` internally, guaranteeing the
/// subscriber is created before the first `recv` and that no event from
/// before the bridge started is retroactively required.
// @trace spec:orchestrator, spec:calmecac
// @Lesson S1-1300
pub async fn run_sink_from_bus(
    bus: &tt_events::Bus,
    output_dir: &Path,
) -> Result<(), TtError> {
    let subscriber = bus.subscribe();
    run_sink_from_subscriber(subscriber, output_dir).await
}

/// Like [`run_sink_from_bus`] but takes an already-created [`Subscriber`].
///
/// Preferred when the caller spawns the bridge on a background task and
/// does not want to hand the task a second [`Bus`] handle (which would
/// keep the bus's sender alive and prevent `recv` from ever seeing
/// `Closed`). Subscribe in the caller, hand the subscriber into the task
/// — that way the bus closes as soon as the caller's [`Bus`] is dropped.
pub async fn run_sink_from_subscriber(
    mut subscriber: tt_events::Subscriber,
    output_dir: &Path,
) -> Result<(), TtError> {
    let mut sinks: HashMap<StripKey, StripTelemetrySink> = HashMap::new();

    loop {
        match subscriber.recv().await {
            Ok(event) => {
                let key = match strip_of(&event) {
                    Some(s) => StripKey::Strip(s),
                    None => StripKey::Run,
                };
                let sink = match sinks.get_mut(&key) {
                    Some(s) => s,
                    None => {
                        let fresh = StripTelemetrySink::open_keyed(output_dir, key).await?;
                        sinks.insert(key, fresh);
                        sinks
                            .get_mut(&key)
                            .expect("just inserted sink must be present")
                    }
                };
                sink.write_event(&event).await?;
            }
            Err(tt_events::RecvError::Closed) => break,
            Err(tt_events::RecvError::Lagged(n)) => {
                tracing::warn!(
                    dropped = n,
                    "tt-telemetry bus-to-sink bridge dropped {n} events — subscriber lagged"
                );
                continue;
            }
        }
    }

    // Best-effort final flush on shutdown.
    for sink in sinks.values_mut() {
        let _ = sink.flush().await;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

/// Current wall-clock time as ISO-8601 (RFC 3339). Public so callers
/// constructing [`ConvergenceMetric`] outside this crate can use the same
/// formatter and get byte-identical timestamps.
pub fn iso8601_now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::io::AsyncReadExt;
    use tokio::time::timeout;
    use tt_core::{PanelHash, SpecName, StripId};
    use tt_events::{Bus, QaVerdict, RenderEvent};

    const IO_TIMEOUT: Duration = Duration::from_secs(5);

    // -- init ------------------------------------------------------------

    #[test]
    fn telemetry_guard_round_trip_is_clean() {
        let tmp = TempDir::new().unwrap();
        let guard = init(tmp.path()).expect("init");
        // Dropping the guard is explicitly supported.
        drop(guard);
        // Second init is a no-op (already installed).
        let guard2 = init(tmp.path()).expect("second init");
        drop(guard2);
    }

    // -- StripTelemetrySink ---------------------------------------------

    fn render_strip_discovered(n: u16) -> Event {
        Event::Render(RenderEvent::StripDiscovered {
            strip: StripId::new(n).unwrap(),
            spec_tag: Some(SpecName::new("orchestrator").unwrap()),
            lesson_tag: None,
        })
    }

    #[tokio::test]
    async fn strip_sink_writes_three_lines() {
        let tmp = TempDir::new().unwrap();
        let strip = StripId::new(7).unwrap();
        let mut sink = StripTelemetrySink::open(tmp.path(), strip).await.unwrap();

        for n in [7, 7, 7] {
            sink.write_event(&render_strip_discovered(n)).await.unwrap();
        }
        sink.flush().await.unwrap();

        let path = tmp.path().join("telemetry").join("strip-07.jsonl");
        assert!(path.exists(), "expected {}", path.display());

        let mut contents = String::new();
        File::open(&path)
            .await
            .unwrap()
            .read_to_string(&mut contents)
            .await
            .unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 3, "got {} lines", lines.len());
        for line in lines {
            let v: serde_json::Value = serde_json::from_str(line).expect("valid json per line");
            assert!(v.get("ts").and_then(|t| t.as_str()).is_some(), "missing ts");
            assert!(v.get("event").is_some(), "missing event");
        }
    }

    #[tokio::test]
    async fn strip_sink_appends_across_reopens() {
        let tmp = TempDir::new().unwrap();
        let strip = StripId::new(3).unwrap();

        let mut sink = StripTelemetrySink::open(tmp.path(), strip).await.unwrap();
        sink.write_event(&render_strip_discovered(3)).await.unwrap();
        drop(sink);

        let mut sink2 = StripTelemetrySink::open(tmp.path(), strip).await.unwrap();
        sink2.write_event(&render_strip_discovered(3)).await.unwrap();
        drop(sink2);

        let path = tmp.path().join("telemetry").join("strip-03.jsonl");
        let contents = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(contents.lines().count(), 2);
    }

    // -- aggregate_drift_scores -----------------------------------------

    fn check_result(pass: bool, confidence: f32) -> Event {
        Event::Qa(QaEvent::CheckResult {
            strip: StripId::new(1).unwrap(),
            panel: 1,
            iteration: 1,
            check_id: "x".into(),
            pass,
            confidence,
            spec_tag: None,
            lesson_tag: None,
        })
    }

    #[test]
    fn aggregate_drift_scores_none_when_no_check_events() {
        let events = vec![render_strip_discovered(1)];
        assert!(aggregate_drift_scores(&events).is_none());
    }

    #[test]
    fn aggregate_drift_scores_mean_max_count() {
        // Two failing (contributions 0.8 and 0.4), one passing (contribution 0.0).
        // mean = (0.8 + 0.4 + 0.0) / 3 = 0.4, max = 0.8, count = 3.
        let events = vec![
            check_result(false, 0.2), // 1 - 0.2 = 0.8
            check_result(false, 0.6), // 1 - 0.6 = 0.4
            check_result(true, 0.9),  // 0.0 because pass
            render_strip_discovered(5), // not a CheckResult — ignored
        ];
        let agg = aggregate_drift_scores(&events).unwrap();
        assert_eq!(agg.count, 3);
        assert!((agg.max - 0.8).abs() < 1e-6, "max was {}", agg.max);
        assert!((agg.mean - 0.4).abs() < 1e-6, "mean was {}", agg.mean);
    }

    #[test]
    fn aggregate_drift_scores_all_pass_gives_zero_mean() {
        let events = vec![check_result(true, 0.99), check_result(true, 0.88)];
        let agg = aggregate_drift_scores(&events).unwrap();
        assert_eq!(agg.count, 2);
        assert_eq!(agg.mean, 0.0);
        assert_eq!(agg.max, 0.0);
    }

    // -- append_metric --------------------------------------------------

    fn sample_metric(name: &str, v: f32) -> ConvergenceMetric {
        ConvergenceMetric {
            name: name.into(),
            value: v,
            ts: iso8601_now(),
            strip: Some(StripId::new(1).unwrap()),
            spec_tag: Some(SpecName::new("visual-qa-loop").unwrap()),
            lesson_tag: None,
        }
    }

    #[tokio::test]
    async fn append_metric_creates_and_appends() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("telemetry").join("metrics.jsonl");
        assert!(!path.exists());

        append_metric(&path, &sample_metric("drift_score.mean", 0.12))
            .await
            .unwrap();
        assert!(path.exists());

        append_metric(&path, &sample_metric("drift_score.max", 0.42))
            .await
            .unwrap();
        append_metric(&path, &sample_metric("cache.hit_ratio", 0.87))
            .await
            .unwrap();

        let contents = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(contents.lines().count(), 3);
        for line in contents.lines() {
            let v: ConvergenceMetric =
                serde_json::from_str(line).expect("round-trips through serde");
            assert!(!v.name.is_empty());
            assert!(!v.ts.is_empty());
        }
    }

    // -- bus-to-sink bridge ---------------------------------------------

    #[tokio::test]
    async fn bus_bridge_routes_events_to_correct_strip_file() {
        let tmp = TempDir::new().unwrap();
        let output = tmp.path().to_path_buf();
        let bus = Bus::new();

        // Subscribe FROM THE TEST TASK so the spawned bridge does not hold
        // a bus clone — that way `drop(bus)` below actually closes the
        // channel and the bridge sees `RecvError::Closed`.
        let subscriber = bus.subscribe();
        let out_for_task = output.clone();
        let bridge = tokio::spawn(async move {
            run_sink_from_subscriber(subscriber, &out_for_task).await
        });
        assert_eq!(bus.subscriber_count(), 1);

        // Strip 2 gets two events; strip 5 gets one; one run-level event
        // (RunStarted has no strip — it goes to run.jsonl).
        bus.emit(RenderEvent::RunStarted {
            run_id: "r1".into(),
            spec_tag: Some(SpecName::new("orchestrator").unwrap()),
            lesson_tag: None,
        });
        bus.emit(RenderEvent::StripDiscovered {
            strip: StripId::new(2).unwrap(),
            spec_tag: None,
            lesson_tag: None,
        });
        bus.emit(RenderEvent::PanelHashComputed {
            strip: StripId::new(2).unwrap(),
            panel: 1,
            panel_hash: PanelHash::from_bytes([0u8; 32]),
            spec_tag: None,
            lesson_tag: None,
        });
        bus.emit(QaEvent::Verdict {
            strip: StripId::new(5).unwrap(),
            panel: 2,
            iteration: 1,
            drift_score: 0.01,
            verdict: QaVerdict::Stable,
            spec_tag: Some(SpecName::new("visual-qa-loop").unwrap()),
            lesson_tag: None,
        });

        // Close the bus so the bridge terminates.
        drop(bus);
        timeout(IO_TIMEOUT, bridge)
            .await
            .expect("bridge did not complete")
            .expect("join")
            .expect("bridge I/O");

        let strip02 = output.join("telemetry").join("strip-02.jsonl");
        let strip05 = output.join("telemetry").join("strip-05.jsonl");
        let run = output.join("telemetry").join("run.jsonl");

        let s02 = tokio::fs::read_to_string(&strip02).await.unwrap();
        let s05 = tokio::fs::read_to_string(&strip05).await.unwrap();
        let r = tokio::fs::read_to_string(&run).await.unwrap();

        assert_eq!(s02.lines().count(), 2, "strip-02.jsonl should have 2 events");
        assert_eq!(s05.lines().count(), 1, "strip-05.jsonl should have 1 event");
        assert_eq!(r.lines().count(), 1, "run.jsonl should have 1 event");

        // Each line must be valid JSON carrying the right shape.
        for line in s02.lines().chain(s05.lines()).chain(r.lines()) {
            let v: serde_json::Value = serde_json::from_str(line).expect("valid json");
            assert!(v["ts"].as_str().is_some());
            assert!(v["event"].is_object());
        }
    }

    // -- strip_of dispatch coverage -------------------------------------

    #[test]
    fn strip_of_returns_none_for_run_lifecycle_events() {
        let e = Event::Render(RenderEvent::RunStarted {
            run_id: "x".into(),
            spec_tag: None,
            lesson_tag: None,
        });
        assert!(strip_of(&e).is_none());
    }

    #[test]
    fn strip_of_returns_some_for_stripped_events() {
        let e = render_strip_discovered(9);
        assert_eq!(strip_of(&e), Some(StripId::new(9).unwrap()));
    }

    #[test]
    fn iso8601_now_has_expected_shape() {
        let ts = iso8601_now();
        // Cheap structural check — we don't pull the `parsing` feature for a
        // full RFC3339 round trip, but the formatted output always contains
        // `T` between date and time and ends with `Z` or a numeric offset.
        assert!(ts.contains('T'), "missing T separator: {ts}");
        assert!(ts.len() >= 20, "ts too short: {ts}");
        assert!(
            ts.ends_with('Z') || ts.contains('+') || ts.contains('-'),
            "ts missing offset: {ts}"
        );
    }
}
