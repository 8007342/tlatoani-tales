//! Tlatoāni Tales — content-addressed hashing.
//!
//! Canonicalization makes human-edited markdown hash stably: Unicode
//! normalization to NFC, line endings to LF, trimmed leading/trailing
//! whitespace, runs of blank lines collapsed. Stable bytes in, stable hash
//! out.
//!
//! Governing spec: `openspec/specs/orchestrator/spec.md` §Content addressing.
//!
// @trace spec:orchestrator, spec:character-loras
// @Lesson S1-500
// @Lesson S1-1300

use sha2::{Digest, Sha256};
use tt_core::PanelHash;
use unicode_normalization::UnicodeNormalization;

/// Canonicalize a human-edited string into the bytes that feed the SHA-256.
///
/// - NFC Unicode normalization (so `Tlatoāni` hashes the same however the
///   macron is composed).
/// - LF line endings.
/// - UTF-8 (input is already `&str`, so this is automatic).
/// - Trimmed leading/trailing whitespace.
/// - Runs of blank lines collapsed to exactly one blank line.
pub fn canonicalize(input: &str) -> String {
    // NFC + normalize line endings in one pass.
    let nfc: String = input.nfc().collect();
    let lf = nfc.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = lf.trim();

    // Collapse runs of blank lines.
    let mut out = String::with_capacity(trimmed.len());
    let mut blank_streak = 0usize;
    for line in trimmed.lines() {
        if line.trim().is_empty() {
            blank_streak += 1;
            if blank_streak == 1 {
                out.push('\n');
            }
        } else {
            blank_streak = 0;
            out.push_str(line);
            out.push('\n');
        }
    }
    // Strip trailing newline added by the loop.
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

/// SHA-256 hex digest of arbitrary bytes. Lowercase, 64 characters.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// SHA-256 raw digest of arbitrary bytes.
pub fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

/// Stand-in for the global style hash — the SHA-256 of the concatenated
/// canonicalized bodies of `style-bible`, `character-canon`,
/// `symbol-dictionary`, and `trace-plate`. Real implementation takes a
/// typed spec graph; this signature is a placeholder until `tt-specs`
/// stabilises `SpecGraph`.
///
/// See `openspec/specs/orchestrator/spec.md` §Global style hash.
pub fn global_style_hash(bodies_concat: &str) -> PanelHash {
    PanelHash::from_bytes(sha256_bytes(canonicalize(bodies_concat).as_bytes()))
}

/// Panel hash — concatenates every input that uniquely determines a panel's
/// pixels and SHA-256s the result. Mutating any input produces a new hash;
/// a new hash is a cache miss; the miss triggers a re-render. Monotonic.
///
/// Inputs (sorted by character name for `loras`) match
/// `openspec/specs/orchestrator/spec.md` §Panel hash.
pub fn panel_hash(
    prompt: &str,
    style: &PanelHash,
    loras: &[PanelHash],
    seed: u64,
    model: &str,
    qwen: Option<&str>,
    schema_version: u32,
) -> PanelHash {
    let mut hasher = Sha256::new();
    hasher.update(canonicalize(prompt).as_bytes());
    hasher.update(style.0);
    for lora in loras {
        hasher.update(lora.0);
    }
    hasher.update(seed.to_le_bytes());
    hasher.update(model.as_bytes());
    if let Some(qwen) = qwen {
        hasher.update(qwen.as_bytes());
    }
    hasher.update(schema_version.to_le_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    PanelHash::from_bytes(out)
}
