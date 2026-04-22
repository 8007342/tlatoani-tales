//! Tlatoāni Tales — OpenSpec and strip proposal loader.
//!
//! Walks `openspec/specs/` and `strips/`, parses each file's YAML
//! frontmatter (if any) plus its markdown body via `pulldown-cmark`, and
//! returns a typed `SpecGraph` the rest of the pipeline consumes. Validates
//! declared lessons, trace specs, and `depends_on` against the registries.
//!
//! Governing spec: `openspec/specs/lessons/spec.md`,
//! `openspec/specs/trace-plate/spec.md`,
//! `openspec/specs/lesson-driven-development/spec.md`.
//!
// @trace spec:lessons, spec:trace-plate, spec:orchestrator
// @Lesson S1-500

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tt_core::{LessonId, SpecName, StripId, TtError};

pub mod frontmatter;
pub mod graph;

/// A single raw spec file as read from disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFile {
    pub name: SpecName,
    pub path: PathBuf,
    pub frontmatter: Option<serde_yaml::Value>,
    pub body: String,
}

/// A strip's `proposal.md` — the declaration of what a strip teaches, which
/// spec governs it, and how its plates are rendered.
///
/// See `openspec/specs/trace-plate/spec.md` §Selection rule (per strip).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripProposal {
    pub strip_id: StripId,
    pub lesson: LessonId,
    pub trace_spec: SpecName,
    pub title: String,
    pub title_float: TitleFloat,
    pub title_backing: TitleBacking,
    pub title_linkable: bool,
    pub reinforces: Vec<LessonId>,
    pub panels: Vec<PanelSpec>,
}

/// Where the top-left title plate sits. `Left` is the default.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TitleFloat {
    #[default]
    Left,
    Right,
}

/// Optional backing behind the stylized title plate — declared when a busy
/// panel hurts legibility.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TitleBacking {
    #[default]
    None,
    Scroll,
}

/// One panel inside a strip proposal. Content is deliberately minimal in
/// scaffolding — the full schema expands in a later change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelSpec {
    pub index: u8,
    pub prompt: String,
    pub seed: u64,
}

/// A lesson spec, matching the seven-field contract from
/// `openspec/specs/lesson-driven-development/spec.md`.
///
/// Scaffolded with optional fields; `tt-lint` enforces presence at verify time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LessonSpec {
    pub id: Option<LessonId>,
    pub display: Option<String>,
    pub abstract_text: Option<String>,
    pub position: Option<String>,
    pub references: Vec<String>,
    pub script: Option<String>,
    pub joke: Option<String>,
    pub punchline: Option<String>,
    pub aha_moment: Option<String>,
    pub trace: Vec<SpecName>,
}

/// Typed graph of everything `tt-specs` loaded from disk.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecGraph {
    pub specs: Vec<SpecFile>,
    pub lessons: Vec<LessonSpec>,
    pub strips: Vec<StripProposal>,
}

impl SpecGraph {
    /// Produce a single canonical string for the four style-governing specs
    /// (style-bible, character-canon, symbol-dictionary, trace-plate) in
    /// declaration order. Consumed by `tt-hashing::global_style_hash`.
    pub fn style_bodies_concat(&self) -> String {
        // Scaffolding: return a stable empty placeholder. Real impl filters
        // `self.specs` for the four style-governing names.
        String::new()
    }
}

/// Walk a project directory and return the full `SpecGraph`.
///
/// Scaffold: returns an empty graph. Implementation lands in a later change.
pub async fn load_all(_project_dir: &Path) -> Result<SpecGraph, TtError> {
    Ok(SpecGraph::default())
}
