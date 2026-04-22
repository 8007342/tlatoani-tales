//! Tlatoāni Tales — image composition.
//!
//! Loads three rendered panels, stitches them horizontally onto a warm-paper
//! canvas, then draws the three plates declared by `trace-plate/spec.md`:
//!
//! | Plate                 | Position      | Content                                                    |
//! |-----------------------|--------------|------------------------------------------------------------|
//! | Title                 | top-left *  | `[<Lesson display>]` — stylized face, optional cream scroll|
//! | Trace + Lesson        | bottom-left  | line 1 `[@Lesson S1-NNN]`, line 2 `[@trace spec:<name>]`  |
//! | Episode               | bottom-right | `Tlatoāni Tales NN/TOTAL`                                  |
//!
//! \* title MAY float top-right when `TitleSpec::float_right = true`.
//!
//! The returned [`ComposeResult`] carries the exact pixel `Rect` of every
//! plate — these feed the `tt-metadata` emitter so the published image-map on
//! `tlatoani-tales.com` does not need to re-parse pixels.
//!
//! Governing specs: `openspec/specs/orchestrator/spec.md`,
//! `openspec/specs/trace-plate/spec.md`, `openspec/specs/style-bible/spec.md`.
//!
//! Font handling: fonts live under `{project_root}/assets/fonts/` and are
//! read at runtime (NOT baked in with `include_bytes!`) so the crate compiles
//! cleanly on hosts where the author has not yet downloaded the TTFs. See
//! `assets/fonts/README.md` for the expected filenames.
//!
// @trace spec:orchestrator, spec:trace-plate, spec:style-bible
// @Lesson S1-1000, @Lesson S1-1500

use ab_glyph::{FontVec, PxScale};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut, text_size};
use std::path::{Path, PathBuf};
use tt_core::{project_root, LessonId, SpecName, StripId, TtError};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Pixel rectangle in the composited PNG's coordinate space (origin top-left).
///
/// Matches the `plate_regions.*` schema emitted by `tt-metadata` per
/// `trace-plate/spec.md` §METADATA schema.
// @trace spec:trace-plate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    /// Inclusive top/left, exclusive bottom/right — standard half-open rect.
    /// Used by the Calmecac image-map builder to resolve click coordinates.
    pub fn contains_point(&self, px: u32, py: u32) -> bool {
        px >= self.x && py >= self.y && px < self.x + self.w && py < self.y + self.h
    }
}

/// Exact pixel regions of the three plates, returned to the caller so the
/// metadata emitter and the Calmecac image-map builder never re-parse pixels.
// @trace spec:trace-plate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlateRegions {
    pub title: Rect,
    pub trace_lesson: TraceLessonRegions,
    pub episode: Rect,
}

/// The bottom-left plate is two independent click targets stacked on one
/// scroll. `outer` is the whole plate, `lesson_line` and `trace_line` are the
/// two sub-rects — the image-map uses the sub-rects for per-line clicks.
// @trace spec:trace-plate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceLessonRegions {
    pub outer: Rect,
    pub lesson_line: Rect,
    pub trace_line: Rect,
}

/// Whether the title plate sits on a cream scroll backing or floats as pure
/// stylized text. Default is `None` (no chrome) per `trace-plate/spec.md`.
// @trace spec:trace-plate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleBacking {
    /// No backing — the stylized title floats over the panel pixels.
    None,
    /// Cream scroll backing (paper-color rounded rectangle) when a busy
    /// panel would otherwise eat legibility. Author opts in via
    /// `title_backing: scroll` in `proposal.md`.
    Scroll,
}

/// Declaration of the top-left (or top-right) title plate.
// @trace spec:trace-plate
#[derive(Debug, Clone, Copy)]
pub struct TitleSpec<'a> {
    /// The lesson's display name, bracketed exactly as rendered
    /// (e.g. `"[Volatile is dangerous]"`).
    pub display: &'a str,
    /// When `true`, render top-right instead of top-left (`title_float: right`).
    pub float_right: bool,
    /// Whether a cream scroll backs the title text.
    pub backing: TitleBacking,
}

/// Everything [`composite_strip`] needs in order to produce one final strip.
// @trace spec:orchestrator, spec:trace-plate
#[derive(Debug, Clone)]
pub struct StripInput<'a> {
    pub strip_id: StripId,
    /// e.g. `15` for Season 1 — used in the episode plate denominator.
    pub total_strips: u16,
    /// Three already-rendered panel PNGs, left-to-right.
    pub panels: [PathBuf; 3],
    pub title: TitleSpec<'a>,
    /// `@Lesson <Sn-NNN>` — bottom-left line 1.
    pub lesson: &'a LessonId,
    /// `@trace spec:<name>` — bottom-left line 2.
    pub trace_spec: &'a SpecName,
}

/// What [`composite_strip`] writes and where.
// @trace spec:orchestrator
#[derive(Debug, Clone)]
pub struct ComposeResult {
    pub png_path: PathBuf,
    pub plate_regions: PlateRegions,
}

/// A pair of `FontVec`s loaded once at orchestrator startup.
///
/// `chrome` backs the trace+lesson and episode plates; `title` backs the
/// top-left title. If the project has not yet committed the stylized title
/// face, [`load_embedded`] reuses the chrome font for `title` and still
/// produces a legible (if less expressive) strip.
// @trace spec:orchestrator, spec:trace-plate
pub struct FontSet {
    pub chrome: FontVec,
    pub title: FontVec,
}

impl std::fmt::Debug for FontSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontSet")
            .field("chrome", &"<FontVec>")
            .field("title", &"<FontVec>")
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Canvas constants — style-bible / trace-plate numerics
// ---------------------------------------------------------------------------

/// Composite canvas width — 1800 px, chosen for ~2:1 aspect with each of the
/// three panels landing on a round 600 px inner width after gutters.
// @trace spec:style-bible
pub const CANVAS_W: u32 = 1800;
/// Composite canvas height — 900 px → overall 2:1 aspect.
// @trace spec:style-bible
pub const CANVAS_H: u32 = 900;
/// Gutter width between adjacent panels. Style-bible: "thin gutters".
// @trace spec:style-bible
pub const GUTTER: u32 = 12;

/// Paper background — `#F4E9D3` per palette anchors.
// @trace spec:style-bible
pub const PAPER: Rgba<u8> = Rgba([0xF4, 0xE9, 0xD3, 0xFF]);
/// Ink color — `#2B2420`.
// @trace spec:style-bible
pub const INK: Rgba<u8> = Rgba([0x2B, 0x24, 0x20, 0xFF]);
/// Cream scroll color for plate backings — paper with slight lift.
pub const SCROLL: Rgba<u8> = Rgba([0xF4, 0xE9, 0xD3, 0xE8]);
/// Scroll edge ink for subtle border.
pub const SCROLL_EDGE: Rgba<u8> = Rgba([0x2B, 0x24, 0x20, 0x40]);

// Plate sizing, in canvas pixels. The `plate_regions` example in
// `trace-plate/spec.md` uses 620×140 for title, ~780×180 for trace_lesson,
// ~720×180 for episode at 1920×1080 scale; these values are chosen to
// respect that shape at 1800×900.
const TITLE_W: u32 = 620;
const TITLE_H: u32 = 140;
const PLATE_BOTTOM_H: u32 = 180;
const TRACE_PLATE_W: u32 = 780;
const EPISODE_PLATE_W: u32 = 720;
const PLATE_INSET: u32 = 12;

// ---------------------------------------------------------------------------
// FontSet — runtime font loading with a helpful Infra error
// ---------------------------------------------------------------------------

/// Load the canonical chrome + title fonts from `{project_root}/assets/fonts/`.
///
/// Errors with [`TtError::Infra`] if the chrome font is missing, pointing the
/// author at `assets/fonts/README.md`. If only the title font is missing, the
/// chrome font is cloned into the title slot so the strip still renders.
// @trace spec:orchestrator, spec:trace-plate
pub fn load_embedded() -> Result<FontSet, TtError> {
    let root = project_root();
    load_from_fonts_dir(&root.join("assets").join("fonts"))
}

/// Same as [`load_embedded`] but reads from an explicit directory. Useful
/// for tests and for callers who ship fonts alongside a distributed binary.
pub fn load_from_fonts_dir(dir: &Path) -> Result<FontSet, TtError> {
    let chrome_path = dir.join("atkinson-hyperlegible-regular.ttf");
    let title_path = dir.join("title-stylized-regular.ttf");

    let chrome_bytes = std::fs::read(&chrome_path).map_err(|_| {
        TtError::Infra(format!(
            "chrome font missing: expected `{}`; see `assets/fonts/README.md` — \
             download Atkinson Hyperlegible from https://brailleinstitute.org/freefont \
             and commit it to the repo",
            chrome_path.display()
        ))
    })?;
    let chrome = FontVec::try_from_vec(chrome_bytes).map_err(|e| {
        TtError::Infra(format!(
            "chrome font at `{}` failed to parse as TTF: {e}",
            chrome_path.display()
        ))
    })?;

    let title = match std::fs::read(&title_path) {
        Ok(bytes) => FontVec::try_from_vec(bytes).map_err(|e| {
            TtError::Infra(format!(
                "title font at `{}` failed to parse as TTF: {e}",
                title_path.display()
            ))
        })?,
        Err(_) => {
            // Title font is optional per the asset README — reuse chrome.
            // We re-read the chrome bytes so we own an independent FontVec.
            let bytes = std::fs::read(&chrome_path).map_err(TtError::Io)?;
            FontVec::try_from_vec(bytes).map_err(|e| {
                TtError::Infra(format!("chrome font re-parse failed: {e}"))
            })?
        }
    };

    Ok(FontSet { chrome, title })
}

// ---------------------------------------------------------------------------
// Geometry — pure functions, heavily tested
// ---------------------------------------------------------------------------

/// Where the three panels land inside the canvas. Panels are laid
/// horizontally with `GUTTER` spacing; panel height is the full canvas
/// height.
// @trace spec:style-bible
pub fn panel_rects(canvas_w: u32, canvas_h: u32) -> [Rect; 3] {
    // Three equal-width panels, separated by two gutters.
    let total_gutter = GUTTER * 2;
    let panel_w = (canvas_w - total_gutter) / 3;
    let y = 0;
    let h = canvas_h;
    [
        Rect { x: 0, y, w: panel_w, h },
        Rect {
            x: panel_w + GUTTER,
            y,
            w: panel_w,
            h,
        },
        Rect {
            x: (panel_w + GUTTER) * 2,
            y,
            w: canvas_w - (panel_w + GUTTER) * 2,
            h,
        },
    ]
}

/// Plate regions for a given canvas + title placement. The bottom plates
/// overlap all of panel 1/3 and ~12% of panel 2 per style-bible, mirrored.
// @trace spec:trace-plate, spec:style-bible
pub fn compute_plate_regions(canvas_w: u32, canvas_h: u32, title_float_right: bool) -> PlateRegions {
    let title_x = if title_float_right {
        canvas_w.saturating_sub(TITLE_W)
    } else {
        0
    };
    let title = Rect {
        x: title_x,
        y: 0,
        w: TITLE_W,
        h: TITLE_H,
    };

    let bottom_y = canvas_h - PLATE_BOTTOM_H;
    let outer = Rect {
        x: 0,
        y: bottom_y,
        w: TRACE_PLATE_W,
        h: PLATE_BOTTOM_H,
    };
    // Two equal-height inset lines — lesson on top, trace below.
    let line_h = (PLATE_BOTTOM_H - PLATE_INSET * 3) / 2;
    let lesson_line = Rect {
        x: outer.x + PLATE_INSET,
        y: outer.y + PLATE_INSET,
        w: outer.w - PLATE_INSET * 2,
        h: line_h,
    };
    let trace_line = Rect {
        x: outer.x + PLATE_INSET,
        y: lesson_line.y + lesson_line.h + PLATE_INSET,
        w: outer.w - PLATE_INSET * 2,
        h: line_h,
    };

    let episode = Rect {
        x: canvas_w - EPISODE_PLATE_W,
        y: bottom_y,
        w: EPISODE_PLATE_W,
        h: PLATE_BOTTOM_H,
    };

    PlateRegions {
        title,
        trace_lesson: TraceLessonRegions {
            outer,
            lesson_line,
            trace_line,
        },
        episode,
    }
}

// ---------------------------------------------------------------------------
// Drawing helpers
// ---------------------------------------------------------------------------

fn to_imageproc_rect(r: Rect) -> imageproc::rect::Rect {
    imageproc::rect::Rect::at(r.x as i32, r.y as i32).of_size(r.w.max(1), r.h.max(1))
}

/// Fill a rectangle and then sketch a faint 1-px inset frame — the
/// project's "rounded scroll" is rendered as a straight rectangle for now
/// (honest MVP; the rounded-corner mask is a later enhancement tracked in
/// `style-bible`). A fully SVG-driven scroll would require a vector
/// rasterizer; `imageproc` does not ship one, and adding `resvg` for two
/// pixels of rounding is not worth the dependency cost today.
// @trace spec:style-bible
fn draw_plate_background(canvas: &mut RgbaImage, r: Rect) {
    draw_filled_rect_mut(canvas, to_imageproc_rect(r), SCROLL);
    // Faint single-pixel frame on top/bottom/left/right to hint at the scroll.
    let top = Rect { x: r.x, y: r.y, w: r.w, h: 1 };
    let bot = Rect { x: r.x, y: r.y + r.h - 1, w: r.w, h: 1 };
    let left = Rect { x: r.x, y: r.y, w: 1, h: r.h };
    let right = Rect { x: r.x + r.w - 1, y: r.y, w: 1, h: r.h };
    for edge in [top, bot, left, right] {
        draw_filled_rect_mut(canvas, to_imageproc_rect(edge), SCROLL_EDGE);
    }
}

/// Draw `text` centered (horizontally + vertically) inside `r` at the given
/// `PxScale`, using the provided `font`. Falls back to top-left alignment
/// when the text is wider than the rect (we do not wrap — wrap/truncation
/// is the caller's responsibility).
fn draw_text_in_rect(
    canvas: &mut RgbaImage,
    r: Rect,
    text: &str,
    font: &FontVec,
    scale: PxScale,
    color: Rgba<u8>,
) {
    let (tw, th) = text_size(scale, font, text);
    let x = r.x as i32 + ((r.w as i32 - tw as i32).max(0)) / 2;
    let y = r.y as i32 + ((r.h as i32 - th as i32).max(0)) / 2;
    draw_text_mut(canvas, color, x, y, scale, font, text);
}

/// Pick a PxScale so the given `text` fits within `r`, starting from `ceiling`
/// and shrinking until it fits (or we hit 10px and give up).
fn fit_scale(font: &FontVec, text: &str, r: Rect, ceiling: f32) -> PxScale {
    let mut s = ceiling;
    while s > 10.0 {
        let scale = PxScale::from(s);
        let (tw, th) = text_size(scale, font, text);
        if tw <= r.w.saturating_sub(4) && th <= r.h.saturating_sub(4) {
            return scale;
        }
        s -= 2.0;
    }
    PxScale::from(10.0)
}

/// Resize a panel image to fit the given rect by scaling (nearest-neighbor
/// is cheap and good enough; the FLUX panels arrive already ~square).
fn paste_panel(canvas: &mut RgbaImage, panel: &image::DynamicImage, rect: Rect) {
    let resized = panel.resize_exact(rect.w, rect.h, image::imageops::FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    for (px, py, pixel) in rgba.enumerate_pixels() {
        let cx = rect.x + px;
        let cy = rect.y + py;
        if cx < canvas.width() && cy < canvas.height() {
            canvas.put_pixel(cx, cy, *pixel);
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Composite one strip's three panels + three plates into a single PNG.
///
/// The function is `async` to slot into the orchestrator's event loop even
/// though the underlying image work is CPU-bound — heavy work is wrapped in
/// `tokio::task::spawn_blocking` so it never stalls the reactor. See
/// `orchestrator/spec.md` §Workspace layout for the context.
// @trace spec:orchestrator, spec:trace-plate, spec:style-bible
// @Lesson S1-1000
pub async fn composite_strip(
    input: &StripInput<'_>,
    output_path: &Path,
    font: &FontSet,
) -> Result<ComposeResult, TtError> {
    // Copy the inputs we need into owned values — we cross an await boundary
    // into `spawn_blocking`, so all captures must be `'static` / `Send`.
    let strip_id = input.strip_id;
    let total = input.total_strips;
    let panel_paths = input.panels.clone();
    let title_display = input.title.display.to_string();
    let title_float_right = input.title.float_right;
    let title_backing = input.title.backing;
    let lesson_line = format!("[@Lesson {}]", input.lesson.short());
    let trace_line = format!("[@trace spec:{}]", input.trace_spec);
    let episode_text = format!("Tlatoāni Tales {}/{}", strip_id, total);
    let output_path_owned = output_path.to_path_buf();

    // `FontVec` is not `Clone`, but the underlying bytes are. We bounce
    // through `as_slice` to build an independent pair inside the blocking
    // task. Both fonts implement `Font` for the Glyph API.
    let chrome_bytes = font.chrome.as_slice().to_vec();
    let title_bytes = font.title.as_slice().to_vec();

    let regions = tokio::task::spawn_blocking(move || -> Result<ComposeResult, TtError> {
        let chrome = FontVec::try_from_vec(chrome_bytes)
            .map_err(|e| TtError::Infra(format!("chrome font re-wrap failed: {e}")))?;
        let title_font = FontVec::try_from_vec(title_bytes)
            .map_err(|e| TtError::Infra(format!("title font re-wrap failed: {e}")))?;

        // Load and verify panels.
        let mut panels = Vec::with_capacity(3);
        for p in panel_paths.iter() {
            let img = image::open(p).map_err(|e| {
                TtError::Infra(format!("panel load failed: `{}`: {e}", p.display()))
            })?;
            // Paranoid canon check: a wildly non-square or minuscule panel
            // means an upstream pipeline bug, not an infra issue.
            let (w, h) = (img.width(), img.height());
            if w < 64 || h < 64 {
                return Err(TtError::Canon(format!(
                    "panel `{}` is degenerate (too small: {w}x{h})",
                    p.display()
                )));
            }
            let ratio = w as f32 / h as f32;
            if !(0.3..=3.0).contains(&ratio) {
                return Err(TtError::Canon(format!(
                    "panel `{}` has implausible aspect {ratio:.2} ({w}x{h})",
                    p.display()
                )));
            }
            panels.push(img);
        }

        // Fresh paper canvas.
        let mut canvas = RgbaImage::from_pixel(CANVAS_W, CANVAS_H, PAPER);

        // Panel bed.
        let rects = panel_rects(CANVAS_W, CANVAS_H);
        for (rect, img) in rects.iter().zip(panels.iter()) {
            paste_panel(&mut canvas, img, *rect);
        }

        // Plate regions are the single source of truth for coordinates.
        let regions = compute_plate_regions(CANVAS_W, CANVAS_H, title_float_right);

        // --- Title plate ---------------------------------------------------
        if matches!(title_backing, TitleBacking::Scroll) {
            draw_plate_background(&mut canvas, regions.title);
        }
        let title_scale = fit_scale(&title_font, &title_display, regions.title, 72.0);
        draw_text_in_rect(
            &mut canvas,
            regions.title,
            &title_display,
            &title_font,
            title_scale,
            INK,
        );

        // --- Trace + Lesson plate (bottom-left) ---------------------------
        draw_plate_background(&mut canvas, regions.trace_lesson.outer);
        let lesson_scale =
            fit_scale(&chrome, &lesson_line, regions.trace_lesson.lesson_line, 48.0);
        draw_text_in_rect(
            &mut canvas,
            regions.trace_lesson.lesson_line,
            &lesson_line,
            &chrome,
            lesson_scale,
            INK,
        );
        let trace_scale =
            fit_scale(&chrome, &trace_line, regions.trace_lesson.trace_line, 42.0);
        draw_text_in_rect(
            &mut canvas,
            regions.trace_lesson.trace_line,
            &trace_line,
            &chrome,
            trace_scale,
            INK,
        );

        // --- Episode plate (bottom-right) ---------------------------------
        draw_plate_background(&mut canvas, regions.episode);
        let ep_scale = fit_scale(&chrome, &episode_text, regions.episode, 56.0);
        draw_text_in_rect(
            &mut canvas,
            regions.episode,
            &episode_text,
            &chrome,
            ep_scale,
            INK,
        );

        // Write PNG.
        if let Some(parent) = output_path_owned.parent() {
            std::fs::create_dir_all(parent)?;
        }
        canvas
            .save(&output_path_owned)
            .map_err(|e| TtError::Infra(format!("png write failed: {e}")))?;

        Ok(ComposeResult {
            png_path: output_path_owned,
            plate_regions: regions,
        })
    })
    .await
    .map_err(|e| TtError::Infra(format!("compose worker panicked: {e}")))??;

    Ok(regions)
}

/// Thin wrapper around `image::open` — here so other crates do not each pull
/// in `image` for a single PNG read.
pub fn load_png(path: &Path) -> Result<image::DynamicImage, TtError> {
    image::open(path).map_err(|e| TtError::Infra(format!("png open failed `{}`: {e}", path.display())))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};
    use std::path::PathBuf;

    // -- Geometry ---------------------------------------------------------

    #[test]
    fn panel_rects_span_canvas_with_gutters() {
        let rects = panel_rects(1800, 900);
        // Heights all equal the canvas.
        for r in &rects {
            assert_eq!(r.h, 900);
            assert_eq!(r.y, 0);
        }
        // Widths sum with gutters to full canvas.
        let total = rects[0].w + rects[1].w + rects[2].w + GUTTER * 2;
        assert_eq!(total, 1800);
        // Adjacency: each panel starts exactly GUTTER pixels after the last.
        assert_eq!(rects[1].x, rects[0].x + rects[0].w + GUTTER);
        assert_eq!(rects[2].x, rects[1].x + rects[1].w + GUTTER);
        // Every panel is ≈ 600 px wide (canvas/3 minus gutter share).
        for r in &rects {
            assert!(r.w > 580 && r.w < 620, "unexpected panel width {}", r.w);
        }
    }

    #[test]
    fn compute_plate_regions_default_layout() {
        let regions = compute_plate_regions(1800, 900, false);
        // Title top-left at origin.
        assert_eq!(regions.title.x, 0);
        assert_eq!(regions.title.y, 0);
        assert_eq!(regions.title.w, TITLE_W);
        assert_eq!(regions.title.h, TITLE_H);

        // Bottom-left trace plate aligned to canvas left edge, bottom band.
        assert_eq!(regions.trace_lesson.outer.x, 0);
        assert_eq!(regions.trace_lesson.outer.y, 900 - PLATE_BOTTOM_H);
        assert_eq!(regions.trace_lesson.outer.w, TRACE_PLATE_W);
        assert_eq!(regions.trace_lesson.outer.h, PLATE_BOTTOM_H);

        // Lesson line is inset; trace line sits directly below it.
        assert!(regions.trace_lesson.lesson_line.x > regions.trace_lesson.outer.x);
        assert!(regions.trace_lesson.trace_line.y > regions.trace_lesson.lesson_line.y);

        // Bottom-right episode plate hugs the right edge.
        assert_eq!(regions.episode.x + regions.episode.w, 1800);
        assert_eq!(regions.episode.y, 900 - PLATE_BOTTOM_H);
        assert_eq!(regions.episode.w, EPISODE_PLATE_W);

        // Sub-lines fit strictly inside the outer trace_lesson rect.
        assert!(
            regions.trace_lesson.lesson_line.y >= regions.trace_lesson.outer.y
                && regions.trace_lesson.lesson_line.y
                    + regions.trace_lesson.lesson_line.h
                    <= regions.trace_lesson.outer.y + regions.trace_lesson.outer.h
        );
        assert!(
            regions.trace_lesson.trace_line.y + regions.trace_lesson.trace_line.h
                <= regions.trace_lesson.outer.y + regions.trace_lesson.outer.h
        );
    }

    #[test]
    fn title_float_right_shifts_x() {
        let default = compute_plate_regions(1800, 900, false);
        let right = compute_plate_regions(1800, 900, true);
        assert_eq!(default.title.x, 0);
        assert!(right.title.x > 0);
        assert_eq!(right.title.x + right.title.w, 1800);
        // Width/height unchanged — just the origin shifts.
        assert_eq!(right.title.w, default.title.w);
        assert_eq!(right.title.h, default.title.h);
    }

    #[test]
    fn plate_regions_stay_inside_canvas() {
        let r = compute_plate_regions(CANVAS_W, CANVAS_H, false);
        let all = [r.title, r.trace_lesson.outer, r.episode];
        for rect in all {
            assert!(rect.x + rect.w <= CANVAS_W, "rect overflows width: {rect:?}");
            assert!(rect.y + rect.h <= CANVAS_H, "rect overflows height: {rect:?}");
        }
    }

    // -- Font loading -----------------------------------------------------

    #[test]
    #[ignore = "requires assets/fonts/atkinson-hyperlegible-regular.ttf to be committed; see assets/fonts/README.md"]
    fn font_load_from_project_root() {
        let fs = load_embedded().expect("chrome font should load once the TTF is committed");
        // Smoke: glyph for 'T' renders to something.
        let (w, _h) = text_size(PxScale::from(48.0), &fs.chrome, "T");
        assert!(w > 0);
    }

    #[test]
    fn font_load_reports_missing_chrome_with_helpful_hint() {
        let tmp = std::env::temp_dir().join("tt-compose-empty-fonts");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let err = load_from_fonts_dir(&tmp).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("assets/fonts/README.md"), "got: {msg}");
        // Chrome-font missing is an Infra class failure.
        assert_eq!(err.class(), tt_core::FailureClass::Infra);
    }

    // -- composite_strip end-to-end --------------------------------------

    /// Synthesize a single-color RGBA PNG of the given size at `path`.
    fn write_synthetic_panel(path: &Path, color: Rgba<u8>, w: u32, h: u32) {
        let img = ImageBuffer::from_pixel(w, h, color);
        img.save(path).unwrap();
    }

    /// Tiny synthetic font backed by ab_glyph's own unit-test font if
    /// present; otherwise the chrome font from the repo; otherwise skip
    /// by returning None — the caller `#[ignore]`s in that case.
    fn synthetic_font_set() -> Option<FontSet> {
        // Prefer the repo's own font if committed.
        let root = tt_core::project_root();
        let dir = root.join("assets").join("fonts");
        if dir.join("atkinson-hyperlegible-regular.ttf").exists() {
            return load_from_fonts_dir(&dir).ok();
        }
        None
    }

    #[tokio::test]
    #[ignore = "requires a TTF under assets/fonts/; run after Atkinson Hyperlegible is committed"]
    async fn composite_strip_produces_png_with_correct_dims() {
        let Some(fs) = synthetic_font_set() else {
            eprintln!("skipping: no chrome font present");
            return;
        };

        let tmp = std::env::temp_dir().join("tt-compose-e2e");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let p1 = tmp.join("p1.png");
        let p2 = tmp.join("p2.png");
        let p3 = tmp.join("p3.png");
        write_synthetic_panel(&p1, Rgba([220, 60, 60, 255]), 600, 900);
        write_synthetic_panel(&p2, Rgba([60, 220, 60, 255]), 600, 900);
        write_synthetic_panel(&p3, Rgba([60, 60, 220, 255]), 600, 900);

        let out = tmp.join("Tlatoāni_Tales_01.png");
        let lesson = LessonId::new("S1-100-volatile-is-dangerous").unwrap();
        let spec = SpecName::new("concept-curriculum").unwrap();
        let input = StripInput {
            strip_id: StripId::new(1).unwrap(),
            total_strips: 15,
            panels: [p1, p2, p3],
            title: TitleSpec {
                display: "[Volatile is dangerous]",
                float_right: false,
                backing: TitleBacking::None,
            },
            lesson: &lesson,
            trace_spec: &spec,
        };

        let result = composite_strip(&input, &out, &fs).await.unwrap();

        // PNG exists and matches canvas dimensions.
        let meta = std::fs::metadata(&result.png_path).unwrap();
        assert!(meta.len() > 0);
        let decoded = image::open(&result.png_path).unwrap();
        assert_eq!(decoded.width(), CANVAS_W);
        assert_eq!(decoded.height(), CANVAS_H);

        // Every plate rect fits inside the canvas.
        for r in [
            result.plate_regions.title,
            result.plate_regions.trace_lesson.outer,
            result.plate_regions.trace_lesson.lesson_line,
            result.plate_regions.trace_lesson.trace_line,
            result.plate_regions.episode,
        ] {
            assert!(r.x + r.w <= CANVAS_W);
            assert!(r.y + r.h <= CANVAS_H);
        }
    }

    /// Non-async synthetic test — skips the `spawn_blocking` path but
    /// exercises the panel-load canon check and the geometry return.
    ///
    /// Ensures that when the panel dimensions are obviously wrong, we
    /// classify the error as `Canon` (not `Infra`).
    #[tokio::test]
    #[ignore = "requires a TTF under assets/fonts/; gated alongside the e2e test"]
    async fn composite_strip_rejects_degenerate_panel_as_canon() {
        let Some(fs) = synthetic_font_set() else {
            return;
        };
        let tmp = std::env::temp_dir().join("tt-compose-canon");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let p1 = tmp.join("tiny.png");
        // 10x10 is below the 64-pixel minimum.
        write_synthetic_panel(&p1, Rgba([200, 0, 0, 255]), 10, 10);
        let p2 = tmp.join("p2.png");
        let p3 = tmp.join("p3.png");
        write_synthetic_panel(&p2, Rgba([0, 200, 0, 255]), 600, 900);
        write_synthetic_panel(&p3, Rgba([0, 0, 200, 255]), 600, 900);
        let lesson = LessonId::new("S1-100-volatile-is-dangerous").unwrap();
        let spec = SpecName::new("concept-curriculum").unwrap();
        let out = tmp.join("out.png");
        let input = StripInput {
            strip_id: StripId::new(1).unwrap(),
            total_strips: 15,
            panels: [p1, p2, p3],
            title: TitleSpec {
                display: "[x]",
                float_right: false,
                backing: TitleBacking::None,
            },
            lesson: &lesson,
            trace_spec: &spec,
        };
        let err = composite_strip(&input, &out, &fs).await.unwrap_err();
        assert_eq!(err.class(), tt_core::FailureClass::Canon);
    }

    // -- Rect helper ------------------------------------------------------

    #[test]
    fn rect_contains_point_boundary() {
        let r = Rect { x: 10, y: 20, w: 5, h: 7 };
        assert!(r.contains_point(10, 20));
        assert!(r.contains_point(14, 26));
        assert!(!r.contains_point(15, 26)); // outside right edge
        assert!(!r.contains_point(14, 27)); // outside bottom edge
        assert!(!r.contains_point(9, 20));
    }

    // -- StripInput + PathBuf are usable without panicking ---------------

    #[test]
    fn strip_input_constructs_cleanly() {
        let lesson = LessonId::new("S1-1500-proof-by-self-reference").unwrap();
        let spec = SpecName::new("trace-plate").unwrap();
        let input = StripInput {
            strip_id: StripId::new(15).unwrap(),
            total_strips: 15,
            panels: [PathBuf::from("/a"), PathBuf::from("/b"), PathBuf::from("/c")],
            title: TitleSpec {
                display: "[Proof by self-reference]",
                float_right: true,
                backing: TitleBacking::Scroll,
            },
            lesson: &lesson,
            trace_spec: &spec,
        };
        // Round-trip: the lesson/title fields are read back as we expect.
        assert_eq!(input.strip_id.as_u16(), 15);
        assert_eq!(input.total_strips, 15);
        assert_eq!(input.lesson.short(), "S1-1500");
        assert_eq!(input.trace_spec.as_str(), "trace-plate");
        assert!(input.title.float_right);
        assert!(matches!(input.title.backing, TitleBacking::Scroll));
    }
}
