//! Tlatoāni Tales — spec-invariant linter.
//!
//! Fast, pure, pre-commit-friendly. Walks the project, applies one rule per
//! governing spec, and returns a structured [`LintReport`]. Any violation is
//! a canon failure (exit 20) per `openspec/specs/orchestrator/spec.md`
//! §Failure modes; warnings are legal per-call and surface teachable-break
//! style non-blocking drift.
//!
//! Implemented rule families:
//!
//! - **Licensing coverage** — every committed file matches an R## rule in
//!   `openspec/specs/licensing/spec.md`.
//! - **Trace presence** — every code file/spec carries a `@trace spec:<name>`
//!   annotation (warnings for prose-only specs).
//! - **Lesson presence** — teaching-adjacent files carry at least one
//!   `@Lesson Sn-NNN` citation (warn-only).
//! - **Tlatoāni spelling** — plain ASCII `Tlatoani` only appears inside the
//!   catalogued TB01/TB02/TB03 allow-list paths (GitHub URLs, domain names,
//!   container-name string literals).
//! - **Plate declaration** — every strip proposal declares `lesson` and
//!   `trace_spec`.
//! - **Slug in registry** — every `@Lesson Sn-NNN` resolves to a lesson in
//!   `lessons/spec.md`.
//! - **Spec-in-lesson coverage** — a strip's declared `trace_spec` SHOULD
//!   appear in the lesson's coverage list (warn-only — coverage is a CRDT
//!   that grows over time).
//! - **Isolation flags** — every `podman run` invocation in code and
//!   scripts carries every entry in [`tt_core::podman::DEFAULT_FLAGS`].
//! - **Containerfile USER** — every `images/<role>/Containerfile` sets a
//!   non-zero UID via `USER` at the end.
//!
//! Lint is observability at build time — `@Lesson S1-800` (see-the-now).
//!
// @trace spec:orchestrator, spec:licensing, spec:trace-plate, spec:lessons, spec:tlatoāni-spelling, spec:isolation
// @Lesson S1-800
// @Lesson S1-1500

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};
use tt_core::{podman, TtError};
use tt_specs::SpecGraph;
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// One rule in the `tt-lint` vocabulary.
///
/// Each variant names a single invariant. `LintReport` groups violations and
/// warnings by `LintRule` so callers (CLI, Calmecac) can filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LintRule {
    /// Every committed file matches an R## rule in `licensing/spec.md`.
    LicensingCoverage,
    /// Every code file or spec has at least one `@trace spec:` citation.
    TracePresence,
    /// Teaching-adjacent file has at least one `@Lesson Sn-NNN` citation.
    LessonPresence,
    /// Plain ASCII `Tlatoani` outside the TB##-catalogued allow-list.
    TlatoaniSpelling,
    /// A strip proposal lacks `lesson` or `trace_spec`.
    PlateDeclaration,
    /// An `@Lesson Sn-NNN` citation points at an unregistered lesson.
    SlugInRegistry,
    /// A strip's declared `trace_spec` is absent from the lesson's coverage.
    SpecInLessonCoverage,
    /// A `podman run` invocation is missing a canonical hardening flag.
    IsolationFlags,
    /// An `images/*/Containerfile` does not end with a non-root `USER`.
    NoWriteAtNonRoot,
}

impl LintRule {
    /// Stable string form for logs, JSON, and display.
    pub fn as_str(&self) -> &'static str {
        match self {
            LintRule::LicensingCoverage => "licensing.coverage",
            LintRule::TracePresence => "trace.presence",
            LintRule::LessonPresence => "lesson.presence",
            LintRule::TlatoaniSpelling => "tlatoāni.spelling",
            LintRule::PlateDeclaration => "plate.declaration",
            LintRule::SlugInRegistry => "slug.in-registry",
            LintRule::SpecInLessonCoverage => "plate.lesson-spec-aligned",
            LintRule::IsolationFlags => "isolation.flags-present",
            LintRule::NoWriteAtNonRoot => "isolation.no-root-in-container",
        }
    }
}

impl fmt::Display for LintRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single rule violation — identifies where and why.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintViolation {
    /// The rule that produced this record.
    pub rule: LintRule,
    /// Project-relative (or absolute if unresolved) path to the offending file.
    pub path: PathBuf,
    /// Offending line number (1-indexed) when a grep-style rule pinpoints one.
    pub line: Option<u32>,
    /// Human-readable detail for the failure.
    pub message: String,
}

/// Aggregate report from a [`verify_all`] run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LintReport {
    /// Canon-class violations — each one fails `tt-render verify` with exit 20.
    pub violations: Vec<LintViolation>,
    /// Non-blocking notices (SHOULD-level rules, convergence drift).
    pub warnings: Vec<LintViolation>,
}

impl LintReport {
    /// `true` if at least one canon-class violation was recorded.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// `true` if no violations AND no warnings were recorded.
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty() && self.warnings.is_empty()
    }

    /// Push a violation into the canon-class bucket.
    pub fn add_violation(
        &mut self,
        rule: LintRule,
        path: impl Into<PathBuf>,
        line: Option<u32>,
        message: impl Into<String>,
    ) {
        self.violations.push(LintViolation {
            rule,
            path: path.into(),
            line,
            message: message.into(),
        });
    }

    /// Push a warning (non-blocking) notice.
    pub fn add_warning(
        &mut self,
        rule: LintRule,
        path: impl Into<PathBuf>,
        line: Option<u32>,
        message: impl Into<String>,
    ) {
        self.warnings.push(LintViolation {
            rule,
            path: path.into(),
            line,
            message: message.into(),
        });
    }
}

impl fmt::Display for LintReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "LintReport: {} violation(s), {} warning(s)",
            self.violations.len(),
            self.warnings.len()
        )?;
        for v in &self.violations {
            write_line(f, "violation", v)?;
        }
        for w in &self.warnings {
            write_line(f, "warning  ", w)?;
        }
        Ok(())
    }
}

fn write_line(f: &mut fmt::Formatter<'_>, tag: &str, v: &LintViolation) -> fmt::Result {
    let line_s = v
        .line
        .map(|n| format!(":{n}"))
        .unwrap_or_else(String::new);
    writeln!(
        f,
        "  [{tag}] {rule} {path}{line} — {msg}",
        tag = tag,
        rule = v.rule,
        path = v.path.display(),
        line = line_s,
        msg = v.message,
    )
}

// ---------------------------------------------------------------------------
// Public entrypoint
// ---------------------------------------------------------------------------

/// Run every rule against `project_dir`, collecting into a single report.
///
/// Non-short-circuiting: every rule runs even if an earlier one produced
/// violations. Returns `TtError` only on hard I/O failure (e.g. the project
/// directory doesn't exist); rule-level failures land in [`LintReport`].
///
/// The `graph` argument is accepted for signature-compatibility with the
/// task contract in `tt-lint`'s governing ticket — the single rule that
/// currently inspects the graph (`check_spec_in_lesson_coverage`) reads the
/// lesson's own on-disk spec anyway, so passing [`SpecGraph::default()`] is
/// legal. Overload [`verify_all_in`] is the zero-arg-convenience form used
/// by `tt-render verify`.
pub async fn verify_all(project_dir: &Path, graph: &SpecGraph) -> Result<LintReport, TtError> {
    if !project_dir.is_dir() {
        return Err(TtError::Usage(format!(
            "verify_all: project_dir does not exist or is not a directory: {}",
            project_dir.display()
        )));
    }

    let mut report = LintReport::default();

    check_licensing_coverage(project_dir, &mut report);
    check_trace_presence(project_dir, &mut report);
    check_lesson_presence(project_dir, &mut report);
    check_tlatoani_spelling(project_dir, &mut report);
    check_plate_declaration(project_dir, &mut report);
    check_slug_in_registry(project_dir, &mut report);
    check_spec_in_lesson_coverage(project_dir, graph, &mut report);
    check_isolation_flags(project_dir, &mut report);
    check_containerfile_user(project_dir, &mut report);

    Ok(report)
}

/// Convenience form used by `tt-render verify` when the caller does not
/// already have a [`SpecGraph`] in hand. Loads nothing from disk — the
/// rules that currently care about the graph also have a disk fallback, so
/// a default graph is safe.
pub async fn verify_all_in(project_dir: &Path) -> Result<LintReport, TtError> {
    let graph = SpecGraph::default();
    verify_all(project_dir, &graph).await
}

// ---------------------------------------------------------------------------
// Walker
// ---------------------------------------------------------------------------

/// Directories we always skip — git metadata, build artefacts, local tooling
/// clones, and ephemeral outputs. Keeps the walker from wandering into
/// target/, tools/ComfyUI/, and the like.
const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "tools",
    "output",
    "cache",
    "node_modules",
    ".opencode",
    ".claude",
];

/// Walk `root`, yielding files while skipping [`SKIP_DIRS`] at any depth.
fn walk_files(root: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !SKIP_DIRS.iter().any(|s| *s == name)
        })
        .filter_map(|r| r.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
}

fn rel<'a>(p: &'a Path, root: &Path) -> &'a Path {
    p.strip_prefix(root).unwrap_or(p)
}

fn read_text(p: &Path) -> Option<String> {
    fs::read_to_string(p).ok()
}

// ---------------------------------------------------------------------------
// Rule: licensing coverage (R##)
// ---------------------------------------------------------------------------

/// Files the licensing table deliberately leaves uncovered — the license
/// texts themselves (R09 is "verbatim upstream — no relicensing").
const LICENSING_VERBATIM: &[&str] = &["LICENSE", "LICENSE-ART"];

/// Decide whether `rel_path` is covered by at least one R## rule. This
/// mirrors the table in `licensing/spec.md` — when the spec grows a new R##,
/// we grow a match arm here.
// @trace spec:licensing
fn licensing_rule_match(rel_path: &Path) -> Option<&'static str> {
    let s = rel_path.to_string_lossy().replace('\\', "/");
    let name = rel_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // R10 — git metadata (match by filename; .gitignore lives at the root,
    // .gitkeep is the empty-directory sentinel convention).
    if matches!(
        name.as_str(),
        ".gitignore" | ".gitattributes" | ".gitkeep"
    ) {
        return Some("R10");
    }

    // R09 — license texts (verbatim upstream).
    if LICENSING_VERBATIM.iter().any(|v| *v == name) {
        return Some("R09");
    }

    // R11 — README.md (also R04, explicit for skimmers).
    if name == "README.md" {
        return Some("R11");
    }

    // R01 — scripts/** (first so a .sh under scripts still lands here cleanly).
    if s.starts_with("scripts/") {
        return Some("R01");
    }

    // R12 — calmecac/** PWA bundle (HTML, JS, CSS, webmanifest, service worker).
    if s.starts_with("calmecac/") {
        return Some("R12");
    }

    // R13 — images/<role>/Containerfile.
    if s.starts_with("images/") && name == "Containerfile" {
        return Some("R13");
    }

    // R14 — lockfiles (Cargo.lock, flake.lock, etc.).
    if s.ends_with(".lock") || name == "Cargo.lock" {
        return Some("R14");
    }

    // R15 — .gitkeep sentinel files (also matched by R10 above, but listed
    // explicitly in the table at R15 for readers skimming).
    if name == ".gitkeep" {
        return Some("R15");
    }

    // R16 — HTML under crates/*/src/** (include_str! template resources).
    if s.starts_with("crates/") && s.ends_with(".html") {
        return Some("R16");
    }

    // R17 — font binaries under assets/fonts/. Gitignored; rule exists
    // so that if a font file IS accidentally committed the licensing
    // graph still resolves it to the SIL OFL 1.1 vendored license.
    if s.starts_with("assets/fonts/")
        && (s.ends_with(".ttf")
            || s.ends_with(".otf")
            || s.ends_with(".woff")
            || s.ends_with(".woff2"))
    {
        return Some("R17");
    }

    // R18 — LICENSES/** — the REUSE-style bucket for vendored license texts.
    if s.starts_with("LICENSES/") {
        return Some("R18");
    }

    // Extension-driven rules.
    if s.ends_with(".sh") {
        return Some("R02");
    }
    if s.ends_with(".py") {
        return Some("R03");
    }
    if s.ends_with(".md") {
        return Some("R04");
    }
    if s.ends_with(".png") {
        return Some("R05");
    }
    if s.ends_with(".jpg") || s.ends_with(".jpeg") || s.ends_with(".webp") {
        return Some("R06");
    }
    if s.ends_with(".svg") {
        return Some("R07");
    }
    if s.ends_with(".yaml") || s.ends_with(".yml") || s.ends_with(".toml") || s.ends_with(".json") {
        return Some("R08");
    }

    None
}

/// Check that every walked file resolves to exactly one R## rule. Files
/// not covered are a canon violation (the licensing spec's invariant).
///
/// Files that are structural but outside the licensing table (Cargo.lock,
/// Containerfiles, rust-toolchain.toml-ish extensions) are *currently*
/// absorbed into R08 when their extension matches; anything else is
/// escalated. This is the CRDT mechanic: the spec grows, not the code.
// @trace spec:licensing
pub fn check_licensing_coverage(project_dir: &Path, report: &mut LintReport) {
    let lic_skip: &[&str] = &[
        // Lockfiles and rust-toolchain — structural, not source. Warn once
        // elsewhere; don't flood licensing violations with a known gap.
        "Cargo.lock",
        "rust-toolchain.toml",
        // Rust source is implicitly covered by the workspace's top-level
        // Cargo.toml license field; `tt-lint` doesn't need an R## for .rs
        // files in the current spec state.
        // Containerfiles are text scripts per licensing §"functional tooling".
        "Containerfile",
    ];

    for path in walk_files(project_dir) {
        let rp = rel(&path, project_dir);
        let name = rp
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Skip Rust sources (*.rs) + known structural files without raising
        // a canon failure — they are covered by the workspace license.
        if name.ends_with(".rs") || lic_skip.iter().any(|s| *s == name) {
            continue;
        }

        if licensing_rule_match(rp).is_none() {
            report.add_violation(
                LintRule::LicensingCoverage,
                rp,
                None,
                format!(
                    "no R## rule in openspec/specs/licensing/spec.md matches `{}`",
                    rp.display()
                ),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Rule: trace presence
// ---------------------------------------------------------------------------

/// File extensions that SHOULD carry a `@trace spec:<name>` annotation when
/// they live inside the repo's governed surface (code and spec prose).
fn is_trace_target(rel_path: &Path) -> bool {
    let s = rel_path.to_string_lossy();
    // Every spec.md under openspec/specs/** must carry @trace.
    if s.starts_with("openspec/specs/") && s.ends_with("spec.md") {
        return true;
    }
    // Rust source files in the workspace.
    if s.ends_with(".rs") {
        return true;
    }
    // Shell scripts under scripts/.
    if s.starts_with("scripts/") && s.ends_with(".sh") {
        return true;
    }
    // Containerfiles.
    rel_path
        .file_name()
        .map(|n| n == "Containerfile")
        .unwrap_or(false)
}

/// Warn when a trace-target file lacks any `@trace spec:` citation. Missing
/// traces inside Rust source are escalated to violations (the governing
/// spec in orchestrator says every orchestrator file is traced); missing
/// traces inside spec prose or scripts stay warnings because some prose
/// specs carry their trace only in the final section heading and our
/// substring match already accepts that pattern.
// @trace spec:orchestrator, spec:trace-plate
pub fn check_trace_presence(project_dir: &Path, report: &mut LintReport) {
    let needle = "@trace spec:";
    for path in walk_files(project_dir) {
        let rp = rel(&path, project_dir);
        if !is_trace_target(rp) {
            continue;
        }
        let Some(text) = read_text(&path) else {
            continue;
        };
        if !text.contains(needle) {
            // Rust sources: escalate. Specs/scripts: warn.
            let is_rs = rp
                .extension()
                .map(|e| e == "rs")
                .unwrap_or(false);
            if is_rs {
                report.add_violation(
                    LintRule::TracePresence,
                    rp,
                    None,
                    format!("`{}` has no `@trace spec:<name>` annotation", rp.display()),
                );
            } else {
                report.add_warning(
                    LintRule::TracePresence,
                    rp,
                    None,
                    format!("`{}` has no `@trace spec:<name>` annotation", rp.display()),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Rule: lesson presence
// ---------------------------------------------------------------------------

/// Teaching-adjacent files SHOULD carry at least one `@Lesson Sn-NNN`
/// citation. Scope is intentionally narrow: lesson specs themselves, strip
/// proposals, and Rust sources inside orchestrator-governed crates. Missing
/// citations are warnings — some prose-only sibling specs don't teach at
/// `@Lesson` granularity and that is OK.
// @trace spec:lessons
pub fn check_lesson_presence(project_dir: &Path, report: &mut LintReport) {
    let re_lesson = Regex::new(r"@Lesson\s+S\d+-\d+").expect("static regex");

    for path in walk_files(project_dir) {
        let rp = rel(&path, project_dir);
        let s = rp.to_string_lossy();

        let is_lesson_spec =
            s.starts_with("openspec/specs/lessons/") && s.ends_with("spec.md") && s != "openspec/specs/lessons/spec.md";
        let is_strip_proposal = s.starts_with("strips/") && s.ends_with("proposal.md");
        let is_crate_lib =
            s.starts_with("crates/") && s.ends_with("/src/lib.rs");

        if !(is_lesson_spec || is_strip_proposal || is_crate_lib) {
            continue;
        }

        let Some(text) = read_text(&path) else {
            continue;
        };
        if !re_lesson.is_match(&text) {
            report.add_warning(
                LintRule::LessonPresence,
                rp,
                None,
                format!("`{}` has no `@Lesson Sn-NNN` citation", rp.display()),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Rule: Tlatoāni spelling (TB01/TB02/TB03)
// ---------------------------------------------------------------------------

/// Substrings that, when present on a matching line, legitimise a plain
/// `Tlatoani` spelling per the catalogued teachable breaks:
///
/// - **TB01** — GitHub repo slug `github.com/8007342/tlatoani-tales`.
/// - **TB02** — domain names `tlatoani-tales.com`, `www.tlatoani-tales.com`,
///   `calmecac.tlatoani-tales.com`.
/// - **TB03** — Podman/toolbox container names `tlatoani-tales`,
///   `tlatoani-tales-inference`, `tlatoani-tales-trainer`,
///   `tlatoani-tales-viewer`, `tlatoani-tales-proxy`.
// @trace spec:tlatoāni-spelling
const TB_ALLOW_SUBSTRINGS: &[&str] = &[
    // TB01 + TB02 — URLs and domain names.
    "tlatoani-tales.com",
    "tlatoani-tales.github",
    "github.com/8007342/tlatoani-tales",
    "/tlatoani-tales/",
    "8007342/tlatoani-tales",
    // TB03 — container names (prefix + specific roles).
    "tlatoani-tales-",
    "\"tlatoani-tales\"",
    "`tlatoani-tales`",
    "<role>: tlatoani-tales",
    "name tlatoani-tales",
    "name=tlatoani-tales",
    "# @trace spec:image-gen-runtime",
    "toolbox run -c tlatoani-tales",
    "toolbox enter tlatoani-tales",
    " tlatoani-tales ",
    " tlatoani-tales.",
    " tlatoani-tales,",
    "(tlatoani-tales)",
    "tlatoani-tales`",
    "`tlatoani-tales-",
    "'tlatoani-tales'",
];

/// Paths where `Tlatoani` (no macron) is catalogued as expected — the
/// tlatoāni-spelling spec itself (which documents the break), the linter
/// crate that implements this rule (which necessarily mentions the plain
/// ASCII form in its doc comments, test fixtures, and error messages),
/// and the `tt-lora` character-name validator whose rejection tests
/// contain the uppercase ASCII form as a negative-case fixture. Matches
/// by path suffix to stay resilient to relative-vs-absolute drift.
// @trace spec:tlatoāni-spelling
const TB_ALLOW_PATHS: &[&str] = &[
    "openspec/specs/tlatoāni-spelling/spec.md",
    "crates/tt-lint/src/lib.rs",
    "crates/tt-lora/src/lib.rs",
    "Cargo.lock",
];

fn path_in_allow_list(rel_path: &Path) -> bool {
    let s = rel_path.to_string_lossy();
    TB_ALLOW_PATHS.iter().any(|a| s.ends_with(*a))
}

fn line_allowed_by_tb(line: &str) -> bool {
    TB_ALLOW_SUBSTRINGS.iter().any(|sub| line.contains(*sub))
}

/// Check every text file for plain `Tlatoani` (ASCII word-boundary, no
/// macron). Flag any occurrence that does NOT appear on a line catalogued
/// by TB01/TB02/TB03.
// @trace spec:tlatoāni-spelling
pub fn check_tlatoani_spelling(project_dir: &Path, report: &mut LintReport) {
    // Word boundary around `Tlatoani` — (?-u) keeps `\b` ASCII-only so a
    // trailing UTF-8 `ā` does NOT count as a word-break (Unicode-mode `\b`
    // treats `ā` as a word char; that would make `Tlatoāni` match on the
    // macron edge, which is the opposite of what we want). ASCII-mode `\b`
    // keeps it strictly English.
    let re = Regex::new(r"(?-u)\bTlatoani\b").expect("static regex");

    for path in walk_files(project_dir) {
        let rp = rel(&path, project_dir);

        if path_in_allow_list(rp) {
            continue;
        }

        // Binary files and large non-text artefacts are skipped by
        // read_text returning None for invalid UTF-8.
        let Some(text) = read_text(&path) else {
            continue;
        };

        for (i, line) in text.lines().enumerate() {
            if re.is_match(line) {
                if line_allowed_by_tb(line) {
                    continue;
                }
                report.add_violation(
                    LintRule::TlatoaniSpelling,
                    rp,
                    Some((i + 1) as u32),
                    format!(
                        "plain `Tlatoani` (no macron) outside catalogued TB allow-list: {}",
                        line.trim()
                    ),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Rule: plate declaration (strip proposals)
// ---------------------------------------------------------------------------

/// Every strip's `proposal.md` MUST declare `lesson:` and `trace_spec:` per
/// `trace-plate/spec.md` §Selection rule. We scan frontmatter-style YAML
/// keys at the top of the file; a proposal that omits either is a canon
/// violation because plates cannot render without them.
// @trace spec:trace-plate
pub fn check_plate_declaration(project_dir: &Path, report: &mut LintReport) {
    let strips = project_dir.join("strips");
    if !strips.is_dir() {
        // No strips authored yet — not a violation; the spec explicitly
        // allows strips/ to be empty in early Season 1 scaffolding.
        return;
    }

    for path in walk_files(&strips) {
        let rp = rel(&path, project_dir);
        if rp
            .file_name()
            .map(|n| n != "proposal.md")
            .unwrap_or(true)
        {
            continue;
        }
        let Some(text) = read_text(&path) else {
            continue;
        };

        let has_lesson = yaml_key_present(&text, "lesson");
        let has_trace_spec = yaml_key_present(&text, "trace_spec");

        if !has_lesson {
            report.add_violation(
                LintRule::PlateDeclaration,
                rp,
                None,
                format!(
                    "strip proposal `{}` must declare `lesson: Sn-NNN` (trace-plate §Selection rule)",
                    rp.display()
                ),
            );
        }
        if !has_trace_spec {
            report.add_violation(
                LintRule::PlateDeclaration,
                rp,
                None,
                format!(
                    "strip proposal `{}` must declare `trace_spec: <spec-name>` (trace-plate §Selection rule)",
                    rp.display()
                ),
            );
        }
    }
}

/// Crude YAML-frontmatter key detector — good enough for the
/// `proposal.md` shape the spec mandates (`key: value` at left margin).
fn yaml_key_present(text: &str, key: &str) -> bool {
    let prefix = format!("{key}:");
    text.lines()
        .any(|l| l.trim_start().starts_with(&prefix))
}

// ---------------------------------------------------------------------------
// Rule: slug in registry
// ---------------------------------------------------------------------------

/// Every `@Lesson Sn-NNN` citation anywhere in the repo must resolve to a
/// registered lesson in `lessons/spec.md`. Unregistered slugs are canon
/// failures — they mean either a typo or an undocumented lesson.
// @trace spec:lessons
pub fn check_slug_in_registry(project_dir: &Path, report: &mut LintReport) {
    let registry_path = project_dir
        .join("openspec")
        .join("specs")
        .join("lessons")
        .join("spec.md");

    let registry = match read_text(&registry_path) {
        Some(s) => s,
        None => return, // No registry → defer to the other checks.
    };

    // Collect every registered short-form slug (`S<n>-<NNN>`) from the
    // registry table.
    let re_id = Regex::new(r"S\d+-\d+").expect("static regex");
    let registered: std::collections::HashSet<String> = re_id
        .find_iter(&registry)
        .map(|m| m.as_str().to_string())
        .collect();

    // Every citation.
    let re_cite = Regex::new(r"@Lesson\s+(S\d+-\d+)").expect("static regex");

    for path in walk_files(project_dir) {
        let rp = rel(&path, project_dir);
        let s = rp.to_string_lossy();
        // Skip the registry itself, Cargo.lock, and the tt-lint crate's
        // test fixtures (which use synthetic `@Lesson S1-999` strings to
        // exercise the unregistered-slug code path).
        if s.ends_with("openspec/specs/lessons/spec.md")
            || s.ends_with("Cargo.lock")
            || s.ends_with("crates/tt-lint/src/lib.rs")
        {
            continue;
        }
        let Some(text) = read_text(&path) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            for cap in re_cite.captures_iter(line) {
                let id = &cap[1];
                if !registered.contains(id) {
                    report.add_violation(
                        LintRule::SlugInRegistry,
                        rp,
                        Some((i + 1) as u32),
                        format!("@Lesson {id} not found in lessons/spec.md registry"),
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Rule: spec-in-lesson coverage
// ---------------------------------------------------------------------------

/// When a strip proposal declares `trace_spec: X`, the lesson it also
/// declares SHOULD list `X` in its `## References in this project` section.
/// Coverage is a growing CRDT; a missing entry is a warning, not a canon
/// failure (retroactively filling coverage is normal after strip ships).
// @trace spec:trace-plate, spec:lessons
pub fn check_spec_in_lesson_coverage(
    project_dir: &Path,
    _graph: &SpecGraph,
    report: &mut LintReport,
) {
    let strips = project_dir.join("strips");
    if !strips.is_dir() {
        return;
    }

    for path in walk_files(&strips) {
        let rp = rel(&path, project_dir);
        if rp
            .file_name()
            .map(|n| n != "proposal.md")
            .unwrap_or(true)
        {
            continue;
        }
        let Some(text) = read_text(&path) else {
            continue;
        };

        let Some(lesson_id) = extract_yaml_value(&text, "lesson") else {
            continue;
        };
        let Some(trace_spec) = extract_yaml_value(&text, "trace_spec") else {
            continue;
        };

        // Load the lesson's own spec.md and look for the trace_spec string
        // inside the `## References in this project` section. The short
        // form `Sn-NNN` is the directory's prefix (legal lesson id).
        let short = lesson_id
            .split('-')
            .take(2)
            .collect::<Vec<_>>()
            .join("-");
        let lesson_dir = project_dir
            .join("openspec")
            .join("specs")
            .join("lessons");
        // Find the directory starting with `<short>-`.
        let entries = match fs::read_dir(&lesson_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        let mut lesson_spec_path: Option<PathBuf> = None;
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!("{short}-")) {
                lesson_spec_path = Some(e.path().join("spec.md"));
                break;
            }
        }
        let Some(lp) = lesson_spec_path else {
            continue;
        };
        let Some(lesson_body) = read_text(&lp) else {
            continue;
        };

        if !lesson_body.contains(&trace_spec) {
            report.add_warning(
                LintRule::SpecInLessonCoverage,
                rp,
                None,
                format!(
                    "strip's `trace_spec: {trace_spec}` does not appear in coverage of lesson `{short}` — consider adding it"
                ),
            );
        }
    }
}

/// Extract `key: value` from frontmatter-style YAML (first occurrence).
fn extract_yaml_value(text: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix(&prefix) {
            return Some(
                rest.trim()
                    .trim_matches(|c: char| c == '"' || c == '\'')
                    .to_string(),
            );
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Rule: isolation flags
// ---------------------------------------------------------------------------

/// Grep every source file for `podman run` invocations and verify that each
/// carries every entry in [`tt_core::podman::DEFAULT_FLAGS`]. The
/// `DEFAULT_FLAGS` constant IS the single source of truth per
/// `isolation/spec.md` §Canonical flags.
///
/// Detection targets **shell scripts only** — a real invocation in Rust
/// uses argv form (`Command::new("podman").args(["run", …])`) and routes
/// through `tt_core::podman::DEFAULT_FLAGS` already; any Rust file that
/// mentions `"podman run"` as a string literal is prose (doc comment, error
/// message). The spec's §Canonical flags rule is enforced in Rust via the
/// unit tests in `tt-lora` that call `podman::lint_flags(argv)`.
// @trace spec:isolation
pub fn check_isolation_flags(project_dir: &Path, report: &mut LintReport) {
    for path in walk_files(project_dir) {
        let rp = rel(&path, project_dir);
        let s = rp.to_string_lossy();

        // Only shell scripts contain direct `podman run …` invocations.
        // Rust code composes argv programmatically via tt_core::podman;
        // Containerfiles don't invoke podman at all (they describe a build).
        // Spec prose inside openspec/ quotes the command as documentation.
        let is_target = s.ends_with(".sh");
        if !is_target {
            continue;
        }
        if s.starts_with("openspec/") {
            continue;
        }

        let Some(text) = read_text(&path) else {
            continue;
        };

        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if !is_shell_podman_run_invocation(line) {
                continue;
            }
            // Look at the invocation as a multi-line window — shell argv
            // is often broken across backslash-continuation lines. We
            // grab the next 40 lines as context.
            let window = lines
                .iter()
                .skip(i)
                .take(40)
                .copied()
                .collect::<Vec<_>>()
                .join("\n");

            // Check the preceding 15 lines for a role-exempt pragma. Both
            // the viewer and inference roles legitimately omit
            // `--network=none` (HTTP-served — would have nothing to
            // forward to) and `--rm` (start/stop lifecycle, not
            // run-and-exit). The trainer role does NOT use this pragma:
            // it genuinely runs with --network=none + --rm. See
            // isolation/spec.md §Network mode per role.
            // Window of 15 lines accommodates the comment block that
            // typically explains the exemption immediately before the
            // invocation.
            let pre_start = i.saturating_sub(15);
            let preceding = lines[pre_start..i].join("\n");
            let is_viewer_role = preceding.contains("# tt-lint: viewer-role");
            let is_inference_role = preceding.contains("# tt-lint: inference-role");
            let is_long_running_service = is_viewer_role || is_inference_role;

            for required in podman::DEFAULT_FLAGS {
                if is_long_running_service
                    && (*required == "--network=none" || *required == "--rm")
                {
                    continue;
                }
                if !window.contains(required) {
                    report.add_violation(
                        LintRule::IsolationFlags,
                        rp,
                        Some((i + 1) as u32),
                        format!(
                            "`podman run` invocation missing canonical flag `{required}` — see openspec/specs/isolation/spec.md §Canonical flags"
                        ),
                    );
                }
            }
            // Only flag the first `podman run` per file — subsequent
            // occurrences usually share the same argv composition.
            break;
        }
    }
}

/// Decide whether a shell line is an actual `podman run` invocation (not a
/// comment or a prose mention inside a heredoc/string). A real invocation:
///
/// - is not a pure comment line (doesn't start with `#` after whitespace),
/// - mentions `podman run` as a command (preceded by start-of-line or
///   whitespace, followed by whitespace/backslash/end),
/// - does NOT wrap `podman run` in backticks (documentation convention).
fn is_shell_podman_run_invocation(line: &str) -> bool {
    let t = line.trim_start();
    if t.starts_with('#') {
        return false;
    }
    // Backtick-wrapped mentions are prose.
    if line.contains("`podman run`") || line.contains("`podman run ") {
        return false;
    }
    // Require the command at a word boundary (start-of-line or whitespace
    // boundary) and followed by whitespace, newline, or backslash.
    let re =
        Regex::new(r"(?m)(?:^|[\s;&|])podman\s+run(?:\s|\\|$)").expect("static regex");
    re.is_match(line)
}

// ---------------------------------------------------------------------------
// Rule: Containerfile USER (non-root)
// ---------------------------------------------------------------------------

/// Every `images/<role>/Containerfile` MUST declare a non-root user via
/// `USER <uid>[:<gid>]`, and that UID MUST be > 0 — matching the
/// isolation-spec `--userns=keep-id` posture. Missing `USER` or `USER 0`
/// is a canon failure.
// @trace spec:isolation
pub fn check_containerfile_user(project_dir: &Path, report: &mut LintReport) {
    let images = project_dir.join("images");
    if !images.is_dir() {
        return;
    }
    let re_user = Regex::new(r"(?m)^\s*USER\s+([^\s:]+)(?::([^\s]+))?").expect("static regex");

    for path in walk_files(&images) {
        let rp = rel(&path, project_dir);
        if rp
            .file_name()
            .map(|n| n != "Containerfile")
            .unwrap_or(true)
        {
            continue;
        }
        let Some(text) = read_text(&path) else {
            continue;
        };

        // Last USER declaration wins per Dockerfile semantics.
        let mut last: Option<(u32, String)> = None;
        for (i, line) in text.lines().enumerate() {
            if let Some(cap) = re_user.captures(line) {
                last = Some(((i + 1) as u32, cap[1].to_string()));
            }
        }

        match last {
            None => report.add_violation(
                LintRule::NoWriteAtNonRoot,
                rp,
                None,
                format!(
                    "Containerfile `{}` has no `USER` declaration — must end as non-root UID per isolation/spec.md",
                    rp.display()
                ),
            ),
            Some((ln, uid)) => {
                // Reject `root`, `0`, or any token that parses to 0.
                let zero = uid == "root" || uid == "0" || uid.parse::<u32>().ok() == Some(0);
                if zero {
                    report.add_violation(
                        LintRule::NoWriteAtNonRoot,
                        rp,
                        Some(ln),
                        format!(
                            "Containerfile `{}` declares root `USER {uid}` — isolation/spec.md forbids root-at-final-USER",
                            rp.display()
                        ),
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write(p: &Path, body: &str) {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, body).unwrap();
    }

    // --- LintReport Display ------------------------------------------------

    #[test]
    fn report_display_shows_counts_and_entries() {
        let mut r = LintReport::default();
        r.add_violation(
            LintRule::IsolationFlags,
            PathBuf::from("scripts/run.sh"),
            Some(10),
            "missing --network=none",
        );
        r.add_warning(
            LintRule::LessonPresence,
            PathBuf::from("crates/tt-x/src/lib.rs"),
            None,
            "no @Lesson",
        );
        let s = format!("{r}");
        assert!(s.contains("1 violation(s)"));
        assert!(s.contains("1 warning(s)"));
        assert!(s.contains("isolation.flags-present"));
        assert!(s.contains("lesson.presence"));
        assert!(s.contains("scripts/run.sh:10"));
    }

    #[test]
    fn has_violations_reflects_bucket_state() {
        let mut r = LintReport::default();
        assert!(!r.has_violations());
        assert!(r.is_clean());
        r.add_warning(
            LintRule::LessonPresence,
            PathBuf::from("."),
            None,
            "w",
        );
        assert!(!r.has_violations());
        assert!(!r.is_clean());
        r.add_violation(
            LintRule::IsolationFlags,
            PathBuf::from("."),
            None,
            "v",
        );
        assert!(r.has_violations());
    }

    // --- Synthetic accumulation --------------------------------------------

    #[test]
    fn synthetic_violations_accumulate_across_rules() {
        let mut r = LintReport::default();
        r.add_violation(LintRule::LicensingCoverage, ".", None, "a");
        r.add_violation(LintRule::PlateDeclaration, ".", None, "b");
        r.add_violation(LintRule::IsolationFlags, ".", None, "c");
        r.add_warning(LintRule::LessonPresence, ".", None, "d");
        assert_eq!(r.violations.len(), 3);
        assert_eq!(r.warnings.len(), 1);
    }

    // --- Tlatoāni spelling -------------------------------------------------

    #[test]
    fn tlatoani_spelling_flags_plain_usage() {
        let td = tempdir().unwrap();
        let root = td.path();
        // A prose file that uses plain Tlatoani where it should be Tlatoāni.
        write(
            &root.join("notes.md"),
            "# title\n\nTlatoani looks at Covi.\n",
        );
        // Corrected spelling elsewhere — macron form is fine.
        write(&root.join("good.md"), "Tlatoāni looks at Covi.\n");

        let mut r = LintReport::default();
        check_tlatoani_spelling(root, &mut r);
        assert_eq!(
            r.violations.len(),
            1,
            "exactly one plain-Tlatoani line should be flagged; got {:?}",
            r.violations
        );
        assert_eq!(r.violations[0].rule, LintRule::TlatoaniSpelling);
        assert_eq!(r.violations[0].line, Some(3));
    }

    #[test]
    fn tlatoani_spelling_skips_tb_allowlisted_lines() {
        let td = tempdir().unwrap();
        let root = td.path();
        // URL line (TB01/TB02) + container-name literal (TB03). Both legal.
        write(
            &root.join("README.md"),
            "repo: https://github.com/8007342/tlatoani-tales\n\
             container: `tlatoani-tales-inference`\n\
             domain: www.tlatoani-tales.com\n",
        );
        let mut r = LintReport::default();
        check_tlatoani_spelling(root, &mut r);
        assert!(
            r.violations.is_empty(),
            "TB allow-listed lines should not be flagged; got {:?}",
            r.violations
        );
    }

    #[test]
    fn tlatoani_spelling_skips_tb_allowlisted_paths() {
        let td = tempdir().unwrap();
        let root = td.path();
        // The spelling spec documents plain Tlatoani intentionally.
        write(
            &root
                .join("openspec")
                .join("specs")
                .join("tlatoāni-spelling")
                .join("spec.md"),
            "discusses plain Tlatoani (no macron) as evidence.\n",
        );
        let mut r = LintReport::default();
        check_tlatoani_spelling(root, &mut r);
        assert!(r.violations.is_empty());
    }

    // --- Isolation flags ---------------------------------------------------

    #[test]
    fn isolation_flags_catches_missing_cap_drop_all() {
        let td = tempdir().unwrap();
        let root = td.path();
        // Missing --cap-drop=ALL.
        write(
            &root.join("scripts").join("run.sh"),
            "#!/bin/sh\n\
             podman run --rm \\\n\
               --security-opt=no-new-privileges \\\n\
               --userns=keep-id \\\n\
               --read-only \\\n\
               --network=none \\\n\
               tlatoani-tales-inference\n",
        );
        let mut r = LintReport::default();
        check_isolation_flags(root, &mut r);
        assert!(
            r.violations
                .iter()
                .any(|v| v.message.contains("--cap-drop=ALL")),
            "expected violation for missing --cap-drop=ALL; got {:?}",
            r.violations
        );
        assert_eq!(r.violations[0].rule, LintRule::IsolationFlags);
    }

    #[test]
    fn isolation_flags_accepts_complete_invocation() {
        let td = tempdir().unwrap();
        let root = td.path();
        write(
            &root.join("scripts").join("run.sh"),
            "#!/bin/sh\n\
             podman run --rm \\\n\
               --cap-drop=ALL \\\n\
               --security-opt=no-new-privileges \\\n\
               --userns=keep-id \\\n\
               --read-only \\\n\
               --network=none \\\n\
               --name=tlatoani-tales-inference \\\n\
               tlatoani-tales-inference:v1\n",
        );
        let mut r = LintReport::default();
        check_isolation_flags(root, &mut r);
        assert!(r.violations.is_empty(), "got {:?}", r.violations);
    }

    // --- Plate declaration -------------------------------------------------

    #[test]
    fn plate_declaration_catches_missing_lesson() {
        let td = tempdir().unwrap();
        let root = td.path();
        // Proposal with trace_spec but no lesson.
        write(
            &root.join("strips").join("01-volatile").join("proposal.md"),
            "title: Volatile is dangerous\n\
             trace_spec: concept-curriculum\n",
        );
        let mut r = LintReport::default();
        check_plate_declaration(root, &mut r);
        assert!(
            r.violations
                .iter()
                .any(|v| v.message.contains("must declare `lesson:")),
            "expected a missing-lesson violation; got {:?}",
            r.violations
        );
    }

    #[test]
    fn plate_declaration_accepts_full_proposal() {
        let td = tempdir().unwrap();
        let root = td.path();
        write(
            &root.join("strips").join("02-save").join("proposal.md"),
            "lesson: S1-200\n\
             title: Save means findable\n\
             trace_spec: concept-curriculum\n",
        );
        let mut r = LintReport::default();
        check_plate_declaration(root, &mut r);
        assert!(r.violations.is_empty(), "got {:?}", r.violations);
    }

    // --- Slug in registry --------------------------------------------------

    #[test]
    fn slug_in_registry_flags_unregistered() {
        let td = tempdir().unwrap();
        let root = td.path();
        write(
            &root
                .join("openspec")
                .join("specs")
                .join("lessons")
                .join("spec.md"),
            "| S1-100-foo | ... |\n| S1-200-bar | ... |\n",
        );
        write(
            &root.join("notes.md"),
            "see @Lesson S1-999 and @Lesson S1-100\n",
        );
        let mut r = LintReport::default();
        check_slug_in_registry(root, &mut r);
        assert_eq!(r.violations.len(), 1);
        assert!(r.violations[0].message.contains("S1-999"));
    }

    // --- Containerfile USER ------------------------------------------------

    #[test]
    fn containerfile_user_flags_root() {
        let td = tempdir().unwrap();
        let root = td.path();
        write(
            &root.join("images").join("inference").join("Containerfile"),
            "FROM fedora-minimal\nUSER root\n",
        );
        let mut r = LintReport::default();
        check_containerfile_user(root, &mut r);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule == LintRule::NoWriteAtNonRoot));
    }

    #[test]
    fn containerfile_user_accepts_nonroot() {
        let td = tempdir().unwrap();
        let root = td.path();
        write(
            &root.join("images").join("inference").join("Containerfile"),
            "FROM fedora-minimal\nUSER 10001:10001\n",
        );
        let mut r = LintReport::default();
        check_containerfile_user(root, &mut r);
        assert!(r.violations.is_empty(), "got {:?}", r.violations);
    }

    #[test]
    fn containerfile_user_flags_missing() {
        let td = tempdir().unwrap();
        let root = td.path();
        write(
            &root.join("images").join("inference").join("Containerfile"),
            "FROM fedora-minimal\nRUN echo hi\n",
        );
        let mut r = LintReport::default();
        check_containerfile_user(root, &mut r);
        assert_eq!(r.violations.len(), 1);
    }

    // --- Licensing coverage -----------------------------------------------

    #[test]
    fn licensing_rule_match_handles_standard_extensions() {
        assert_eq!(licensing_rule_match(Path::new("foo/bar.md")), Some("R04"));
        assert_eq!(
            licensing_rule_match(Path::new("scripts/run.sh")),
            Some("R01")
        );
        assert_eq!(licensing_rule_match(Path::new("lib/run.sh")), Some("R02"));
        assert_eq!(licensing_rule_match(Path::new("a/b.png")), Some("R05"));
        assert_eq!(licensing_rule_match(Path::new("x/y.toml")), Some("R08"));
        assert_eq!(licensing_rule_match(Path::new("README.md")), Some("R11"));
        assert_eq!(licensing_rule_match(Path::new("LICENSE")), Some("R09"));
        assert_eq!(licensing_rule_match(Path::new(".gitignore")), Some("R10"));
        assert_eq!(licensing_rule_match(Path::new("weird.unknown")), None);
    }

    // --- Integration test against the real repo ---------------------------

    #[tokio::test]
    async fn verify_all_against_real_repo_has_no_canon_violations() {
        let root = tt_core::project_root();
        if !root.is_dir() {
            eprintln!("skipping: project root not found on this host");
            return;
        }
        let graph = SpecGraph::default();
        let report = verify_all(&root, &graph).await.expect("verify_all Ok");
        eprintln!(
            "real-repo lint: {} violation(s), {} warning(s)",
            report.violations.len(),
            report.warnings.len()
        );
        if report.has_violations() {
            panic!(
                "real-repo lint produced canon violations — human judgment required:\n{report}"
            );
        }
    }
}
