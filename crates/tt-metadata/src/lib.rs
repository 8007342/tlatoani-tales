//! Tlatoāni Tales — METADATA.json emitter.
//!
//! Writes `output/Tlatoāni_Tales_NN.json` exactly matching the schema in
//! `openspec/specs/trace-plate/spec.md` §METADATA schema. The publishing site
//! at `www.tlatoani-tales.com` reads `plate_regions` verbatim to build the
//! HTML image-map — no pixel re-parsing, no second source of truth.
//!
//! **No metadata, no ship.** Every rendered strip PNG has a sibling JSON here
//! or the orchestrator refuses to promote the run (see `orchestrator/spec.md`
//! §Invariants).
//!
//! Governing specs: `trace-plate/spec.md` (schema), `orchestrator/spec.md`
//! (emission point — step 6c of the render flow), `calmecac/spec.md` (the
//! downstream URL targets).
//!
// @trace spec:trace-plate, spec:orchestrator, spec:calmecac
// @Lesson S1-1500

use serde::{Deserialize, Serialize};
use std::path::Path;
use tt_core::{LessonId, SpecName, TtError};

// ---------------------------------------------------------------------------
// Plate regions — mirror of tt-compose's layout output
// ---------------------------------------------------------------------------
//
// The `trace-plate/spec.md` schema nests three pixel rectangles (two of which
// share a two-line split-region structure). Keeping the JSON shape
// self-contained in this crate decouples the on-disk schema from the
// `tt_compose::PlateRegions` in-memory layout type — which is still being
// built by a parallel agent. When `tt_compose::PlateRegions` exposes a
// `Serialize` form with an identical JSON surface, we can drop the mirror and
// reuse directly, but today this crate is the source of truth for the on-wire
// bytes. A `From<tt_compose::PlateRegions>` impl lives behind a cfg and a TODO
// below; no downstream feature flag required.

/// Pixel rectangle in the composited PNG's coordinate space (origin top-left).
///
/// Mirrors `tt_compose::Rect` exactly. The JSON shape
/// (`{"x": .., "y": .., "w": .., "h": ..}`) is load-bearing: the publishing
/// site's image-map builder consumes it verbatim (see
/// `trace-plate/spec.md` §Clickable semantics).
// @trace spec:trace-plate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// The bottom-left plate — two independent click regions (one for the
/// `@Lesson` line, one for the `@trace` line) wrapped in an enclosing
/// bounding box. Matches the schema's `trace_lesson` object verbatim.
// @trace spec:trace-plate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceLessonRegion {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub lesson_line: Rect,
    pub trace_line: Rect,
}

/// The three plate regions emitted on every strip's METADATA — title
/// (top-left), trace+lesson (bottom-left), episode (bottom-right).
///
/// This is the JSON-facing mirror of `tt_compose::PlateRegions`. If/when
/// `tt-compose` starts deriving `Serialize` on its own `PlateRegions` with
/// this exact shape, callers can pass that directly through a `From` impl —
/// see TODO below.
// @trace spec:trace-plate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlateRegionsJson {
    pub title: Rect,
    pub trace_lesson: TraceLessonRegion,
    pub episode: Rect,
}

// TODO (post parallel-agent merge): once tt_compose::PlateRegions derives
// Serialize and matches this layout byte-for-byte, replace this mirror with a
// `From<tt_compose::PlateRegions> for PlateRegionsJson` impl (or reuse the
// upstream type directly). Today tt-compose's scaffold does not export a
// `PlateRegions` type, so this crate owns the on-disk shape.

// ---------------------------------------------------------------------------
// URL templating
// ---------------------------------------------------------------------------

const REPO_BASE: &str = "https://github.com/8007342/tlatoani-tales";
const CALMECAC_BASE: &str = "https://calmecac.tlatoani-tales.com";

/// Percent-encode a single query-parameter value. We deliberately do not use
/// `url::form_urlencoded` here because we want the author-legible `+` for the
/// space in `@Lesson S1-NNN` searches to remain a literal `+` (which is what
/// GitHub's search UI produces) — but `@` and `:` must become `%40` / `%3A`
/// for the canonical form documented in `trace-plate/spec.md` §URL forms.
///
/// Rules: `A-Z a-z 0-9 - _ . ~` pass through unchanged (RFC 3986 unreserved).
/// Everything else is `%HH` uppercase hex. Space is rendered as `%20`; callers
/// that want GitHub's `+`-for-space form must substitute `+` themselves
/// (lessons and spec names never contain spaces, so this is moot here).
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        let keep = matches!(
            b,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~'
        );
        if keep {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// GitHub code-search URL for a lesson's `@Lesson` citations.
///
/// Example: `S1-1500-proof-by-self-reference` →
/// `https://github.com/8007342/tlatoani-tales/search?q=%40Lesson+S1-1500-proof-by-self-reference&type=code`.
///
/// The `%40` is the canonical encoding of `@` per `trace-plate/spec.md` §URL
/// forms; the `+` separator between `@Lesson` and the id matches GitHub's
/// search UI output. The full slug is used (not the short form) so unrelated
/// matches on bare short IDs like `S1-100` don't pollute the hit list.
// @trace spec:trace-plate
pub fn lesson_search_url(lesson: &LessonId) -> String {
    format!(
        "{REPO_BASE}/search?q=%40Lesson+{slug}&type=code",
        slug = percent_encode(lesson.as_str())
    )
}

/// GitHub blob URL for the lessons registry on `main`.
///
/// Points at the single registry (`openspec/specs/lessons/spec.md`) rather
/// than a per-lesson anchor, because lessons are registered rows in that spec
/// — the file is the ground truth, and the anchor is whatever GitHub renders.
pub fn lesson_spec_url(_lesson: &LessonId) -> String {
    format!("{REPO_BASE}/blob/main/openspec/specs/lessons/spec.md")
}

/// GitHub code-search URL for a spec's `@trace` citations.
///
/// Example: `tlatoāni-spelling` →
/// `https://github.com/8007342/tlatoani-tales/search?q=%40trace+spec%3Atlatoa%CC%84ni-spelling&type=code`
/// — `%40` for `@`, `%3A` for `:`, and UTF-8 bytes of the macron percent-escaped.
// @trace spec:trace-plate
pub fn trace_search_url(spec: &SpecName) -> String {
    format!(
        "{REPO_BASE}/search?q=%40trace+spec%3A{name}&type=code",
        name = percent_encode(spec.as_str())
    )
}

/// GitHub blob URL for the named spec's `spec.md`.
///
/// Spec names are kebab-case-over-UTF-8 (see `tt_core::SpecName`). In
/// practice every spec on disk sits under `openspec/specs/<name>/spec.md`;
/// if the name contains non-ASCII code points (e.g. `tlatoāni-spelling`),
/// GitHub handles the raw UTF-8 path fine and we pass it through unencoded
/// so the URL stays human-readable when pasted into a browser URL bar.
// @trace spec:trace-plate
pub fn trace_spec_url(spec: &SpecName) -> String {
    format!(
        "{REPO_BASE}/blob/main/openspec/specs/{name}/spec.md",
        name = spec.as_str()
    )
}

/// Calmecac lesson-view URL. Uses the **short** form (`S1-NNN`) because the
/// Calmecac routing contract (`calmecac/spec.md`) keys on the grep-friendly
/// short id, not on the slug.
// @trace spec:calmecac
pub fn calmecac_lesson_url(lesson: &LessonId) -> String {
    format!("{CALMECAC_BASE}/lesson/{}", lesson.short())
}

/// Calmecac spec-view URL. Passes the full (possibly UTF-8) spec name
/// through; Calmecac's backend handles route decoding.
// @trace spec:calmecac
pub fn calmecac_spec_url(spec: &SpecName) -> String {
    format!("{CALMECAC_BASE}/spec/{}", spec.as_str())
}

// ---------------------------------------------------------------------------
// Caption builder
// ---------------------------------------------------------------------------

/// Canonical strip caption. Matches the schema example verbatim:
///
/// ```text
/// Tlatoāni Tales NN/15 — [Title] — @Lesson S1-NNN / @trace spec:<name>
/// ```
///
/// `strip` is expected to carry the human form (e.g. `"TT 01/15"` or
/// `"Tlatoāni Tales 01/15"` — the caller chooses which renders in the caption
/// by passing the desired prefix; this function does not re-wrap). The author
/// directive from `trace-plate/spec.md` §Episode plate prefers
/// `"Tlatoāni Tales NN/TOTAL"` (with the macron), and the examples in the
/// schema follow that form — hence the en-dash joins.
// @trace spec:trace-plate
pub fn build_caption(strip: &str, title: &str, lesson: &LessonId, trace: &SpecName) -> String {
    format!(
        "{strip} — {title} — @Lesson {lesson_short} / @trace spec:{trace_name}",
        lesson_short = lesson.short(),
        trace_name = trace.as_str()
    )
}

// ---------------------------------------------------------------------------
// StripMetadata — the on-disk JSON document
// ---------------------------------------------------------------------------

/// Exactly the schema in `trace-plate/spec.md` §METADATA schema.
///
/// Field order here matches the spec's JSONC example so the emitted file
/// reads like the spec (serde preserves struct field order in `serde_json`
/// output). No field is optional: the schema lists every one as required, and
/// "no metadata, no ship" means a missing field is a canon failure upstream
/// (enforced by the VLM checks `plate.*` in `visual-qa-loop/spec.md`).
///
/// Two fields that *could* look optional but are not:
/// - `reinforces_lessons`: always present, may be an empty array.
/// - `concepts_assumed`: always present, may be an empty array.
///
/// Typed fields (`lesson: LessonId`, `trace_spec: SpecName`) serialize as
/// their canonical string forms — the readable JSON shape matches the schema
/// exactly, with the validation handled at construction time by `tt_core`.
// @trace spec:trace-plate, spec:orchestrator
// @Lesson S1-1500
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StripMetadata {
    /// `"Tlatoāni Tales NN/15"` — the human strip identifier. The `/15`
    /// denominator is the TOTAL convergence signal for Season 1.
    pub strip: String,

    /// Display name without brackets (e.g. `"Volatile is dangerous"`).
    pub title: String,

    /// Exactly as rendered in-panel, with brackets
    /// (e.g. `"[Volatile is dangerous]"`).
    pub title_display: String,

    /// `"Qwen-Image"` when the stylized title plate was used, `"FLUX-only"`
    /// when a strip shipped pre-title-plate. The schema's example is
    /// `"Qwen-Image"`; the string is authoritative.
    pub title_render_model: String,

    /// `true` when the author declared `title_float: right` in `proposal.md`.
    /// `false` (default) places the title top-left.
    pub title_float: bool,

    /// Per-strip opt-in. When `true`, the title region links to the Calmecac
    /// lesson view (see `trace-plate/spec.md` §Clickable semantics).
    pub title_linkable: bool,

    pub lesson: LessonId,
    pub lesson_display: String,
    pub lesson_search_url: String,
    pub lesson_spec_url: String,

    pub trace_spec: SpecName,
    pub trace_search_url: String,
    pub trace_spec_url: String,

    /// Concepts **introduced** by this strip (per
    /// `lessons/spec.md` coverage list). At least one.
    pub concepts_taught: Vec<String>,

    /// Concepts **assumed** (previously introduced; referenced without
    /// re-teaching). May be empty.
    pub concepts_assumed: Vec<String>,

    /// Other lesson IDs this strip echoes. May be empty — but always
    /// present as an array.
    pub reinforces_lessons: Vec<LessonId>,

    pub plate_regions: PlateRegionsJson,

    pub calmecac_lesson_url: String,
    pub calmecac_spec_url: String,

    /// Accessible description of all three panels, naming all three plates.
    /// Required (`plate.*` QA checks assume non-empty here).
    pub alt_text: String,

    /// Canonical caption from [`build_caption`].
    pub caption: String,
}

// ---------------------------------------------------------------------------
// Emission / reading
// ---------------------------------------------------------------------------

/// Write a strip's `METADATA.json` to `path`.
///
/// Pretty-printed (2-space indent per the schema example) and UTF-8 — the
/// strip format's `Tlatoāni Tales` name round-trips with the macron preserved.
/// Uses `tokio::fs::write` so the orchestrator's event loop is never blocked
/// (honors the async-non-blocking hard requirement in the cross-project
/// conventions).
// @trace spec:trace-plate, spec:orchestrator
pub async fn write_metadata(meta: &StripMetadata, path: &Path) -> Result<(), TtError> {
    let mut buf = Vec::with_capacity(1024);
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    meta.serialize(&mut ser)
        .map_err(|e| TtError::Parse(format!("metadata serialize: {e}")))?;
    // Trailing newline — POSIX text-file convention, matches every other
    // artefact emitted by the orchestrator.
    buf.push(b'\n');
    tokio::fs::write(path, &buf).await?;
    Ok(())
}

/// Read a strip's `METADATA.json` back from disk.
///
/// Used by `tt-calmecac-indexer` (future) to rebuild the convergence graph
/// without re-running ComfyUI, and by `tt-lint` during `verify` to assert the
/// artefact still parses after a spec mutation. Strict parsing — any
/// field missing is a canon failure (exit 10 via `TtError::Parse`).
// @trace spec:trace-plate, spec:calmecac
pub async fn read_metadata(path: &Path) -> Result<StripMetadata, TtError> {
    let bytes = tokio::fs::read(path).await?;
    serde_json::from_slice(&bytes).map_err(|e| TtError::Parse(format!("metadata parse: {e}")))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lesson() -> LessonId {
        LessonId::new("S1-1500-proof-by-self-reference").unwrap()
    }

    fn sample_spec() -> SpecName {
        SpecName::new("trace-plate").unwrap()
    }

    fn sample_regions() -> PlateRegionsJson {
        PlateRegionsJson {
            title: Rect { x: 0, y: 0, w: 620, h: 140 },
            trace_lesson: TraceLessonRegion {
                x: 0,
                y: 860,
                w: 780,
                h: 180,
                lesson_line: Rect { x: 12, y: 872, w: 760, h: 78 },
                trace_line: Rect { x: 12, y: 958, w: 760, h: 72 },
            },
            episode: Rect { x: 1180, y: 860, w: 720, h: 180 },
        }
    }

    fn sample_meta() -> StripMetadata {
        let lesson = sample_lesson();
        let spec = sample_spec();
        StripMetadata {
            strip: "Tlatoāni Tales 15/15".into(),
            title: "Proof by self-reference".into(),
            title_display: "[Proof by self-reference]".into(),
            title_render_model: "Qwen-Image".into(),
            title_float: false,
            title_linkable: true,
            lesson_display: "Proof by self-reference".into(),
            lesson_search_url: lesson_search_url(&lesson),
            lesson_spec_url: lesson_spec_url(&lesson),
            trace_search_url: trace_search_url(&spec),
            trace_spec_url: trace_spec_url(&spec),
            concepts_taught: vec!["C15".into()],
            concepts_assumed: vec!["C13".into(), "C14".into()],
            reinforces_lessons: vec![],
            plate_regions: sample_regions(),
            calmecac_lesson_url: calmecac_lesson_url(&lesson),
            calmecac_spec_url: calmecac_spec_url(&spec),
            alt_text: "Three panels. Top-left title plate reads [Proof by self-reference]. \
                       Bottom-left plate reads @Lesson S1-1500 and @trace spec:trace-plate. \
                       Bottom-right plate reads Tlatoāni Tales 15/15."
                .into(),
            caption: build_caption(
                "Tlatoāni Tales 15/15",
                "[Proof by self-reference]",
                &lesson,
                &spec,
            ),
            lesson,
            trace_spec: spec,
        }
    }

    // -- URL templates ----------------------------------------------------

    #[test]
    fn lesson_search_url_encodes_at_sign() {
        let url = lesson_search_url(&sample_lesson());
        assert_eq!(
            url,
            "https://github.com/8007342/tlatoani-tales/search?\
             q=%40Lesson+S1-1500-proof-by-self-reference&type=code"
        );
        assert!(url.contains("%40Lesson"));
        assert!(url.contains("&type=code"));
    }

    #[test]
    fn trace_search_url_encodes_at_and_colon() {
        let url = trace_search_url(&sample_spec());
        assert_eq!(
            url,
            "https://github.com/8007342/tlatoani-tales/search?\
             q=%40trace+spec%3Atrace-plate&type=code"
        );
        assert!(url.contains("%40trace"));
        assert!(url.contains("spec%3Atrace-plate"));
    }

    #[test]
    fn trace_search_url_encodes_macron_bytes() {
        // tlatoāni-spelling is a real spec name — the macron must be
        // percent-encoded in the URL. `ā` in UTF-8 is 0xC4 0x81.
        let spec = SpecName::new("tlatoāni-spelling").unwrap();
        let url = trace_search_url(&spec);
        assert!(
            url.contains("%C4%81"),
            "expected macron bytes %C4%81 in URL, got {url}"
        );
        assert!(url.starts_with(
            "https://github.com/8007342/tlatoani-tales/search?q=%40trace+spec%3Atlato"
        ));
    }

    #[test]
    fn lesson_spec_url_points_at_registry() {
        assert_eq!(
            lesson_spec_url(&sample_lesson()),
            "https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/lessons/spec.md"
        );
    }

    #[test]
    fn trace_spec_url_points_at_named_spec() {
        assert_eq!(
            trace_spec_url(&sample_spec()),
            "https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/trace-plate/spec.md"
        );
    }

    #[test]
    fn calmecac_lesson_url_uses_short_id() {
        assert_eq!(
            calmecac_lesson_url(&sample_lesson()),
            "https://calmecac.tlatoani-tales.com/lesson/S1-1500"
        );
    }

    #[test]
    fn calmecac_spec_url_uses_raw_name() {
        assert_eq!(
            calmecac_spec_url(&sample_spec()),
            "https://calmecac.tlatoani-tales.com/spec/trace-plate"
        );
    }

    #[test]
    fn repo_and_calmecac_bases_are_ascii() {
        // TB01/TB02 — domain and repo URLs stay ASCII.
        assert!(REPO_BASE.is_ascii());
        assert!(CALMECAC_BASE.is_ascii());
    }

    // -- Caption ----------------------------------------------------------

    #[test]
    fn caption_matches_spec_format() {
        let caption = build_caption(
            "Tlatoāni Tales 01/15",
            "[Volatile is dangerous]",
            &LessonId::new("S1-100-volatile-is-dangerous").unwrap(),
            &SpecName::new("concept-curriculum").unwrap(),
        );
        assert_eq!(
            caption,
            "Tlatoāni Tales 01/15 — [Volatile is dangerous] — \
             @Lesson S1-100 / @trace spec:concept-curriculum"
        );
        // The `/15` TOTAL denominator is the Season-1 convergence signal.
        assert!(caption.contains("Tlatoāni Tales 01/15"));
    }

    #[test]
    fn caption_preserves_macron() {
        let caption = build_caption(
            "Tlatoāni Tales 11/15",
            "[Shape has meaning]",
            &LessonId::new("S1-1100-shape-has-meaning").unwrap(),
            &SpecName::new("visual-qa-loop").unwrap(),
        );
        // Macron survives through the format path.
        assert!(caption.contains("Tlatoāni"));
        // The short lesson form — not the slug — is in the caption.
        assert!(caption.contains("@Lesson S1-1100 /"));
        assert!(!caption.contains("shape-has-meaning"));
    }

    // -- Strip field format (TOTAL convergence) ---------------------------

    #[test]
    fn strip_field_carries_total_convergence_signal() {
        let meta = sample_meta();
        // The "NN/15" denominator communicates nearness-to-convergence —
        // per trace-plate/spec.md §Episode plate.
        assert!(meta.strip.ends_with("/15"));
        assert!(meta.strip.starts_with("Tlatoāni Tales "));
    }

    #[test]
    fn strip_accepts_tt_short_prefix() {
        // The schema shows both "TT NN/15" and "Tlatoāni Tales NN/15" in
        // different fields; callers pick. Here we document that shape.
        let meta = StripMetadata {
            strip: "TT 10/15".into(),
            ..sample_meta()
        };
        assert!(meta.strip.starts_with("TT "));
        assert!(meta.strip.ends_with("/15"));
    }

    // -- Pretty-print & round-trip ---------------------------------------

    #[tokio::test]
    async fn json_roundtrip_via_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Tlatoāni_Tales_15.json");
        let original = sample_meta();
        write_metadata(&original, &path).await.unwrap();
        let reloaded = read_metadata(&path).await.unwrap();
        assert_eq!(original, reloaded);
    }

    #[tokio::test]
    async fn pretty_print_uses_two_space_indent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Tlatoāni_Tales_15.json");
        let meta = sample_meta();
        write_metadata(&meta, &path).await.unwrap();
        let text = tokio::fs::read_to_string(&path).await.unwrap();
        // Opening brace on line 1, two-space indented field on line 2.
        assert!(text.starts_with("{\n  \""), "expected 2-space indent, got:\n{text}");
        // Trailing newline.
        assert!(text.ends_with("\n"));
        // Human can read the file — schema-shaped keys are present.
        for key in [
            "\"strip\"",
            "\"title\"",
            "\"title_display\"",
            "\"title_render_model\"",
            "\"title_float\"",
            "\"title_linkable\"",
            "\"lesson\"",
            "\"lesson_search_url\"",
            "\"trace_spec\"",
            "\"plate_regions\"",
            "\"calmecac_lesson_url\"",
            "\"calmecac_spec_url\"",
            "\"alt_text\"",
            "\"caption\"",
        ] {
            assert!(text.contains(key), "missing key {key} in emitted JSON");
        }
        // Macron preserved in the UTF-8 file.
        assert!(text.contains("Tlatoāni"));
    }

    #[test]
    fn plate_regions_serialize_matches_schema_shape() {
        let regions = sample_regions();
        let v = serde_json::to_value(regions).unwrap();
        // trace_lesson has a nested lesson_line and trace_line.
        assert!(v["title"]["x"].is_number());
        assert!(v["trace_lesson"]["lesson_line"]["x"].is_number());
        assert!(v["trace_lesson"]["trace_line"]["w"].is_number());
        assert!(v["episode"]["h"].is_number());
    }

    // -- percent_encode ---------------------------------------------------

    #[test]
    fn percent_encode_handles_reserved_chars() {
        assert_eq!(percent_encode("S1-1500"), "S1-1500");
        assert_eq!(percent_encode("@"), "%40");
        assert_eq!(percent_encode(":"), "%3A");
        assert_eq!(percent_encode("a-b_c.d~e"), "a-b_c.d~e");
        // UTF-8 macron → two bytes, both encoded.
        assert_eq!(percent_encode("ā"), "%C4%81");
    }
}
