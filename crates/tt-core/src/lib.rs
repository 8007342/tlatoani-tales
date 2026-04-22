//! Tlatoāni Tales — shared type foundation.
//!
//! This crate exports the newtype wrappers, error taxonomy, and podman
//! hardening flag constants used by the rest of the `tt-*` workspace.
//! Nothing here performs I/O; every other crate depends on these types.
//!
//! Governing spec: `openspec/specs/orchestrator/spec.md`.
//!
// @trace spec:orchestrator, spec:isolation
// @Lesson S1-1300

use serde::{Deserialize, Serialize};
use std::fmt;

pub mod podman;

/// Identifier for a published lesson, e.g. `"S1-100-volatile-is-dangerous"`
/// in the full form or `"S1-100"` in the short form used in code citations.
///
/// See `openspec/specs/lessons/spec.md` for the canonical naming rules.
// @trace spec:lessons
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LessonId(pub String);

impl fmt::Display for LessonId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl LessonId {
    /// Construct a lesson id from a string-like value. No validation is
    /// performed here; `tt-specs` checks membership against the registry.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// Name of an OpenSpec spec, e.g. `"orchestrator"` or `"visual-qa-loop"`.
///
/// `@trace spec:<name>` citations reference this.
// @trace spec:orchestrator
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpecName(pub String);

impl fmt::Display for SpecName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl SpecName {
    /// Construct a spec name. No validation here; `tt-lint` enforces the
    /// trace-presence and slug-in-registry rules.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// Content-addressed hash (SHA-256 bytes) for a rendered panel or any other
/// cache cell in the workspace. Renders as lowercase hex via `Display`.
// @trace spec:orchestrator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PanelHash(pub [u8; 32]);

impl fmt::Display for PanelHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

impl PanelHash {
    /// Wrap a raw 32-byte SHA-256 digest.
    pub fn from_bytes(b: [u8; 32]) -> Self {
        Self(b)
    }

    /// Render as lowercase hex. Same as `Display`; provided as a convenience
    /// for call sites that prefer a method name.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

/// Strip identifier — the two-digit episode number (01..15 for Season 1).
// @trace spec:trace-plate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StripId(pub u16);

impl fmt::Display for StripId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}", self.0)
    }
}

/// Season identifier (Season 1 → `SeasonId { n: 1 }`).
// @trace spec:seasons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SeasonId {
    pub n: u16,
}

/// Trust zone — every artefact in the workspace sits in one.
///
/// The orchestrator's crates run in `Trusted`; ComfyUI / ai-toolkit / ollama
/// runtimes run in an `Untrusted(Role)` container. See
/// `openspec/specs/isolation/spec.md`.
// @trace spec:isolation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Zone {
    /// Our Rust code, inside the `tlatoani-tales` toolbox.
    Trusted,
    /// A hardened disposable container hosting untrusted code.
    Untrusted(Role),
}

/// Role an untrusted container is playing in the render pipeline.
// @trace spec:isolation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// ComfyUI + FLUX + Qwen-Image + ollama VLM.
    Inference,
    /// ai-toolkit LoRA trainer.
    Trainer,
    /// Distribution-standard httpd serving the Calmecac bundle.
    Viewer,
}

/// Classification of a failure — the orchestrator exit codes hinge on this.
///
/// `Infra` → exit 30/31. `Canon` → exit 10/20. See
/// `openspec/specs/orchestrator/spec.md` §Failure modes.
// @trace spec:orchestrator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureClass {
    /// The tool is wrong — container unreachable, model missing, bind-mount denied.
    Infra,
    /// The comic is wrong — drift past the threshold, tombstoned lesson cited.
    Canon,
}

/// Canonical error taxonomy surface for library crates. Individual crates may
/// re-export this or wrap it; binary crates lean on `anyhow::Error` at the
/// edges and preserve the `FailureClass` via context.
// @trace spec:orchestrator
#[derive(Debug, thiserror::Error)]
pub enum TtError {
    #[error("infra failure: {0}")]
    Infra(String),
    #[error("canon failure: {0}")]
    Canon(String),
    #[error("usage error: {0}")]
    Usage(String),
}

impl TtError {
    /// Return the failure class that governs the exit code for this error.
    pub fn class(&self) -> FailureClass {
        match self {
            TtError::Infra(_) => FailureClass::Infra,
            TtError::Canon(_) => FailureClass::Canon,
            TtError::Usage(_) => FailureClass::Canon, // usage errors exit 40 but aren't infra
        }
    }
}

/// Convenience alias used throughout the workspace.
pub type Result<T> = std::result::Result<T, TtError>;
