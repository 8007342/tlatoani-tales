//! Tlatoāni Tales — image composition.
//!
//! Loads three rendered panels, stitches them horizontally, composites the
//! Qwen-Image title plate top-left, draws the chrome trace+lesson plate
//! bottom-left and the episode plate bottom-right. Writes
//! `output/Tlatoāni_Tales_NN.png`.
//!
//! Governing spec: `openspec/specs/trace-plate/spec.md`,
//! `openspec/specs/style-bible/spec.md`.
//!
// @trace spec:trace-plate, spec:style-bible

use std::path::{Path, PathBuf};
use tt_core::{LessonId, SpecName, StripId, TtError};

/// Title plate declaration — content and placement knobs from the strip's
/// `proposal.md`.
#[derive(Debug, Clone)]
pub struct TitleSpec {
    pub display: String,
    pub float_right: bool,
    pub backing_scroll: bool,
}

/// Episode plate label, e.g. `"Tlatoāni Tales 11/15"`.
#[derive(Debug, Clone)]
pub struct EpisodeLabel(pub String);

/// Composite a strip's three rendered panels + three plates into a single
/// PNG. Returns the output path.
pub async fn composite_strip(
    _strip_id: StripId,
    _panels: &[PathBuf],
    _title: &TitleSpec,
    _lesson: &LessonId,
    _trace: &SpecName,
    _episode: &EpisodeLabel,
) -> Result<PathBuf, TtError> {
    unimplemented!("tt-compose composite_strip is scaffolded; real render lands in a later change")
}

/// Load a PNG into an `image::DynamicImage`. Thin wrapper — exposed so other
/// crates don't each pull in `image` directly.
pub fn load_png(_path: &Path) -> Result<image::DynamicImage, TtError> {
    unimplemented!("tt-compose load_png is scaffolded")
}
