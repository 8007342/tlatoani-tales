//! Tlatoāni Tales — METADATA.json emitter.
//!
//! Exactly the schema in `openspec/specs/trace-plate/spec.md` §METADATA
//! schema. The publishing site reads `plate_regions` verbatim to build the
//! HTML image-map; no pixel re-parsing. "No metadata, no ship."
//!
// @trace spec:trace-plate

use serde::{Deserialize, Serialize};
use std::path::Path;
use tt_core::TtError;

/// Pixel rectangle in the composited PNG's coordinate space (origin top-left).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// The bottom-left plate is two independent click regions — one for the
/// `@Lesson` line, one for the `@trace` line. The enclosing rectangle is
/// preserved for callers that want a single bounding box.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TraceLessonRegion {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub lesson_line: Rect,
    pub trace_line: Rect,
}

/// The three plate regions emitted on every strip's METADATA.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlateRegions {
    pub title: Rect,
    pub trace_lesson: TraceLessonRegion,
    pub episode: Rect,
}

/// Minimum information the emitter needs to write a strip's METADATA.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripInfo {
    pub strip: String,              // "TT NN/15"
    pub title: String,              // no brackets
    pub title_display: String,      // "[Volatile is dangerous]"
    pub title_render_model: String, // "Qwen-Image"
    pub title_float: String,        // "left" | "right"
    pub title_linkable: bool,

    pub lesson: String,
    pub lesson_display: String,
    pub lesson_search_url: String,
    pub lesson_spec_url: String,
    pub calmecac_lesson_url: String,

    pub trace_spec: String,
    pub trace_search_url: String,
    pub trace_spec_url: String,
    pub calmecac_spec_url: String,

    pub concepts_taught: Vec<String>,
    pub concepts_assumed: Vec<String>,
    pub reinforces_lessons: Vec<String>,

    pub alt_text: String,
    pub caption: String,
}

/// Write `output/Tlatoāni_Tales_NN.json` for a strip.
pub fn write_metadata(
    _strip: &StripInfo,
    _plate_regions: &PlateRegions,
    _out_path: &Path,
) -> Result<(), TtError> {
    unimplemented!("tt-metadata write_metadata is scaffolded; real emission lands in a later change")
}
