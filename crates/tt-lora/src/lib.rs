//! Tlatoāni Tales — LoRA training wrapper.
//!
//! `tt-lora` is the trusted-zone Rust crate that *is* the LoRA subsystem as
//! far as the rest of the workspace is concerned. Nothing else in our code
//! talks to ai-toolkit directly. Starts the `tlatoani-tales-trainer`
//! container (idempotent), launches ai-toolkit inside with a rendered
//! config file, parses stdout into typed `LoraEvent`s, and SHA-256s the
//! trained `.safetensors` inside the trusted zone — the untrusted zone
//! cannot lie about its output.
//!
//! Governing spec: `openspec/specs/character-loras/spec.md`,
//! `openspec/specs/isolation/spec.md`.
//!
// @trace spec:character-loras, spec:isolation
// @Lesson S1-500
// @Lesson S1-1500

use serde::{Deserialize, Serialize};
use std::path::Path;
use tt_core::TtError;

/// A character name as declared in the LoRA manifest, e.g. `"tlatoāni"` or
/// `"covi"`. UTF-8 throughout — the file layout carries the macron even
/// though the container name (TB03) cannot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterName(pub String);

/// The committed `lora-manifest.json` contract — reproducibility artefact.
///
/// Fields match `openspec/specs/character-loras/spec.md` §Manifest; kept as
/// a flat scaffold until the full schema materializes in a later change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraManifest {
    pub character: String,
    pub version: u32,
    pub trigger_token: String,
    pub output_sha256: String,
    pub trained_at: String,
}

/// Handle to the trainer container.
pub struct LoraTrainer {
    #[allow(dead_code)]
    container: String,
}

impl LoraTrainer {
    /// Bind this handle to the canonical trainer container
    /// (`tlatoani-tales-trainer` — ASCII per TB03).
    pub fn new() -> Self {
        Self {
            container: "tlatoani-tales-trainer".to_string(),
        }
    }

    /// Train a character LoRA from the reference sheets at `refs`. Emits
    /// `LoraEvent` on the shared bus (caller wires it up). Returns the
    /// committed manifest.
    pub async fn train(
        &self,
        _character: &CharacterName,
        _refs: &Path,
    ) -> Result<LoraManifest, TtError> {
        unimplemented!("tt-lora train is scaffolded; real subprocess wiring lands in a later change")
    }
}

impl Default for LoraTrainer {
    fn default() -> Self {
        Self::new()
    }
}
