//! Tlatoāni Tales — content-addressed hashing.
//!
//! Canonicalization turns human-edited markdown into bytes that hash stably:
//! Unicode normalization to NFC, line endings to LF, trimmed outer
//! whitespace, runs of blank lines collapsed. Stable bytes in, stable hash
//! out. Everything downstream — the panel cache, the global style hash, the
//! LoRA manifest hash — composes atop this one primitive.
//!
//! The cache this crate feeds is the materialization of ME05: a G-Set CRDT
//! whose cells are `sha256(inputs)` keys. Edits reconcile monotonically;
//! union never contradicts. That is why canonicalization matters — two
//! byte-different-but-semantically-identical inputs must collapse to the
//! same cell, or the lattice leaks. `@Lesson S1-500` (*edits-that-reconcile*)
//! is why `canonicalize` exists; `@Lesson S1-1300` (*loop-closes*) is why
//! `panel_hash` exists.
//!
//! Governing spec: `openspec/specs/orchestrator/spec.md` §Content addressing.
//!
// @trace spec:orchestrator, spec:character-loras, spec:style-bible
// @Lesson S1-500
// @Lesson S1-1300

use sha2::{Digest, Sha256};
use tt_core::PanelHash;
use unicode_normalization::UnicodeNormalization;

// ---------------------------------------------------------------------------
// Canonicalization
// ---------------------------------------------------------------------------

/// Canonicalize a human-edited string into the bytes that feed a SHA-256.
///
/// Rules, in order:
/// 1. NFC Unicode normalization — so `Tlatoāni` hashes the same whether the
///    macron arrived as a precomposed `ā` or as `a` + combining macron.
/// 2. LF line endings — CRLF → LF, lone CR → LF.
/// 3. UTF-8 — the input is already `&str`, so this is automatic.
/// 4. Outer whitespace trimmed.
/// 5. Runs of blank lines collapsed to exactly one blank line. Authors who
///    add vertical padding do not invalidate caches.
///
/// Idempotent: `canonicalize(canonicalize(x)) == canonicalize(x)`. This
/// property is unit-tested below and is load-bearing for ME05 (a G-Set whose
/// keys are derived from non-idempotent functions silently leaks cells).
///
/// `@trace spec:style-bible, spec:character-canon, spec:symbol-dictionary,
/// spec:trace-plate`
pub fn canonicalize(input: &str) -> String {
    // Step 1+2: NFC and line-ending normalization in one pass.
    let nfc: String = input.nfc().collect();
    let lf = nfc.replace("\r\n", "\n").replace('\r', "\n");

    // Step 4: outer trim.
    let trimmed = lf.trim();

    // Step 5: collapse runs of blank lines.
    let mut out = String::with_capacity(trimmed.len());
    let mut in_blank_run = false;
    let mut first_line = true;
    for line in trimmed.lines() {
        if line.trim().is_empty() {
            if !in_blank_run && !first_line {
                // Emit exactly one blank line — represented as a bare `\n`
                // separating the previous content line from the next.
                out.push('\n');
            }
            in_blank_run = true;
        } else {
            if !first_line {
                out.push('\n');
            }
            out.push_str(line);
            in_blank_run = false;
            first_line = false;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// SHA-256 helpers
// ---------------------------------------------------------------------------

/// Raw SHA-256 of arbitrary bytes.
#[inline]
pub fn sha256_bytes(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

/// SHA-256 of arbitrary bytes, rendered as lowercase 64-char hex.
#[inline]
pub fn sha256_hex(input: &[u8]) -> String {
    hex::encode(sha256_bytes(input))
}

/// Canonicalize a string then SHA-256 it; wrap as a `PanelHash`. The typical
/// entry point for hashing human-edited content.
#[inline]
pub fn sha256_of_string(input: &str) -> PanelHash {
    PanelHash::from_bytes(sha256_bytes(canonicalize(input).as_bytes()))
}

// ---------------------------------------------------------------------------
// Global style hash
// ---------------------------------------------------------------------------

/// Global style hash — SHA-256 of the four canonical style-spec bodies
/// concatenated with `\n` separators, in a fixed order:
///
///   1. `style-bible`
///   2. `character-canon`
///   3. `symbol-dictionary`
///   4. `trace-plate`
///
/// The order is alphabetical by spec name and is **part of the formula**.
/// The function is deterministic in the order it receives its arguments —
/// callers must not sort or permute. Swapping two inputs at the call site
/// yields a different hash (tested below). The purpose of fixing the order
/// here is to pin the recipe down as this spec text dictates; `tt-specs`
/// passes the bodies in this order and nowhere else.
///
/// Separators exist so two adjacent bodies cannot accidentally concatenate
/// into a valid third body. A `\n` byte between each field removes the
/// ambiguity without adding meaningful length.
///
/// Mutating any of the four style specs invalidates every cached panel
/// project-wide.
///
/// See `openspec/specs/orchestrator/spec.md` §Global style hash.
///
/// `@trace spec:orchestrator, spec:style-bible, spec:character-canon,
/// spec:symbol-dictionary, spec:trace-plate`
pub fn global_style_hash(
    style_bible: &str,
    character_canon: &str,
    symbol_dictionary: &str,
    trace_plate: &str,
) -> PanelHash {
    let mut hasher = Sha256::new();
    hasher.update(canonicalize(style_bible).as_bytes());
    hasher.update(b"\n");
    hasher.update(canonicalize(character_canon).as_bytes());
    hasher.update(b"\n");
    hasher.update(canonicalize(symbol_dictionary).as_bytes());
    hasher.update(b"\n");
    hasher.update(canonicalize(trace_plate).as_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    PanelHash::from_bytes(out)
}

// ---------------------------------------------------------------------------
// Panel hash
// ---------------------------------------------------------------------------

/// Inputs that uniquely determine the pixels of a single panel.
///
/// Held as borrows where possible — `panel_hash` runs thousands of times in a
/// big batch render and should not force a clone on every call. `Vec` is used
/// for `character_loras` because it is the only field whose element count
/// varies per call; callers typically materialize it once per panel anyway.
///
/// Field semantics and on-the-wire serialization are documented on
/// [`panel_hash`].
pub struct PanelInput<'a> {
    /// Panel prompt copy, verbatim as it appears in `strips/NN-slug/proposal.md`.
    /// Canonicalized before hashing.
    pub prompt: &'a str,
    /// Style hash produced by [`global_style_hash`].
    pub style: PanelHash,
    /// LoRA manifest hashes for every character present in this panel. Order
    /// matters but is enforced by [`panel_hash`], not by the caller — see that
    /// function's docs.
    pub character_loras: Vec<PanelHash>,
    /// Random seed for this panel, declared in the strip proposal.
    pub seed: u64,
    /// Base model identifier, e.g. `"flux1-schnell-fp8.safetensors"` or its
    /// SHA-256. Canonicalized before hashing (so trivial whitespace diffs in
    /// a manifest do not invalidate the cache).
    pub base_model: &'a str,
    /// Optional Qwen-Image model identifier. `Some(_)` only when the panel
    /// has a title plate. Canonicalized before hashing.
    pub qwen_model: Option<&'a str>,
    /// Cache schema version. Bumping this invalidates the world on purpose.
    pub schema_version: u32,
}

/// Compute the panel hash. Mutating any input produces a different hash;
/// identical input produces the same hash (determinism — tested below).
///
/// # Serialization format (fed to SHA-256, in order)
///
/// | Field | Bytes | Notes |
/// |---|---|---|
/// | `canonicalize(prompt)` | variable (UTF-8) | Human-edited text. |
/// | separator | 1 | ASCII `\n`. Disambiguates end-of-prompt from first LoRA. |
/// | `style.0` | 32 | Raw SHA-256 of the four style specs. |
/// | `character_loras.len()` | 4 | Little-endian `u32`. Length-prefixing makes
///     the list self-delimiting so a 2-LoRA panel and a 3-LoRA panel can
///     never produce the same preimage. |
/// | sorted `character_loras` | `32 * N` | Each LoRA hash is 32 raw bytes.
///     The list is sorted **inside** `panel_hash` by raw byte order so
///     callers cannot accidentally vary the hash by re-ordering characters.
///     Sorting by raw hash is stable and does not require the caller to
///     know the character-name convention. |
/// | `seed` | 8 | Little-endian `u64`. |
/// | separator | 1 | ASCII `\n`. Disambiguates seed bytes from the
///     variable-length model string that follows. |
/// | `canonicalize(base_model)` | variable (UTF-8) | |
/// | separator | 1 | ASCII `\n`. Disambiguates end-of-base-model from the
///     Qwen presence byte. |
/// | `qwen_model` presence | 1 | `0x00` for `None`, `0x01` for `Some`.
///     An explicit tag byte rather than a length prefix, because the absent
///     case is the common one and the tag byte keeps the preimage legible
///     when debugging. |
/// | `canonicalize(qwen_model)` | variable (UTF-8) | Only when
///     `qwen_model.is_some()`. |
/// | separator | 1 | ASCII `\n`. Present regardless of `qwen_model` so the
///     schema-version bytes live at a consistent relative position in the
///     `Some` and `None` preimages (distinguishable only by the tag byte). |
/// | `schema_version` | 4 | Little-endian `u32`. |
///
/// Every variable-length string is followed by a fixed single-byte
/// separator (`\n`); every variable-count list is length-prefixed. Together
/// these keep the preimage unambiguous — two different `(prompt, base_model)`
/// pairs can never hash-collide by choosing where the boundary lives.
///
/// Sorting `character_loras` inside this function (not at the call site)
/// mirrors the spec: *"character_lora_hashes[present] sorted by character
/// name"*. We sort by raw hash bytes instead of by name because the hash is
/// the value actually present — sorting by name would require threading the
/// name through, adding coupling for no correctness benefit.
///
/// `@trace spec:orchestrator, spec:character-loras`
/// `@Lesson S1-1300`
pub fn panel_hash(input: &PanelInput<'_>) -> PanelHash {
    let mut hasher = Sha256::new();

    // prompt (canonicalized) + separator
    hasher.update(canonicalize(input.prompt).as_bytes());
    hasher.update(b"\n");

    // style (32 raw bytes)
    hasher.update(input.style.as_bytes());

    // character_loras: length-prefixed, sorted by raw bytes
    let n = input.character_loras.len() as u32;
    hasher.update(n.to_le_bytes());
    // Sort a borrowed view without mutating the caller's Vec. Uses one
    // heap allocation of `N` pointers (32-byte arrays are Copy, so we sort
    // them directly); for a typical panel N ≤ 3 this is trivial.
    let mut loras: Vec<&[u8; 32]> = input.character_loras.iter().map(|h| h.as_bytes()).collect();
    loras.sort_unstable();
    for lora in &loras {
        hasher.update(**lora);
    }

    // seed (8 le bytes) + separator
    hasher.update(input.seed.to_le_bytes());
    hasher.update(b"\n");

    // base_model (canonicalized) + separator
    hasher.update(canonicalize(input.base_model).as_bytes());
    hasher.update(b"\n");

    // qwen_model: presence tag + optional canonicalized body + separator
    match input.qwen_model {
        Some(q) => {
            hasher.update([0x01u8]);
            hasher.update(canonicalize(q).as_bytes());
        }
        None => {
            hasher.update([0x00u8]);
        }
    }
    hasher.update(b"\n");

    // schema_version (4 le bytes)
    hasher.update(input.schema_version.to_le_bytes());

    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    PanelHash::from_bytes(out)
}

// ---------------------------------------------------------------------------
// LoRA manifest hash
// ---------------------------------------------------------------------------

/// Hash a LoRA manifest JSON body. Canonicalization runs over the raw JSON
/// string: whitespace-only diffs in `characters/<name>/lora-manifest.json`
/// must not invalidate every panel featuring that character.
///
/// Callers in `tt-lora` produce the manifest JSON and pass it here; the
/// returned hash participates in [`panel_hash`] via
/// [`PanelInput::character_loras`].
///
/// `@trace spec:character-loras`
#[inline]
pub fn lora_manifest_hash(manifest_json: &str) -> PanelHash {
    sha256_of_string(manifest_json)
}

// ---------------------------------------------------------------------------
// Commit hash helpers (consumed by tt-calmecac-indexer)
// ---------------------------------------------------------------------------

/// Return the standard short-SHA form — the first 7 hex characters.
///
/// Assumes the input is a valid 40-char lowercase commit SHA. Callers can
/// validate with [`commit_is_canonical`] first. For shorter inputs the
/// function returns the input unchanged — a deliberate leniency so that log
/// output never panics on corrupt data.
///
/// `@trace spec:calmecac`
#[inline]
pub fn short_commit(full: &str) -> &str {
    if full.len() < 7 {
        full
    } else {
        // Commit SHAs are ASCII hex, so byte slice indexing is char-safe.
        &full[..7]
    }
}

/// True iff the argument is 40 lowercase hex characters — a canonical git
/// commit SHA. Used by `tt-calmecac-indexer` to reject garbled entries
/// before they leak into `calmecac-index.json`.
///
/// `@trace spec:calmecac`
pub fn commit_is_canonical(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- canonicalize -----------------------------------------------------

    #[test]
    fn canonicalize_is_idempotent() {
        let inputs = [
            "",
            "hello",
            "line1\nline2\n",
            "line1\r\nline2\r\n",
            "  padded  \n",
            "a\n\n\n\nb",
            "\u{0061}\u{0304}ni", // decomposed "āni"
            "Tlatoāni Tales",
            "line1\n\n\nline2\n\n\n\nline3",
        ];
        for input in inputs {
            let once = canonicalize(input);
            let twice = canonicalize(&once);
            assert_eq!(once, twice, "canonicalize not idempotent for {input:?}");
        }
    }

    #[test]
    fn canonicalize_converts_crlf_and_cr_to_lf() {
        assert_eq!(canonicalize("a\r\nb"), "a\nb");
        assert_eq!(canonicalize("a\rb"), "a\nb");
        // Mixed: CRLF then lone CR.
        assert_eq!(canonicalize("a\r\nb\rc"), "a\nb\nc");
    }

    #[test]
    fn canonicalize_trims_outer_whitespace() {
        assert_eq!(canonicalize("   hello   "), "hello");
        assert_eq!(canonicalize("\n\n\nhello\n\n\n"), "hello");
        assert_eq!(canonicalize("\t\thello world\t\t"), "hello world");
    }

    #[test]
    fn canonicalize_collapses_blank_line_runs() {
        assert_eq!(canonicalize("a\n\n\n\nb"), "a\n\nb");
        assert_eq!(canonicalize("a\n\nb\n\n\nc"), "a\n\nb\n\nc");
        // A single blank line stays a single blank line.
        assert_eq!(canonicalize("a\n\nb"), "a\n\nb");
    }

    #[test]
    fn canonicalize_composes_nfc() {
        // U+0061 LATIN SMALL LETTER A + U+0304 COMBINING MACRON → U+0101 ā.
        let decomposed = "Tlatoa\u{0304}ni";
        let composed = "Tlatoāni";
        assert_eq!(canonicalize(decomposed), composed);
        // Hashes must match too — the whole point of NFC in this crate.
        assert_eq!(
            sha256_of_string(decomposed),
            sha256_of_string(composed),
            "NFC canonicalization did not align hashes",
        );
    }

    #[test]
    fn canonicalize_empty_and_whitespace_only() {
        assert_eq!(canonicalize(""), "");
        assert_eq!(canonicalize("   "), "");
        assert_eq!(canonicalize("\n\n\n"), "");
        assert_eq!(canonicalize("\r\n\r\n"), "");
    }

    // --- SHA-256 helpers --------------------------------------------------

    #[test]
    fn sha256_hex_known_vectors() {
        // RFC standard test vectors.
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        );
    }

    #[test]
    fn sha256_bytes_matches_hex() {
        let bytes = sha256_bytes(b"abc");
        assert_eq!(hex::encode(bytes), sha256_hex(b"abc"));
    }

    #[test]
    fn sha256_of_string_canonicalizes() {
        // Leading/trailing whitespace and CRLF must wash out.
        assert_eq!(
            sha256_of_string("hello"),
            sha256_of_string("  hello\r\n"),
        );
    }

    // --- global_style_hash -------------------------------------------------

    #[test]
    fn global_style_hash_is_deterministic() {
        let h1 = global_style_hash("style", "canon", "symbols", "traces");
        let h2 = global_style_hash("style", "canon", "symbols", "traces");
        assert_eq!(h1, h2);
    }

    #[test]
    fn global_style_hash_is_order_sensitive() {
        // The formula is order-sensitive by design: swapping two arguments
        // MUST change the hash. Callers are responsible for passing inputs
        // in the spec-defined order (alphabetical by spec name).
        let canonical = global_style_hash("style", "canon", "symbols", "traces");
        let swapped = global_style_hash("canon", "style", "symbols", "traces");
        assert_ne!(canonical, swapped);
    }

    #[test]
    fn global_style_hash_detects_spec_mutation() {
        // Mutating any one of the four inputs must change the result.
        let base = global_style_hash("a", "b", "c", "d");
        assert_ne!(base, global_style_hash("a!", "b", "c", "d"));
        assert_ne!(base, global_style_hash("a", "b!", "c", "d"));
        assert_ne!(base, global_style_hash("a", "b", "c!", "d"));
        assert_ne!(base, global_style_hash("a", "b", "c", "d!"));
    }

    // --- panel_hash -------------------------------------------------------

    fn sample_input<'a>(prompt: &'a str, seed: u64) -> PanelInput<'a> {
        PanelInput {
            prompt,
            style: PanelHash::from_bytes([1u8; 32]),
            character_loras: vec![
                PanelHash::from_bytes([2u8; 32]),
                PanelHash::from_bytes([3u8; 32]),
            ],
            seed,
            base_model: "flux1-schnell-fp8.safetensors",
            qwen_model: None,
            schema_version: 1,
        }
    }

    #[test]
    fn panel_hash_is_deterministic() {
        let a = sample_input("a prompt", 42);
        let b = sample_input("a prompt", 42);
        assert_eq!(panel_hash(&a), panel_hash(&b));
    }

    #[test]
    fn panel_hash_changes_on_prompt_edit() {
        let a = sample_input("a prompt", 42);
        let b = sample_input("a prompt!", 42);
        assert_ne!(panel_hash(&a), panel_hash(&b));
    }

    #[test]
    fn panel_hash_changes_on_seed_change() {
        let a = sample_input("a prompt", 42);
        let b = sample_input("a prompt", 43);
        assert_ne!(panel_hash(&a), panel_hash(&b));
    }

    #[test]
    fn panel_hash_changes_on_style_change() {
        let mut a = sample_input("p", 1);
        let mut b = sample_input("p", 1);
        a.style = PanelHash::from_bytes([0xAAu8; 32]);
        b.style = PanelHash::from_bytes([0xBBu8; 32]);
        assert_ne!(panel_hash(&a), panel_hash(&b));
    }

    #[test]
    fn panel_hash_is_lora_order_invariant() {
        // The formula sorts loras internally, so the caller cannot perturb
        // the hash by re-ordering characters in the panel.
        let mut a = sample_input("p", 1);
        let mut b = sample_input("p", 1);
        a.character_loras = vec![
            PanelHash::from_bytes([2u8; 32]),
            PanelHash::from_bytes([3u8; 32]),
        ];
        b.character_loras = vec![
            PanelHash::from_bytes([3u8; 32]),
            PanelHash::from_bytes([2u8; 32]),
        ];
        assert_eq!(panel_hash(&a), panel_hash(&b));
    }

    #[test]
    fn panel_hash_changes_on_lora_set_change() {
        let mut a = sample_input("p", 1);
        let mut b = sample_input("p", 1);
        a.character_loras = vec![PanelHash::from_bytes([2u8; 32])];
        b.character_loras = vec![
            PanelHash::from_bytes([2u8; 32]),
            PanelHash::from_bytes([3u8; 32]),
        ];
        // Different count → length prefix differs → hash differs.
        assert_ne!(panel_hash(&a), panel_hash(&b));
    }

    #[test]
    fn panel_hash_distinguishes_qwen_some_from_none() {
        let mut a = sample_input("p", 1);
        let mut b = sample_input("p", 1);
        a.qwen_model = None;
        b.qwen_model = Some("");
        // Empty-string Qwen is still a Some — tag byte differs — hash must
        // differ, proving the presence tag is load-bearing.
        assert_ne!(panel_hash(&a), panel_hash(&b));
    }

    #[test]
    fn panel_hash_changes_on_qwen_body_change() {
        let mut a = sample_input("p", 1);
        let mut b = sample_input("p", 1);
        a.qwen_model = Some("qwen-image-v1");
        b.qwen_model = Some("qwen-image-v2");
        assert_ne!(panel_hash(&a), panel_hash(&b));
    }

    #[test]
    fn panel_hash_changes_on_base_model_change() {
        let mut a = sample_input("p", 1);
        let mut b = sample_input("p", 1);
        a.base_model = "flux1-schnell-fp8.safetensors";
        b.base_model = "flux1-dev-fp8.safetensors";
        assert_ne!(panel_hash(&a), panel_hash(&b));
    }

    #[test]
    fn panel_hash_changes_on_schema_version_bump() {
        let mut a = sample_input("p", 1);
        let mut b = sample_input("p", 1);
        a.schema_version = 1;
        b.schema_version = 2;
        assert_ne!(panel_hash(&a), panel_hash(&b));
    }

    #[test]
    fn panel_hash_prompt_canonicalizes() {
        // Whitespace-only prompt edits must not bust the cache.
        let mut a = sample_input("hello", 1);
        let mut b = sample_input("  hello\r\n", 1);
        a.character_loras = vec![];
        b.character_loras = vec![];
        assert_eq!(panel_hash(&a), panel_hash(&b));
    }

    // --- lora_manifest_hash -----------------------------------------------

    #[test]
    fn lora_manifest_hash_round_trip() {
        let manifest = r#"{"character":"ocelotl","output":{"sha256":"abc"}}"#;
        let h1 = lora_manifest_hash(manifest);
        let h2 = lora_manifest_hash(manifest);
        assert_eq!(h1, h2);
        // Whitespace-only diffs in the JSON must not change the hash.
        let padded = format!("  {manifest}  \r\n");
        assert_eq!(h1, lora_manifest_hash(&padded));
        // A content change must.
        let mutated = r#"{"character":"ocelotl","output":{"sha256":"abd"}}"#;
        assert_ne!(h1, lora_manifest_hash(mutated));
    }

    // --- commit helpers ---------------------------------------------------

    #[test]
    fn short_commit_returns_first_seven() {
        assert_eq!(
            short_commit("abcdef0123456789abcdef0123456789abcdef01"),
            "abcdef0",
        );
    }

    #[test]
    fn short_commit_short_input_is_passthrough() {
        assert_eq!(short_commit("abc"), "abc");
        assert_eq!(short_commit(""), "");
        assert_eq!(short_commit("abcdef"), "abcdef");
        // Exactly 7 chars: returned verbatim.
        assert_eq!(short_commit("abcdef0"), "abcdef0");
    }

    #[test]
    fn commit_is_canonical_accepts_40_char_hex() {
        assert!(commit_is_canonical(
            "abcdef0123456789abcdef0123456789abcdef01"
        ));
        assert!(commit_is_canonical(
            "0000000000000000000000000000000000000000"
        ));
    }

    #[test]
    fn commit_is_canonical_rejects_everything_else() {
        // Wrong length.
        assert!(!commit_is_canonical(""));
        assert!(!commit_is_canonical("abc"));
        assert!(!commit_is_canonical(
            "abcdef0123456789abcdef0123456789abcdef0" // 39
        ));
        assert!(!commit_is_canonical(
            "abcdef0123456789abcdef0123456789abcdef012" // 41
        ));
        // Uppercase hex is non-canonical — git's lowercase-only convention.
        assert!(!commit_is_canonical(
            "ABCDEF0123456789ABCDEF0123456789ABCDEF01"
        ));
        // Non-hex character.
        assert!(!commit_is_canonical(
            "abcdef0123456789abcdef0123456789abcdefzz"
        ));
    }
}
