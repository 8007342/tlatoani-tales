//! Tlatoāni Tales — OpenSpec and strip proposal loader.
//!
//! Walks `openspec/specs/**/spec.md` and `strips/**/proposal.md`, mines
//! `@trace spec:…` / `@Lesson …` tags, and returns a typed [`SpecGraph`]
//! the rest of the pipeline consumes. [`SpecGraph::validate`] surfaces
//! unresolved references against the loaded registries.
//!
//! Governing specs: `openspec/specs/lessons/spec.md`,
//! `openspec/specs/trace-plate/spec.md`,
//! `openspec/specs/lesson-driven-development/spec.md`.
//
// @trace spec:lessons, spec:trace-plate, spec:orchestrator
// @Lesson S1-400

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tt_core::{LessonId, SpecName, StripId, TtError};

pub mod frontmatter;
pub mod graph;

// ---------------------------------------------------------------------------
// Public data types
// ---------------------------------------------------------------------------

/// A single raw spec file plus the trace/lesson tags mined from its body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFile {
    /// Parent directory name (`openspec/specs/orchestrator/` → `orchestrator`).
    pub name: SpecName,
    pub path: PathBuf,
    pub frontmatter: Option<serde_yaml::Value>,
    pub title: Option<String>,
    pub body: String,
    pub traces: Vec<SpecName>,
    pub lesson_refs: Vec<LessonId>,
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
    pub path: PathBuf,
}

/// Where the top-left title plate sits. `Left` is the default.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TitleFloat {
    #[default]
    Left,
    Right,
}

/// Optional backing behind the stylized title plate.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TitleBacking {
    #[default]
    None,
    Scroll,
}

/// One panel inside a strip proposal. Minimal for now; the full schema
/// lands when proposals start being authored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelSpec {
    pub index: u8,
    pub prompt: String,
    pub seed: u64,
}

/// A per-lesson spec, matching the seven-field contract from
/// `openspec/specs/lesson-driven-development/spec.md`. Sections that were
/// absent or held only the `_(TBD …)_` placeholder come through as `None`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LessonSpec {
    pub id: Option<LessonId>,
    pub path: Option<PathBuf>,
    pub display: Option<String>,
    pub abstract_text: Option<String>,
    pub position: Option<String>,
    pub predecessors: Vec<LessonId>,
    pub successors: Vec<LessonId>,
    pub references: Vec<String>,
    pub script: Option<String>,
    pub joke: Option<String>,
    pub punchline: Option<String>,
    pub aha_moment: Option<String>,
    pub trace: Vec<SpecName>,
    pub lesson_refs: Vec<LessonId>,
}

/// Typed graph of everything `tt-specs` loaded from disk.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecGraph {
    pub specs: Vec<SpecFile>,
    pub lessons: Vec<LessonSpec>,
    pub strips: Vec<StripProposal>,
}

// ---------------------------------------------------------------------------
// SpecGraph — accessors + validation
// ---------------------------------------------------------------------------

impl SpecGraph {
    /// Concatenated bodies of the four style-governing specs (style-bible,
    /// character-canon, symbol-dictionary, trace-plate) in declaration order.
    /// Consumed by `tt-hashing::global_style_hash`.
    pub fn style_bodies_concat(&self) -> String {
        const STYLE_SPECS: &[&str] = &[
            "style-bible",
            "character-canon",
            "symbol-dictionary",
            "trace-plate",
        ];
        let mut out = String::new();
        for &name in STYLE_SPECS {
            if let Some(spec) = self.specs.iter().find(|s| s.name.as_str() == name) {
                out.push_str(&spec.body);
            }
        }
        out
    }

    /// Look up a spec by name.
    pub fn spec(&self, name: &SpecName) -> Option<&SpecFile> {
        self.specs.iter().find(|s| &s.name == name)
    }

    /// Look up a lesson by id. Supports both full-slug and short-form
    /// (`S1-100`) lookups; the short form resolves unambiguously by
    /// `S<n>-<NNN>` prefix when exactly one lesson matches.
    pub fn lesson(&self, id: &LessonId) -> Option<&LessonSpec> {
        if let Some(l) = self.lessons.iter().find(|l| l.id.as_ref() == Some(id)) {
            return Some(l);
        }
        let short = id.short();
        let mut matches = self
            .lessons
            .iter()
            .filter(|l| l.id.as_ref().is_some_and(|x| x.short() == short));
        let first = matches.next()?;
        if matches.next().is_some() {
            return None; // Ambiguous — refuse rather than pick silently.
        }
        Some(first)
    }

    /// Strips that cite this lesson as their primary lesson.
    pub fn strips_for_lesson(&self, id: &LessonId) -> Vec<&StripProposal> {
        self.strips.iter().filter(|s| &s.lesson == id).collect()
    }

    /// Specs whose `traces` include `name`.
    pub fn specs_citing(&self, name: &SpecName) -> Vec<&SpecFile> {
        self.specs
            .iter()
            .filter(|s| s.traces.iter().any(|t| t == name))
            .collect()
    }

    /// Lessons citing another lesson via their own `@Lesson` refs.
    pub fn lessons_citing(&self, id: &LessonId) -> Vec<&LessonSpec> {
        self.lessons
            .iter()
            .filter(|l| l.lesson_refs.iter().any(|r| r == id))
            .collect()
    }

    /// Validate cross-references. Returns every issue rather than short-
    /// circuiting, so tt-lint can present the full list. Checks that every
    /// trace, every `@Lesson`, every strip's declared lesson, and every
    /// predecessor/successor resolves against the loaded registries.
    pub fn validate(&self) -> Result<(), Vec<TtError>> {
        let mut errs = Vec::new();
        let unresolved_spec = |n: &SpecName, ctx: &str, errs: &mut Vec<TtError>| {
            if self.spec(n).is_none() {
                errs.push(TtError::Canon(format!("{ctx}: @trace spec:{n} does not resolve")));
            }
        };
        let unresolved_lesson = |l: &LessonId, ctx: &str, errs: &mut Vec<TtError>| {
            if self.lesson(l).is_none() {
                errs.push(TtError::Canon(format!("{ctx}: lesson {l} does not resolve")));
            }
        };

        for spec in &self.specs {
            let ctx = spec.path.display().to_string();
            for tr in &spec.traces { unresolved_spec(tr, &ctx, &mut errs); }
            for lr in &spec.lesson_refs { unresolved_lesson(lr, &ctx, &mut errs); }
        }
        for lesson in &self.lessons {
            let ctx = lesson
                .path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<unknown>".into());
            for tr in &lesson.trace { unresolved_spec(tr, &ctx, &mut errs); }
            for lr in &lesson.lesson_refs { unresolved_lesson(lr, &ctx, &mut errs); }
            for pre in &lesson.predecessors { unresolved_lesson(pre, &ctx, &mut errs); }
            for suc in &lesson.successors { unresolved_lesson(suc, &ctx, &mut errs); }
        }
        for strip in &self.strips {
            let ctx = strip.path.display().to_string();
            unresolved_lesson(&strip.lesson, &ctx, &mut errs);
            unresolved_spec(&strip.trace_spec, &ctx, &mut errs);
            for re in &strip.reinforces { unresolved_lesson(re, &ctx, &mut errs); }
        }
        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}

// ---------------------------------------------------------------------------
// load_all — the single entry point
// ---------------------------------------------------------------------------

/// Walk `project_dir` and return the full [`SpecGraph`].
///
/// Reads every `openspec/specs/**/spec.md` into a [`SpecFile`], additionally
/// parsing per-lesson specs into a [`LessonSpec`]. If `strips/` exists, reads
/// every `strips/NN-slug/proposal.md` into a [`StripProposal`]. Tolerant:
/// non-fatal issues become empty fields; resolvability is surfaced later by
/// [`SpecGraph::validate`].
pub async fn load_all(project_dir: &Path) -> Result<SpecGraph, TtError> {
    let mut graph = SpecGraph::default();

    let specs_root = project_dir.join("openspec").join("specs");
    if specs_root.is_dir() {
        for entry in walkdir::WalkDir::new(&specs_root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !entry.file_type().is_file()
                || path.file_name().and_then(|n| n.to_str()) != Some("spec.md")
            {
                continue;
            }
            let content = tokio::fs::read_to_string(path).await?;
            let (spec_file, lesson_spec) = parse_spec(path, &content);
            if let Some(s) = spec_file { graph.specs.push(s); }
            if let Some(l) = lesson_spec { graph.lessons.push(l); }
        }
    }

    let strips_root = project_dir.join("strips");
    if strips_root.is_dir() {
        for entry in walkdir::WalkDir::new(&strips_root)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !entry.file_type().is_file()
                || path.file_name().and_then(|n| n.to_str()) != Some("proposal.md")
            {
                continue;
            }
            let content = tokio::fs::read_to_string(path).await?;
            if let Some(s) = parse_strip_proposal(path, &content) { graph.strips.push(s); }
        }
    }

    Ok(graph)
}

// ---------------------------------------------------------------------------
// Internal parsers
// ---------------------------------------------------------------------------

/// Parse a `spec.md`. Returns a `SpecFile`; for per-lesson specs also a
/// populated `LessonSpec`.
fn parse_spec(path: &Path, content: &str) -> (Option<SpecFile>, Option<LessonSpec>) {
    let (fm, body) = frontmatter::split(content);
    let title = first_h1(&body);

    let Some(name) = spec_name_from_path(path) else { return (None, None) };
    // Per-lesson dirs carry uppercase `S`; fall back to the lowercased form
    // so every spec.md still lands in `graph.specs` with a valid SpecName.
    let spec_name = match SpecName::new(&name) {
        Ok(n) => n,
        Err(_) => match SpecName::new(&name.to_lowercase()) {
            Ok(n) => n,
            Err(_) => return (None, None),
        },
    };

    let body_clean = strip_fenced_code(&body);
    let traces = extract_traces(&body_clean);
    let lesson_refs = extract_lesson_refs(&body_clean);

    let spec_file = SpecFile {
        name: spec_name,
        path: path.to_path_buf(),
        frontmatter: fm.map(|f| f.raw),
        title: title.clone(),
        body: body.clone(),
        traces: traces.clone(),
        lesson_refs: lesson_refs.clone(),
    };

    // Is this a per-lesson spec? Test the *original* directory name since
    // `SpecName` lowercases the `S` prefix.
    let lesson_spec = LessonId::new(&name).ok().map(|id| {
        let sections = section_map(&body);
        let position = sections.get("Position").cloned();
        let (predecessors, successors) = position
            .as_deref()
            .map(parse_position)
            .unwrap_or_default();
        LessonSpec {
            id: Some(id),
            path: Some(path.to_path_buf()),
            display: title,
            abstract_text: sections.get("Abstract").cloned(),
            position,
            predecessors,
            successors,
            references: sections
                .get("References in this project")
                .map(|b| parse_list_items(b))
                .unwrap_or_default(),
            script: sections.get("Script").and_then(|b| non_stub(b)),
            joke: sections.get("Joke").and_then(|b| non_stub(b)),
            punchline: sections.get("Punchline").and_then(|b| non_stub(b)),
            aha_moment: ["Aha moment", "Aha!", "Aha"]
                .iter()
                .find_map(|k| sections.get(*k).cloned()),
            trace: traces,
            lesson_refs,
        }
    });

    (Some(spec_file), lesson_spec)
}

/// Parse a `strips/NN-slug/proposal.md`. Returns `None` if required
/// frontmatter fields are missing or malformed.
fn parse_strip_proposal(path: &Path, content: &str) -> Option<StripProposal> {
    let (fm, _body) = frontmatter::split(content);
    let fm = fm?;

    let lesson = LessonId::new(fm.get_str("lesson")?).ok()?;
    let trace_spec = SpecName::new(fm.get_str("trace_spec")?).ok()?;
    let title = fm.get_str("title").unwrap_or("").to_string();
    let reinforces = fm
        .get_list_str("reinforces")
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| LessonId::new(&s).ok())
        .collect();
    let strip_id = fm
        .get_str("strip")
        .and_then(parse_strip_field)
        .or_else(|| strip_id_from_dir(path))
        .unwrap_or_else(|| StripId::new(1).expect("1 is always valid"));

    Some(StripProposal {
        strip_id,
        lesson,
        trace_spec,
        title,
        title_float: TitleFloat::default(),
        title_backing: TitleBacking::default(),
        title_linkable: true,
        reinforces,
        panels: Vec::new(),
        path: path.to_path_buf(),
    })
}

// ---------------------------------------------------------------------------
// Markdown helpers
// ---------------------------------------------------------------------------

fn first_h1(body: &str) -> Option<String> {
    body.lines()
        .find_map(|l| l.strip_prefix("# ").map(|r| r.trim().to_string()))
}

/// Split a body on `## ` headings into `(heading -> section-body)`.
fn section_map(body: &str) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    let mut current: Option<String> = None;
    let mut buf = String::new();
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(name) = current.take() {
                out.insert(name, buf.trim().to_string());
                buf.clear();
            }
            current = Some(rest.trim().to_string());
            continue;
        }
        if current.is_some() {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    if let Some(name) = current {
        out.insert(name, buf.trim().to_string());
    }
    out
}

/// Parse a bulleted `## Position` block into `(predecessors, successors)`.
fn parse_position(body: &str) -> (Vec<LessonId>, Vec<LessonId>) {
    let mut pred = Vec::new();
    let mut succ = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) else {
            continue;
        };
        if let Some(rest) = strip_ci_prefix(rest, "Predecessors:") {
            pred.extend(parse_lesson_list(rest));
        } else if let Some(rest) = strip_ci_prefix(rest, "Successors:") {
            succ.extend(parse_lesson_list(rest));
        }
    }
    (pred, succ)
}

fn strip_ci_prefix<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if s.len() >= prefix.len()
        && s.is_char_boundary(prefix.len())
        && s[..prefix.len()].eq_ignore_ascii_case(prefix)
    {
        Some(s[prefix.len()..].trim_start())
    } else {
        None
    }
}

/// Split a comma-separated lesson list, tolerating em-dash placeholder and a
/// trailing parenthetical (e.g. `"… (and transitively …)"`).
fn parse_lesson_list(s: &str) -> Vec<LessonId> {
    let s = s.find(" (").map(|i| &s[..i]).unwrap_or(s);
    s.split(',')
        .map(|tok| tok.trim().trim_end_matches('.'))
        .filter(|tok| !tok.is_empty() && *tok != "—" && *tok != "-")
        .filter_map(|tok| LessonId::new(tok).ok())
        .collect()
}

fn parse_list_items(body: &str) -> Vec<String> {
    body.lines()
        .filter_map(|line| {
            let line = line.trim_start();
            line.strip_prefix("- ")
                .or_else(|| line.strip_prefix("* "))
                .map(|r| r.trim().to_string())
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// `None` for `_(TBD …)_` placeholders and empty bodies; `Some(trimmed)`
/// otherwise.
fn non_stub(body: &str) -> Option<String> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lowered = trimmed.to_ascii_lowercase();
    if lowered.starts_with("_(tbd") || lowered.contains("tbd — populated") {
        return None;
    }
    Some(trimmed.to_string())
}

/// Strip triple-backtick fenced code blocks so our regexes don't chase
/// tags inside code examples. Best-effort.
fn strip_fenced_code(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut in_fence = false;
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            out.push('\n');
            continue;
        }
        if !in_fence {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

// ---------------------------------------------------------------------------
// Tag extraction
// ---------------------------------------------------------------------------

fn trace_token_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"spec:([A-Za-z0-9\-][^\s,`\)\]\}]*)").expect("trace token regex compiles")
    })
}

fn lesson_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"@Lesson\s+\[?(S\d+-\d+(?:-[A-Za-z0-9\-]+)?)").expect("lesson regex compiles")
    })
}

/// Every `@trace spec:<name>` reference in `body`, in first-seen order.
/// Handles comma-separated lists (`@trace spec:foo, spec:bar`) by scanning
/// `spec:<name>` tokens on the same line after each `@trace` anchor.
fn extract_traces(body: &str) -> Vec<SpecName> {
    let mut seen: Vec<SpecName> = Vec::new();
    for line in body.lines() {
        let mut cursor = 0usize;
        while let Some(idx) = line[cursor..].find("@trace") {
            let start = cursor + idx;
            for cap in trace_token_regex().captures_iter(&line[start..]) {
                if let Some(m) = cap.get(1) {
                    let tok = m.as_str().trim_end_matches(|c: char| {
                        matches!(c, '.' | ',' | ';' | ')' | ']' | '}' | '`' | '"' | '\'')
                    });
                    if let Ok(name) = SpecName::new(tok) {
                        if !seen.contains(&name) {
                            seen.push(name);
                        }
                    }
                }
            }
            cursor = start + "@trace".len();
        }
    }
    seen
}

/// Every `@Lesson` reference in `body`. Short forms (`S1-100`) are stored as
/// a synthesized full id (`S1-100-short-form-citation`) so `LessonId::new`
/// succeeds; [`SpecGraph::lesson`] resolves them by prefix.
fn extract_lesson_refs(body: &str) -> Vec<LessonId> {
    let mut out: Vec<LessonId> = Vec::new();
    for cap in lesson_regex().captures_iter(body) {
        let token = &cap[1];
        let id = LessonId::new(token)
            .or_else(|_| LessonId::new(&format!("{token}-short-form-citation")));
        if let Ok(id) = id {
            if !out.contains(&id) {
                out.push(id);
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

fn spec_name_from_path(path: &Path) -> Option<String> {
    Some(path.parent()?.file_name()?.to_str()?.to_string())
}

fn strip_id_from_dir(path: &Path) -> Option<StripId> {
    let dir = path.parent()?.file_name()?.to_str()?;
    let num: String = dir.chars().take_while(|c| c.is_ascii_digit()).collect();
    StripId::new(num.parse().ok()?).ok()
}

fn parse_strip_field(s: &str) -> Option<StripId> {
    // Accept "TT 01/15", "01/15", or just "01".
    let after_tt = s.trim().trim_start_matches("TT").trim();
    let num: String = after_tt.chars().take_while(|c| c.is_ascii_digit()).collect();
    StripId::new(num.parse().ok()?).ok()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tt_core::project_root;

    #[test]
    fn section_map_splits_on_h2() {
        let body = "# Title\n\n## Abstract\n\nFoo.\n\n## Position\n\nBar.\n";
        let m = section_map(body);
        assert_eq!(m.get("Abstract").map(|s| s.as_str()), Some("Foo."));
        assert_eq!(m.get("Position").map(|s| s.as_str()), Some("Bar."));
    }

    #[test]
    fn non_stub_recognises_tbd() {
        assert!(non_stub("_(TBD — populated when strip is authored)_").is_none());
        assert_eq!(non_stub("Panel 1. Covi points.").unwrap(), "Panel 1. Covi points.");
    }

    #[test]
    fn extract_traces_handles_comma_list() {
        let body = "`@trace spec:lessons, spec:trace-plate, spec:orchestrator`";
        let traces = extract_traces(body);
        let names: Vec<String> = traces.iter().map(|n| n.as_str().to_string()).collect();
        for want in ["lessons", "trace-plate", "orchestrator"] {
            assert!(names.iter().any(|n| n == want), "missing {want} in {names:?}");
        }
    }

    #[test]
    fn extract_traces_ignores_fenced_code() {
        let body = "```\n@trace spec:ignored\n```\n@trace spec:real\n";
        let cleaned = strip_fenced_code(body);
        let names: Vec<String> = extract_traces(&cleaned)
            .iter()
            .map(|n| n.as_str().to_string())
            .collect();
        assert!(names.contains(&"real".to_string()));
        assert!(!names.contains(&"ignored".to_string()));
    }

    #[test]
    fn extract_lesson_refs_handles_both_forms() {
        let refs = extract_lesson_refs("@Lesson S1-100-volatile-is-dangerous and @Lesson S1-400.");
        assert_eq!(refs.len(), 2);
        assert!(refs.iter().any(|l| l.short() == "S1-100"));
        assert!(refs.iter().any(|l| l.short() == "S1-400"));
    }

    #[test]
    fn parse_position_parses_predecessors_and_successors() {
        let body = "- Season: S1\n- Number: 200\n- Predecessors: S1-100-volatile-is-dangerous\n- Successors: S1-300-memory-lives-in-history\n";
        let (p, s) = parse_position(body);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].short(), "S1-100");
        assert_eq!(s[0].short(), "S1-300");
    }

    #[test]
    fn parse_position_tolerates_em_dash() {
        let (p, s) = parse_position("- Predecessors: —\n- Successors: S1-200-save-means-findable\n");
        assert!(p.is_empty());
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn parse_position_tolerates_transitive_parenthetical() {
        let body = "- Predecessors: S1-1400-monotonic-convergence (and transitively all of S1-100 through S1-1300)\n- Successors: —\n";
        let (p, s) = parse_position(body);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].short(), "S1-1400");
        assert!(s.is_empty());
    }

    // -- Integration: the real repo ---------------------------------------

    #[tokio::test]
    async fn load_all_finds_the_real_corpus() {
        let graph = crate::load_all(&project_root()).await.expect("load_all ok");
        assert!(
            graph.specs.len() >= 20,
            "found {} specs (expected >= 20)",
            graph.specs.len()
        );
        assert_eq!(
            graph.lessons.len(),
            15,
            "Season 1 has 15 lessons; found {}",
            graph.lessons.len()
        );

        assert!(graph.spec(&SpecName::new("orchestrator").unwrap()).is_some());
        assert!(
            graph
                .lesson(&LessonId::new("S1-100-volatile-is-dangerous").unwrap())
                .is_some()
        );

        let dashboards = graph
            .lesson(&LessonId::new("S1-1000-dashboards-must-add-observability").unwrap())
            .expect("dashboards lesson present");
        assert!(
            dashboards.script.is_some(),
            "S1-1000 has a fleshed-out Script section"
        );

        for lesson in &graph.lessons {
            let id = lesson.id.as_ref().expect("every lesson has an id");
            let abs = lesson.abstract_text.as_deref().unwrap_or("");
            assert!(!abs.is_empty(), "{id}: abstract must be non-empty");
            if id.short() != "S1-1000" {
                assert!(
                    lesson.script.is_none(),
                    "{id}: expected stub script; got {:?}",
                    lesson.script
                );
            }
        }

        // Short-form `@Lesson S1-100` citations round-trip via the synthesized
        // stand-in id.
        let short = LessonId::new("S1-100-short-form-citation").unwrap();
        assert!(graph.lesson(&short).is_some());

        if let Err(errs) = graph.validate() {
            for e in &errs {
                eprintln!("validation error: {e}");
            }
            panic!("spec graph did not validate: {} errors", errs.len());
        }
    }

    #[tokio::test]
    async fn style_bodies_concat_is_non_empty_for_real_repo() {
        let graph = crate::load_all(&project_root()).await.expect("load_all ok");
        assert!(!graph.style_bodies_concat().is_empty());
    }
}
