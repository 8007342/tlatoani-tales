//! Tlatoāni Tales — async HTTP client for ComfyUI.
//!
//! The trusted Rust zone never imports Python. It reaches the ComfyUI
//! runtime — which lives in a hardened `--network=none` container — over
//! the forwarded localhost port declared by the launcher. One workflow
//! submit becomes a `prompt_id`; `stream()` turns that id into a stream of
//! typed `ComfyEvent`s.
//!
//! Governing spec: `openspec/specs/orchestrator/spec.md`,
//! `openspec/specs/isolation/spec.md`.
//!
// @trace spec:orchestrator, spec:isolation

use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tt_core::TtError;
use tt_events::ComfyEvent;
use url::Url;

/// Opaque ComfyUI prompt id. Returned from `submit`, consumed by `stream`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptId(pub String);

/// A ComfyUI workflow — a typed JSON object whose shape mirrors the nodes
/// ComfyUI understands. Scaffold: opaque `serde_json::Value` until the real
/// node graph typing lands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow(pub serde_json::Value);

/// Handle to the running ComfyUI instance.
pub struct ComfyClient {
    #[allow(dead_code)]
    base_url: Url,
    #[allow(dead_code)]
    client: reqwest::Client,
}

impl ComfyClient {
    /// Build a client pointing at the given base URL (e.g.
    /// `http://127.0.0.1:8188/`).
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Submit a workflow. Returns the ComfyUI prompt id.
    pub async fn submit(&self, _workflow: Workflow) -> Result<PromptId, TtError> {
        unimplemented!("tt-comfy submit is scaffolded; implementation lands in a later change")
    }

    /// Stream progress/result events for a previously submitted prompt.
    pub fn stream(&self, _prompt_id: PromptId) -> impl Stream<Item = ComfyEvent> + Send + 'static {
        // Empty stream placeholder until the real long-poll of `/history/:id`
        // lands. Callers can still wire subscribers without compiler churn.
        futures::stream::empty()
    }
}
