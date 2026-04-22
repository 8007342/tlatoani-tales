//! Tlatoāni Tales — async HTTP client for ComfyUI.
//!
//! The trusted Rust zone never imports Python. It reaches the ComfyUI
//! runtime — which lives in a hardened `--network=none` container — over
//! the forwarded localhost port declared by the launcher. One workflow
//! submit becomes a `prompt_id`; [`ComfyClient::watch`] turns that id into
//! a stream of typed [`ComfyStatus`] updates by polling `/history/<id>`
//! on a fixed cadence (see `WATCH_POLL_INTERVAL`). Rendered PNG bytes are
//! fetched with [`ComfyClient::fetch_output`].
//!
//! This crate is the seam where the render loop closes across the trust
//! boundary: telemetry from iteration `i` (a rendered panel's drift
//! report) flows back in as the prompt addendum for iteration `i+1`. The
//! ability to close that loop presupposes the orchestrator can talk to
//! ComfyUI without importing Python — that's what this crate is for.
//!
//! Governing spec: `openspec/specs/orchestrator/spec.md`,
//! `openspec/specs/isolation/spec.md`.
//!
// @trace spec:orchestrator, spec:isolation
// @Lesson S1-1300

use std::collections::BTreeMap;
use std::time::Duration;

use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tt_core::TtError;
use url::Url;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Cadence of [`ComfyClient::watch`]'s `/history/<id>` poll.
///
/// Per `orchestrator/spec.md` §Invariants: *"Never poll in tight loops."*
/// 500ms is slow enough to avoid hammering the untrusted container, fast
/// enough that the outer progress-bar UI feels live for a 20-step workflow
/// (~10s total).
pub const WATCH_POLL_INTERVAL: Duration = Duration::from_millis(500);

// ---------------------------------------------------------------------------
// Newtypes
// ---------------------------------------------------------------------------

/// Opaque ComfyUI prompt id. Returned from [`ComfyClient::submit`],
/// consumed by [`ComfyClient::history`] and [`ComfyClient::watch`].
// @trace spec:orchestrator
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PromptId(pub String);

impl PromptId {
    /// The inner string form — what ComfyUI stamps the prompt with.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PromptId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A ComfyUI workflow — a pass-through wrapper around the node-graph
/// `serde_json::Value`.
///
/// We deliberately do not model ComfyUI's node graph here. ComfyUI
/// workflows are deep, heterogeneous, and extended by every custom node
/// in the container; partial modeling would be a maintenance tax with
/// no safety payoff at the HTTP-client layer. The tt-specs / tt-render
/// side generates the JSON; this crate ferries it across the boundary.
///
/// See `orchestrator/spec.md` §Workspace layout line: *"strictly typed
/// workflow JSON structs"* — the structured typing that matters lives
/// upstream (panel prompt → workflow), not at the HTTP edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow(serde_json::Value);

impl Workflow {
    /// Parse a workflow from a JSON string (e.g. read from disk).
    pub fn from_json_str(s: &str) -> Result<Self, TtError> {
        let v: serde_json::Value = serde_json::from_str(s)
            .map_err(|e| TtError::Canon(format!("workflow JSON parse error: {e}")))?;
        Self::from_value(v)
    }

    /// Wrap an already-parsed `serde_json::Value`. The value must be a JSON
    /// object — ComfyUI expects `{"<node_id>": { ... }, ...}`.
    pub fn from_value(v: serde_json::Value) -> Result<Self, TtError> {
        if !v.is_object() {
            return Err(TtError::Canon(
                "workflow must be a JSON object mapping node_id → node".into(),
            ));
        }
        Ok(Self(v))
    }

    /// Borrow the underlying JSON value.
    pub fn as_json(&self) -> &serde_json::Value {
        &self.0
    }

    /// Extract the inner `serde_json::Value`.
    pub fn into_json(self) -> serde_json::Value {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

/// Result of a `/system_stats` probe.
///
/// `reachable == true` means the untrusted ComfyUI container answered a
/// 2xx on `/system_stats`. The orchestrator uses this as the liveness
/// check before submitting a workflow — a fail-fast on infra problems
/// rather than a half-submitted prompt that never resolves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Whether the ComfyUI HTTP endpoint responded successfully.
    pub reachable: bool,
    /// Raw body of the `/system_stats` response (for diagnostics).
    pub raw: Option<String>,
}

// ---------------------------------------------------------------------------
// History / status
// ---------------------------------------------------------------------------

/// One output file in a `/history` response for a completed prompt.
///
/// ComfyUI reports per-node outputs with `filename`, `subfolder`, and
/// `type` (one of `"output"`, `"temp"`, `"input"`). The PNG bytes are
/// fetched via [`ComfyClient::fetch_output`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputFile {
    pub filename: String,
    #[serde(default)]
    pub subfolder: String,
    /// The `type` field in ComfyUI's response. Renamed because `type` is a
    /// reserved word in Rust.
    #[serde(rename = "type")]
    pub kind: String,
}

/// Parsed `/history/<id>` response.
///
/// Shape mirrors the relevant subset of ComfyUI's history envelope:
///
/// ```json
/// {
///   "<prompt_id>": {
///     "status": { "status_str": "success", "completed": true, ... },
///     "outputs": { "<node_id>": { "images": [ {filename, subfolder, type}, ... ] } }
///   }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptHistory {
    /// Raw status string reported by ComfyUI (`"success"`, `"error"`,
    /// sometimes absent while the prompt is still executing).
    pub status: Option<String>,
    /// Whether ComfyUI considers the prompt fully completed.
    pub completed: bool,
    /// Node-id → list of output files.
    pub outputs: BTreeMap<String, Vec<OutputFile>>,
    /// First error message surfaced by the history envelope (workflow
    /// validation, OOM, missing model, etc.), if any.
    pub error: Option<String>,
}

impl PromptHistory {
    /// The first PNG output across all nodes, if any — convenience used
    /// by the common case where a workflow renders exactly one image.
    pub fn first_output(&self) -> Option<&OutputFile> {
        self.outputs.values().flatten().next()
    }
}

/// Progressively-reported status yielded by [`ComfyClient::watch`].
///
/// `Pending` is the pre-completion phase (no error, not yet finished).
/// `Completed` terminates the stream with a parsed `PromptHistory`.
/// Errors from polling surface as `Err(TtError)` items on the stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComfyStatus {
    /// Prompt is queued or executing; no outputs yet.
    Pending,
    /// Prompt finished — either with outputs, or with an error field set.
    Completed(PromptHistory),
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Handle to the running ComfyUI instance.
///
/// The client is cheap to clone — `reqwest::Client` shares a connection
/// pool across clones, and `Url` is a plain string wrapper.
#[derive(Debug, Clone)]
pub struct ComfyClient {
    base_url: Url,
    client: reqwest::Client,
}

impl ComfyClient {
    /// Build a client pointing at the given base URL (e.g.
    /// `http://127.0.0.1:8188/`). The base URL must be absolute and have
    /// a trailing slash-tolerant form; paths are joined onto it verbatim.
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Build a client with a caller-supplied `reqwest::Client`
    /// (useful in tests that want custom timeouts).
    pub fn with_client(base_url: Url, client: reqwest::Client) -> Self {
        Self { base_url, client }
    }

    /// The base URL this client targets.
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Probe `/system_stats`. Returns a `HealthStatus` indicating whether
    /// the container is reachable; transport-level failures return
    /// `reachable: false` rather than erroring — the spec wants a boolean
    /// liveness probe here, not an exception.
    pub async fn health(&self) -> Result<HealthStatus, TtError> {
        let url = join_path(&self.base_url, "system_stats")?;
        match self.client.get(url).send().await {
            Ok(resp) => {
                let ok = resp.status().is_success();
                let body = resp.text().await.ok();
                Ok(HealthStatus {
                    reachable: ok,
                    raw: body,
                })
            }
            Err(_) => Ok(HealthStatus {
                reachable: false,
                raw: None,
            }),
        }
    }

    /// Submit a workflow to `POST /prompt`. Returns the ComfyUI prompt id.
    ///
    /// ComfyUI wraps the workflow under `"prompt"` at the top level of
    /// the request body. Non-2xx responses carry JSON describing the
    /// validation failure — we map those to [`TtError::Canon`] because
    /// a malformed workflow is an upstream (spec/tt-specs) defect, not
    /// an infra outage. Transport failures map to [`TtError::Infra`].
    pub async fn submit(&self, workflow: Workflow) -> Result<PromptId, TtError> {
        let url = join_path(&self.base_url, "prompt")?;
        let body = serde_json::json!({ "prompt": workflow.into_json() });

        let resp = self
            .client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| TtError::Infra(format!("comfy submit transport error: {e}")))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| {
            TtError::Infra(format!("comfy submit body-read error: {e}"))
        })?;

        if !status.is_success() {
            return Err(TtError::Canon(format!(
                "comfy rejected workflow (HTTP {status}): {text}"
            )));
        }

        let parsed: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
            TtError::Infra(format!("comfy submit response parse error: {e}: {text}"))
        })?;

        let id = parsed
            .get("prompt_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                TtError::Infra(format!(
                    "comfy submit response missing `prompt_id` field: {text}"
                ))
            })?;

        Ok(PromptId(id.to_string()))
    }

    /// Fetch the parsed `/history/<id>` envelope.
    ///
    /// While the prompt is still executing, ComfyUI either returns an
    /// empty object `{}` or an entry with `completed: false` — both
    /// surface here as `completed: false`. Terminal error states
    /// populate [`PromptHistory::error`]; the caller decides whether
    /// to treat them as canon or infra.
    pub async fn history(&self, id: &PromptId) -> Result<PromptHistory, TtError> {
        let url = join_path(&self.base_url, &format!("history/{}", id.0))?;

        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| TtError::Infra(format!("comfy history transport error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(TtError::Infra(format!(
                "comfy history HTTP {status}: {text}"
            )));
        }

        let envelope: serde_json::Value = resp.json().await.map_err(|e| {
            TtError::Infra(format!("comfy history JSON parse error: {e}"))
        })?;

        Ok(parse_history(&envelope, id))
    }

    /// Poll `/history/<id>` at [`WATCH_POLL_INTERVAL`] until the prompt
    /// completes or errors, yielding [`ComfyStatus`] updates.
    ///
    /// The stream yields `Ok(ComfyStatus::Pending)` for each empty poll,
    /// then exactly one terminal `Ok(ComfyStatus::Completed(history))`
    /// and stops. Transport-level errors from individual polls yield
    /// `Err(TtError::Infra(..))` and also terminate the stream — the
    /// orchestrator will emit `ComfyEvent::Failed`/`Timeout` at the
    /// outer layer.
    ///
    /// We use the crate-level constant `WATCH_POLL_INTERVAL` rather than
    /// HTTP long-poll because ComfyUI's upstream `/history` endpoint is
    /// a snapshot read, not an event stream. A websocket-based watcher
    /// is reserved for future convergence (see `orchestrator/spec.md`
    /// §Future convergence — "gRPC-style event stream between tt-render
    /// and Calmecac").
    pub fn watch(
        &self,
        id: PromptId,
    ) -> impl Stream<Item = Result<ComfyStatus, TtError>> + Send + 'static {
        let client = self.clone();
        futures::stream::unfold(WatchState::Polling(client, id), |state| async move {
            match state {
                WatchState::Done => None,
                WatchState::Polling(client, id) => match client.history(&id).await {
                    Ok(hist) if hist.completed || hist.error.is_some() => {
                        Some((Ok(ComfyStatus::Completed(hist)), WatchState::Done))
                    }
                    Ok(_) => {
                        // Not done yet — yield a Pending tick, then sleep
                        // before the next poll. Consumers treat Pending as a
                        // liveness beacon, not a content payload.
                        tokio::time::sleep(WATCH_POLL_INTERVAL).await;
                        Some((Ok(ComfyStatus::Pending), WatchState::Polling(client, id)))
                    }
                    Err(e) => Some((Err(e), WatchState::Done)),
                },
            }
        })
    }

    /// Fetch the raw bytes of a rendered output file via
    /// `GET /view?filename=...&subfolder=...&type=...`.
    ///
    /// `kind` is ComfyUI's `type` parameter — normally `"output"` for
    /// finalized renders; `"temp"` for intermediate previews. Returns
    /// the full body (PNG bytes) on success; transport or non-2xx
    /// responses map to [`TtError::Infra`].
    pub async fn fetch_output(
        &self,
        filename: &str,
        subfolder: &str,
        kind: &str,
    ) -> Result<Vec<u8>, TtError> {
        let mut url = join_path(&self.base_url, "view")?;
        url.query_pairs_mut()
            .append_pair("filename", filename)
            .append_pair("subfolder", subfolder)
            .append_pair("type", kind);

        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| TtError::Infra(format!("comfy view transport error: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(TtError::Infra(format!(
                "comfy view HTTP {status}: {text}"
            )));
        }

        let bytes = resp.bytes().await.map_err(|e| {
            TtError::Infra(format!("comfy view body-read error: {e}"))
        })?;
        Ok(bytes.to_vec())
    }
}

// ---------------------------------------------------------------------------
// Internal: watch state machine
// ---------------------------------------------------------------------------

enum WatchState {
    Polling(ComfyClient, PromptId),
    Done,
}

// ---------------------------------------------------------------------------
// Internal: URL joining + history parsing
// ---------------------------------------------------------------------------

/// Join `path` onto `base`, tolerating base URLs with or without a trailing
/// slash. The ComfyUI port URL is whatever the launcher gives us; we don't
/// get to dictate shape.
fn join_path(base: &Url, path: &str) -> Result<Url, TtError> {
    // `Url::join` treats a trailing slash as a directory — if the base
    // doesn't have one, we append it before joining.
    let mut base = base.clone();
    if !base.path().ends_with('/') {
        let mut p = base.path().to_string();
        p.push('/');
        base.set_path(&p);
    }
    base.join(path)
        .map_err(|e| TtError::Infra(format!("invalid comfy URL join: {e}")))
}

/// Parse the raw `/history` JSON envelope into a `PromptHistory`.
///
/// ComfyUI's envelope layers the prompt id inside the top object; we
/// accept either `{ "<id>": { ... } }` or `{ ... }` (when the caller
/// already drilled in).
fn parse_history(envelope: &serde_json::Value, id: &PromptId) -> PromptHistory {
    let inner = envelope.get(id.as_str()).unwrap_or(envelope);

    let status_obj = inner.get("status");
    let status_str = status_obj
        .and_then(|s| s.get("status_str"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let completed = status_obj
        .and_then(|s| s.get("completed"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Error surfacing: ComfyUI reports validation / execution errors
    // either as `"error"` at the envelope root, as a `status_str == "error"`,
    // or as a populated `"messages"` array on the status object. We
    // sniff all three.
    let error = inner
        .get("error")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            if status_str.as_deref() == Some("error") {
                Some(
                    status_obj
                        .and_then(|s| s.get("messages"))
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "comfy reported error".into()),
                )
            } else {
                None
            }
        });

    let mut outputs: BTreeMap<String, Vec<OutputFile>> = BTreeMap::new();
    if let Some(out_obj) = inner.get("outputs").and_then(|v| v.as_object()) {
        for (node_id, node_out) in out_obj {
            // ComfyUI typically nests image outputs under "images":
            //   { "<node_id>": { "images": [ {filename, subfolder, type}, ... ] } }
            // Other output keys (e.g. "gifs", "audio") are ignored by
            // this client — tt-compose only consumes PNGs.
            if let Some(imgs) = node_out.get("images").and_then(|v| v.as_array()) {
                let files: Vec<OutputFile> = imgs
                    .iter()
                    .filter_map(|img| serde_json::from_value::<OutputFile>(img.clone()).ok())
                    .collect();
                if !files.is_empty() {
                    outputs.insert(node_id.clone(), files);
                }
            }
        }
    }

    PromptHistory {
        status: status_str,
        completed,
        outputs,
        error,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use std::time::Duration;
    use wiremock::matchers::{body_json, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Build a `ComfyClient` pointed at a freshly-started `MockServer`.
    async fn mock_client() -> (MockServer, ComfyClient) {
        let server = MockServer::start().await;
        let url = Url::parse(&server.uri()).unwrap();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();
        (server, ComfyClient::with_client(url, client))
    }

    fn sample_workflow() -> Workflow {
        Workflow::from_json_str(r#"{"3":{"class_type":"KSampler","inputs":{"seed":42}}}"#)
            .unwrap()
    }

    // ---- Workflow --------------------------------------------------------

    #[test]
    fn workflow_rejects_non_object() {
        let err = Workflow::from_json_str("[1,2,3]").unwrap_err();
        assert_eq!(err.class(), tt_core::FailureClass::Canon);

        let err = Workflow::from_json_str("not json").unwrap_err();
        assert_eq!(err.class(), tt_core::FailureClass::Canon);
    }

    #[test]
    fn workflow_accepts_object_and_roundtrips() {
        let wf = sample_workflow();
        assert!(wf.as_json().get("3").is_some());
        let again = Workflow::from_value(wf.as_json().clone()).unwrap();
        assert_eq!(wf.as_json(), again.as_json());
    }

    // ---- health ----------------------------------------------------------

    #[tokio::test]
    async fn health_reports_reachable_on_2xx() {
        let (server, client) = mock_client().await;
        Mock::given(method("GET"))
            .and(path("/system_stats"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{\"ok\":1}"))
            .mount(&server)
            .await;

        let h = client.health().await.unwrap();
        assert!(h.reachable);
        assert_eq!(h.raw.as_deref(), Some("{\"ok\":1}"));
    }

    #[tokio::test]
    async fn health_reports_unreachable_on_connection_failure() {
        // Use a valid but unroutable URL — wiremock not mounted here.
        let bogus = Url::parse("http://127.0.0.1:1/").unwrap();
        let client = ComfyClient::with_client(
            bogus,
            reqwest::Client::builder()
                .timeout(Duration::from_millis(500))
                .build()
                .unwrap(),
        );
        let h = client.health().await.unwrap();
        assert!(!h.reachable);
    }

    // ---- submit ----------------------------------------------------------

    #[tokio::test]
    async fn submit_returns_prompt_id() {
        let (server, client) = mock_client().await;
        let wf = sample_workflow();
        let expected_body = serde_json::json!({ "prompt": wf.as_json() });

        Mock::given(method("POST"))
            .and(path("/prompt"))
            .and(body_json(&expected_body))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "prompt_id": "abc-123",
                "number": 0,
                "node_errors": {}
            })))
            .mount(&server)
            .await;

        let id = client.submit(wf).await.unwrap();
        assert_eq!(id.as_str(), "abc-123");
    }

    #[tokio::test]
    async fn submit_non_2xx_maps_to_canon() {
        let (server, client) = mock_client().await;
        Mock::given(method("POST"))
            .and(path("/prompt"))
            .respond_with(ResponseTemplate::new(400).set_body_string(
                r#"{"error":{"type":"invalid_prompt","message":"bad node"}}"#,
            ))
            .mount(&server)
            .await;

        let err = client.submit(sample_workflow()).await.unwrap_err();
        assert_eq!(err.class(), tt_core::FailureClass::Canon);
    }

    #[tokio::test]
    async fn submit_transport_error_maps_to_infra() {
        let bogus = Url::parse("http://127.0.0.1:1/").unwrap();
        let client = ComfyClient::with_client(
            bogus,
            reqwest::Client::builder()
                .timeout(Duration::from_millis(500))
                .build()
                .unwrap(),
        );
        let err = client.submit(sample_workflow()).await.unwrap_err();
        assert_eq!(err.class(), tt_core::FailureClass::Infra);
    }

    // ---- history ---------------------------------------------------------

    fn history_envelope_completed(id: &str, node: &str, filename: &str) -> serde_json::Value {
        serde_json::json!({
            id: {
                "status": { "status_str": "success", "completed": true },
                "outputs": {
                    node: {
                        "images": [
                            { "filename": filename, "subfolder": "", "type": "output" }
                        ]
                    }
                }
            }
        })
    }

    #[tokio::test]
    async fn history_parses_completed_prompt() {
        let (server, client) = mock_client().await;
        let id = PromptId("xyz".into());
        Mock::given(method("GET"))
            .and(path("/history/xyz"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                history_envelope_completed("xyz", "9", "panel_0001.png"),
            ))
            .mount(&server)
            .await;

        let hist = client.history(&id).await.unwrap();
        assert_eq!(hist.status.as_deref(), Some("success"));
        assert!(hist.completed);
        assert!(hist.error.is_none());
        let first = hist.first_output().expect("one output");
        assert_eq!(first.filename, "panel_0001.png");
        assert_eq!(first.kind, "output");
    }

    #[tokio::test]
    async fn history_parses_pending_empty_envelope() {
        let (server, client) = mock_client().await;
        let id = PromptId("pending".into());
        Mock::given(method("GET"))
            .and(path("/history/pending"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let hist = client.history(&id).await.unwrap();
        assert!(!hist.completed);
        assert!(hist.outputs.is_empty());
        assert!(hist.error.is_none());
    }

    #[tokio::test]
    async fn history_surfaces_error_status() {
        let (server, client) = mock_client().await;
        let id = PromptId("bad".into());
        Mock::given(method("GET"))
            .and(path("/history/bad"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "bad": {
                    "status": {
                        "status_str": "error",
                        "completed": true,
                        "messages": [["execution_error", { "exception_type": "OutOfMemoryError" }]]
                    },
                    "outputs": {}
                }
            })))
            .mount(&server)
            .await;

        let hist = client.history(&id).await.unwrap();
        assert!(hist.completed);
        assert!(hist.error.is_some());
        assert!(
            hist.error.as_deref().unwrap().contains("OutOfMemoryError"),
            "expected OOM in error, got {:?}",
            hist.error
        );
    }

    // ---- watch -----------------------------------------------------------

    #[tokio::test]
    async fn watch_yields_pending_then_completion() {
        let (server, client) = mock_client().await;
        let id = PromptId("watch-1".into());

        // First response: empty (pending). `up_to_n_times(1)` — wiremock
        // serves this once, then falls through to the next matcher.
        Mock::given(method("GET"))
            .and(path("/history/watch-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        // Subsequent responses: completed.
        Mock::given(method("GET"))
            .and(path("/history/watch-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                history_envelope_completed("watch-1", "9", "done.png"),
            ))
            .mount(&server)
            .await;

        let mut stream = Box::pin(client.watch(id));
        let first = stream.next().await.expect("first item").expect("ok");
        assert_eq!(first, ComfyStatus::Pending);
        let second = stream.next().await.expect("second item").expect("ok");
        match second {
            ComfyStatus::Completed(h) => {
                assert!(h.completed);
                assert_eq!(h.first_output().unwrap().filename, "done.png");
            }
            other => panic!("expected Completed, got {other:?}"),
        }
        // Stream terminates.
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn watch_terminates_immediately_on_completion() {
        let (server, client) = mock_client().await;
        let id = PromptId("fast".into());
        Mock::given(method("GET"))
            .and(path("/history/fast"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                history_envelope_completed("fast", "9", "out.png"),
            ))
            .mount(&server)
            .await;

        let mut stream = Box::pin(client.watch(id));
        let first = stream.next().await.expect("item").expect("ok");
        assert!(matches!(first, ComfyStatus::Completed(_)));
        assert!(stream.next().await.is_none());
    }

    // ---- fetch_output ----------------------------------------------------

    #[tokio::test]
    async fn fetch_output_returns_bytes() {
        let (server, client) = mock_client().await;
        // Minimal PNG header — enough to prove bytes round-trip.
        let png_bytes: Vec<u8> = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
        Mock::given(method("GET"))
            .and(path("/view"))
            .and(query_param("filename", "done.png"))
            .and(query_param("subfolder", ""))
            .and(query_param("type", "output"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(png_bytes.clone())
                    .insert_header("content-type", "image/png"),
            )
            .mount(&server)
            .await;

        let got = client
            .fetch_output("done.png", "", "output")
            .await
            .unwrap();
        assert_eq!(got, png_bytes);
    }

    #[tokio::test]
    async fn fetch_output_non_2xx_is_infra() {
        let (server, client) = mock_client().await;
        Mock::given(method("GET"))
            .and(path("/view"))
            .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
            .mount(&server)
            .await;

        let err = client
            .fetch_output("missing.png", "", "output")
            .await
            .unwrap_err();
        assert_eq!(err.class(), tt_core::FailureClass::Infra);
    }

    // ---- url joining -----------------------------------------------------

    #[test]
    fn join_path_tolerates_missing_trailing_slash() {
        let base = Url::parse("http://127.0.0.1:8188").unwrap();
        let joined = join_path(&base, "prompt").unwrap();
        assert_eq!(joined.as_str(), "http://127.0.0.1:8188/prompt");
    }

    #[test]
    fn join_path_works_with_trailing_slash() {
        let base = Url::parse("http://127.0.0.1:8188/").unwrap();
        let joined = join_path(&base, "history/abc").unwrap();
        assert_eq!(joined.as_str(), "http://127.0.0.1:8188/history/abc");
    }
}
