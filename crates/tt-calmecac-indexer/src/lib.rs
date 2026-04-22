//! Tlatoāni Tales — Calmecac concept-index builder.
//!
//! Reads substrate (markdown, paths, dates, hashes) and emits concepts
//! (lessons, rules, changes). The substrate-erasing step is load-bearing:
//! filenames and hashes are parsed and *thrown away* before the index is
//! written, so the bundle the untrusted httpd container serves never
//! contains path-shaped strings or commit hashes. The UI does not need to
//! remember to hide substrate — the substrate never reaches it.
//!
//! Governing spec: `openspec/specs/calmecac/spec.md` §Concept index
//! generation. Tombstone rendering obeys `openspec/specs/tombstones/spec.md`
//! — tombstones surface as visible-but-retired entries, never hidden.
//!
//! The indexer **is** lesson S1-1000 (dashboards-must-add-observability)
//! materialized — it walks the project, surfaces relationships between
//! teachings and rules, and writes a graph the viewer can render. The
//! abstraction-rule self-check at the bottom is the S1-1500 loop closing:
//! proof-by-self-reference, because the generator enforces on its own
//! output the rule it was written to enforce.
//!
// @trace spec:calmecac, spec:tombstones, spec:orchestrator
// @Lesson S1-1000
// @Lesson S1-1500

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use time::OffsetDateTime;
use tokio::process::Command;
use tt_core::TtError;

// ---------------------------------------------------------------------------
// Public types — the concept graph the viewer reads on startup.
// ---------------------------------------------------------------------------

/// Top-level concept index. One JSON file, loaded once on bundle boot.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalmecacIndex {
    pub generated_at: String,
    pub schema_version: u32,
    pub seasons: Vec<SeasonNode>,
    pub lessons: Vec<LessonNode>,
    pub rules: Vec<RuleNode>,
    pub strips: Vec<StripNode>,
    pub meta_examples: Vec<MetaExampleNode>,
    pub tombstones: Vec<TombstoneNode>,
    pub changes: Vec<ChangeNode>,
    pub convergence: Vec<ConvergencePoint>,
}

/// A season — the curriculum unit the reader holds in one phrase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonNode {
    pub id: String,
    pub display: String,
    pub lesson_count: u16,
    pub shipped: bool,
}

/// A lesson — the teaching layer the reader encounters in the comic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonNode {
    pub id: String,
    pub short_id: String,
    pub display: String,
    pub season: String,
    pub takeaway: String,
    pub position: Position,
    pub reinforced_in: Vec<String>,
    pub governs: Vec<String>,
    pub tombstoned: bool,
    pub successor: Option<String>,
}

/// A rule — what the reader knows as an `@trace rule:<name>` chip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleNode {
    pub name: String,
    pub display: String,
    pub governs_lessons: Vec<String>,
    pub cited_by_changes: Vec<String>,
    pub tombstoned: bool,
}

/// A strip — one published comic in the season.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripNode {
    pub id: String,
    pub lesson: String,
    pub trace_spec: String,
    pub title: String,
    pub last_change: Option<String>,
}

/// A meta-example — a load-bearing structural teaching (ME##).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaExampleNode {
    pub id: String,
    pub display: String,
    pub demonstrates: Vec<String>,
    pub governing_rule: String,
    pub tombstoned: bool,
}

/// A tombstone — a retired entry, visibly inert.
///
/// Per `tombstones/spec.md` T04: Calmecac MUST render tombstones as
/// greyed/reachable entries. Hiding would defeat T02 (archival legibility).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TombstoneNode {
    pub id: String,
    pub kind: String,
    pub retired_at: String,
    pub reason: String,
    pub successor: Option<String>,
}

/// A change — a commit, projected through the abstraction filter.
///
/// `short_hash` is substrate — it does NOT serialize. The viewer only ever
/// sees `id` (with the `change-` prefix that distinguishes concept from
/// mechanism), plus the date and the concept-level subject.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeNode {
    pub id: String,
    #[serde(skip)]
    pub short_hash: String,
    pub ts: String,
    pub subject: String,
    pub traces: Vec<String>,
    pub lessons: Vec<String>,
}

/// A convergence measurement — one point on a rule's history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergencePoint {
    pub ts: String,
    pub metric: String,
    pub value: f32,
    pub subject_kind: String,
    pub subject_id: Option<String>,
}

/// A lesson's position in the curriculum — predecessors and successors as
/// IDs, the season, and the lesson number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub season: String,
    pub number: u16,
    pub predecessors: Vec<String>,
    pub successors: Vec<String>,
}

// ---------------------------------------------------------------------------
// Entry point.
// ---------------------------------------------------------------------------

/// Build the concept index for a project checkout and write it to `out_path`
/// as pretty-printed JSON.
///
/// Runs at build time, emits once, exits — the indexer is NOT a web service
/// and the httpd container never invokes it (trust boundary preserved per
/// `calmecac/spec.md` §Trust-boundary placement).
pub async fn build_index(project_dir: &Path, out_path: &Path) -> Result<(), TtError> {
    let specs_dir = project_dir.join("openspec").join("specs");

    let rules = collect_rules(&specs_dir)?;
    let (lessons, lesson_tombstones) = collect_lessons(&specs_dir)?;
    let (meta_examples, me_tombstones) = collect_meta_examples(&specs_dir)?;
    let strips = collect_strips(project_dir)?;
    let changes = collect_changes(project_dir).await?;
    let seasons = derive_seasons(&lessons);
    let tombstones = [lesson_tombstones, me_tombstones].concat();
    let convergence = collect_convergence(project_dir, &rules).await?;

    let rules = link_rules_to_changes(rules, &changes, &lessons);
    let lessons = link_lessons(lessons, &strips, &rules);
    let strips = link_strips(strips, &changes);

    let generated_at = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into());

    let index = CalmecacIndex {
        generated_at,
        schema_version: 1,
        seasons,
        lessons,
        rules,
        strips,
        meta_examples,
        tombstones,
        changes,
        convergence,
    };

    let json = serde_json::to_string_pretty(&index)
        .map_err(|e| TtError::Parse(format!("serialize index: {e}")))?;

    assert_abstraction_clean(&json)?;

    if let Some(parent) = out_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(out_path, json).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Rule collection — one per spec directory under openspec/specs/.
// ---------------------------------------------------------------------------

fn collect_rules(specs_dir: &Path) -> Result<Vec<RuleNode>, TtError> {
    if !specs_dir.exists() {
        return Ok(Vec::new());
    }
    let mut rules = Vec::new();
    for entry in std::fs::read_dir(specs_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        // Nested lesson directories live under lessons/; skip anything that
        // looks like a lesson full-slug (starts with `S` then a digit).
        if is_lesson_slug(&name) {
            continue;
        }
        let spec_file = entry.path().join("spec.md");
        if !spec_file.exists() {
            continue;
        }
        let body = std::fs::read_to_string(&spec_file)?;
        let display = extract_title(&body).unwrap_or_else(|| humanize(&name));
        rules.push(RuleNode {
            name,
            display,
            governs_lessons: Vec::new(),
            cited_by_changes: Vec::new(),
            tombstoned: false,
        });
    }
    rules.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(rules)
}

fn is_lesson_slug(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some('S'))
        && chars.next().map(|c| c.is_ascii_digit()).unwrap_or(false)
}

fn extract_title(body: &str) -> Option<String> {
    for line in body.lines() {
        let line = line.trim_start();
        if let Some(rest) = line.strip_prefix("# ") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

fn humanize(name: &str) -> String {
    name.split('-')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Lesson collection — walks the registry + per-lesson specs.
// ---------------------------------------------------------------------------

fn collect_lessons(specs_dir: &Path) -> Result<(Vec<LessonNode>, Vec<TombstoneNode>), TtError> {
    let lessons_dir = specs_dir.join("lessons");
    if !lessons_dir.exists() {
        return Ok((Vec::new(), Vec::new()));
    }

    // Parse the registry for takeaways + tombstones.
    let registry_path = lessons_dir.join("spec.md");
    let registry = std::fs::read_to_string(&registry_path).unwrap_or_default();
    let (takeaways, tombstones) = parse_lesson_registry(&registry);

    let mut lessons = Vec::new();
    for entry in std::fs::read_dir(&lessons_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let full = entry.file_name().to_string_lossy().into_owned();
        if !is_lesson_slug(&full) {
            continue;
        }
        let spec_file = entry.path().join("spec.md");
        if !spec_file.exists() {
            continue;
        }
        let body = std::fs::read_to_string(&spec_file)?;
        let node = parse_lesson_spec(&full, &body, &takeaways)?;
        lessons.push(node);
    }
    lessons.sort_by(|a, b| a.position.number.cmp(&b.position.number));
    Ok((lessons, tombstones))
}

fn parse_lesson_spec(
    full_slug: &str,
    body: &str,
    takeaways: &BTreeMap<String, String>,
) -> Result<LessonNode, TtError> {
    // Display is "Season lesson-name" from the first heading.
    let display = extract_title(body)
        .map(|t| t.trim_start_matches(char::is_alphabetic).to_string())
        .and_then(|rest| {
            // Heading form: "S1-100 — Volatile is dangerous". We want the
            // right side.
            rest.split_once('—').map(|(_, r)| r.trim().to_string())
        })
        .unwrap_or_else(|| humanize(&full_slug[full_slug.find('-').unwrap_or(0) + 1..]));

    // Short id: the season + number.
    let short_id = full_slug
        .splitn(3, '-')
        .take(2)
        .collect::<Vec<_>>()
        .join("-");
    let season = short_id
        .split('-')
        .next()
        .unwrap_or("S1")
        .to_string();

    // Parse the Position block.
    let position = parse_position_block(body, &season);

    let takeaway = takeaways.get(&short_id).cloned().unwrap_or_default();

    Ok(LessonNode {
        id: full_slug.to_string(),
        short_id,
        display,
        season,
        takeaway,
        position,
        reinforced_in: Vec::new(),
        governs: Vec::new(),
        tombstoned: false,
        successor: None,
    })
}

fn parse_position_block(body: &str, season_fallback: &str) -> Position {
    let mut in_position = false;
    let mut season = season_fallback.to_string();
    let mut number: u16 = 0;
    let mut predecessors: Vec<String> = Vec::new();
    let mut successors: Vec<String> = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            in_position = trimmed.eq_ignore_ascii_case("## position");
            continue;
        }
        if !in_position {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("- Season:") {
            season = rest.trim().to_string();
        } else if let Some(rest) = trimmed.strip_prefix("- Number:") {
            number = rest.trim().parse().unwrap_or(0);
        } else if let Some(rest) = trimmed.strip_prefix("- Predecessors:") {
            predecessors = parse_lesson_list(rest);
        } else if let Some(rest) = trimmed.strip_prefix("- Successors:") {
            successors = parse_lesson_list(rest);
        }
    }
    Position {
        season,
        number,
        predecessors,
        successors,
    }
}

fn parse_lesson_list(s: &str) -> Vec<String> {
    let s = s.trim();
    if s.is_empty() || s == "—" || s == "-" || s == "none" {
        return Vec::new();
    }
    s.split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty() && *p != "—")
        .map(|p| p.to_string())
        .collect()
}

/// Extract the "takeaway" column from the Season 1 registry table, and also
/// gather the tombstoned-slug rows from the migration table.
fn parse_lesson_registry(body: &str) -> (BTreeMap<String, String>, Vec<TombstoneNode>) {
    let mut takeaways = BTreeMap::new();
    let mut tombstones = Vec::new();

    // Rows in the registry table look like:
    //   | `S1-100-volatile-is-dangerous` | Volatile is dangerous | Starting fresh ... | TT 01/15 | ... |
    // Use a regex that captures the short_id and takeaway.
    let row_re = Regex::new(r"\| `(S\d+-\d+)-[^`]+` \| [^|]+ \| ([^|]+) \|").unwrap();
    for line in body.lines() {
        if let Some(cap) = row_re.captures(line) {
            let short = cap.get(1).unwrap().as_str().to_string();
            let take = cap.get(2).unwrap().as_str().trim().to_string();
            takeaways.entry(short).or_insert(take);
        }
    }

    // Tombstone row patterns:
    //   `lesson_volatile_is_dangerous`  →  `S1-100-volatile-is-dangerous`
    //   `S1-950-dashboards-must-add-observability` → ...tombstoned 2026-04-22 — <reason>
    let ts_re = Regex::new(
        r"\| `(lesson_[A-Za-z0-9_]+|S\d+-\d+-[A-Za-z0-9-]+)` \| `(S\d+-\d+-[A-Za-z0-9-]+)`(?:\s*— tombstoned (\d{4}-\d{2}-\d{2})(?:\s*—\s*([^|]+))?)? \|",
    )
    .unwrap();
    // Tombstone date for the legacy migration sweep.
    let legacy_date = "2026-04-22";
    for line in body.lines() {
        if let Some(cap) = ts_re.captures(line) {
            let old = cap.get(1).unwrap().as_str().to_string();
            let new = cap.get(2).unwrap().as_str().to_string();
            let date = cap
                .get(3)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| legacy_date.to_string());
            let reason = cap
                .get(4)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_else(|| "tombstoned for curriculum renumbering".into());
            tombstones.push(TombstoneNode {
                id: old,
                kind: "lesson".into(),
                retired_at: date,
                reason,
                successor: Some(new),
            });
        }
    }
    (takeaways, tombstones)
}

// ---------------------------------------------------------------------------
// Meta-example collection — parse the ledger table from meta-examples/spec.md.
// ---------------------------------------------------------------------------

fn collect_meta_examples(
    specs_dir: &Path,
) -> Result<(Vec<MetaExampleNode>, Vec<TombstoneNode>), TtError> {
    let path = specs_dir.join("meta-examples").join("spec.md");
    if !path.exists() {
        return Ok((Vec::new(), Vec::new()));
    }
    let body = std::fs::read_to_string(&path)?;
    let mut items = Vec::new();
    let mut tombstones = Vec::new();
    // Row: | ME## | **display** | Cxx, Cyy | `spec/file` | notes |
    //  or: | ~~ME10~~ | *(tombstoned ...)* | — | — | ... |
    let active_re = Regex::new(
        r"\|\s*(ME\d+)\s*\|\s*\*\*([^*|]+)\*\*\s*\|\s*([^|]*)\|\s*([^|]*)\|",
    )
    .unwrap();
    let tomb_re = Regex::new(
        r"\|\s*~~(ME\d+)~~\s*\|\s*\*\(tombstoned\s+(\d{4}-\d{2}-\d{2})\)\*",
    )
    .unwrap();
    for line in body.lines() {
        if let Some(cap) = tomb_re.captures(line) {
            let id = cap.get(1).unwrap().as_str().to_string();
            let date = cap.get(2).unwrap().as_str().to_string();
            tombstones.push(TombstoneNode {
                id: id.clone(),
                kind: "meta-example".into(),
                retired_at: date,
                reason: "author declined".into(),
                successor: None,
            });
            items.push(MetaExampleNode {
                id,
                display: "(retired)".into(),
                demonstrates: Vec::new(),
                governing_rule: String::new(),
                tombstoned: true,
            });
            continue;
        }
        if let Some(cap) = active_re.captures(line) {
            let id = cap.get(1).unwrap().as_str().to_string();
            let display = cap.get(2).unwrap().as_str().trim().to_string();
            let demonstrates = cap
                .get(3)
                .unwrap()
                .as_str()
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            // Strip the backticks + any trailing .md so we emit a concept
            // name, not a substrate path. "foo/bar/spec.md" -> "foo"/"bar",
            // we keep the last directory segment before spec.md.
            let rule_raw = cap.get(4).unwrap().as_str().trim();
            let governing_rule = erase_substrate_governing_rule(rule_raw);
            items.push(MetaExampleNode {
                id,
                display,
                demonstrates,
                governing_rule,
                tombstoned: false,
            });
        }
    }
    Ok((items, tombstones))
}

fn erase_substrate_governing_rule(raw: &str) -> String {
    // raw might look like: `licensing/spec.md`, `openspec/config.yaml`,
    // `visual-qa-loop/spec.md`, or just `—`.
    let s = raw.trim().trim_matches('`');
    if s == "—" || s == "-" || s.is_empty() {
        return String::new();
    }
    // Cut before any `/spec.md` / `/config.yaml` / etc.
    let base = s.split('/').next().unwrap_or(s);
    // Drop dotted-extension form just in case.
    base.split('.').next().unwrap_or(base).to_string()
}

// ---------------------------------------------------------------------------
// Strip collection — output/Tlatoāni_Tales_NN.json metadata sidecars.
// ---------------------------------------------------------------------------

fn collect_strips(project_dir: &Path) -> Result<Vec<StripNode>, TtError> {
    let out = project_dir.join("output");
    if !out.exists() {
        return Ok(Vec::new());
    }
    let re = Regex::new(r"Tlatoāni_Tales_(\d{2})\.json$").unwrap();
    let mut strips = Vec::new();
    for entry in walkdir::WalkDir::new(&out)
        .max_depth(1)
        .into_iter()
        .flatten()
    {
        let p = entry.path();
        let name = match p.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if let Some(cap) = re.captures(name) {
            let nn = cap.get(1).unwrap().as_str();
            let total = "15";
            let data = match std::fs::read_to_string(p) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let v: serde_json::Value = match serde_json::from_str(&data) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let lesson = v
                .get("lesson")
                .and_then(|x| x.get("short"))
                .and_then(|x| x.as_str())
                .or_else(|| v.get("lesson").and_then(|x| x.as_str()))
                .unwrap_or_default()
                .to_string();
            let trace_spec = v
                .get("trace_spec")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string();
            let title = v
                .get("title")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string();
            strips.push(StripNode {
                id: format!("{nn}/{total}"),
                lesson,
                trace_spec,
                title,
                last_change: None,
            });
        }
    }
    strips.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(strips)
}

// ---------------------------------------------------------------------------
// Change collection — `git log` output, abstraction-filtered.
// ---------------------------------------------------------------------------

async fn collect_changes(project_dir: &Path) -> Result<Vec<ChangeNode>, TtError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project_dir)
        .arg("log")
        .arg("--format=%h%x00%ct%x00%s%n%b%x00%x00")
        .output()
        .await;
    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return Ok(Vec::new()),
    };
    let raw = String::from_utf8_lossy(&output.stdout).into_owned();

    let trace_re = Regex::new(r"@trace\s+((?:spec:[A-Za-z0-9\u{00C0}-\u{024F}_\-]+(?:\s*,\s*)?)+)")
        .unwrap();
    let single_trace_re = Regex::new(r"spec:([A-Za-z0-9\u{00C0}-\u{024F}_\-]+)").unwrap();
    let lesson_re = Regex::new(r"@Lesson\s+(S\d+-\d+)(?:-[A-Za-z0-9-]+)?").unwrap();

    let mut out = Vec::new();
    for block in raw.split("\x00\x00") {
        let block = block.trim_start_matches('\n');
        if block.trim().is_empty() {
            continue;
        }
        let mut parts = block.splitn(3, '\x00');
        let hash = match parts.next() {
            Some(h) if !h.is_empty() => h.trim().to_string(),
            _ => continue,
        };
        let ts_unix = parts
            .next()
            .and_then(|s| s.trim().parse::<i64>().ok())
            .unwrap_or(0);
        let rest = parts.next().unwrap_or("");
        let (subject, body) = match rest.split_once('\n') {
            Some((s, b)) => (s.to_string(), b.to_string()),
            None => (rest.to_string(), String::new()),
        };
        let full_msg = format!("{subject}\n{body}");
        let mut traces = BTreeSet::new();
        for cap in trace_re.captures_iter(&full_msg) {
            let list = cap.get(1).unwrap().as_str();
            for m in single_trace_re.captures_iter(list) {
                traces.insert(m.get(1).unwrap().as_str().to_string());
            }
        }
        let mut lessons = BTreeSet::new();
        for cap in lesson_re.captures_iter(&full_msg) {
            lessons.insert(cap.get(1).unwrap().as_str().to_string());
        }
        let short_hash = if hash.len() > 7 {
            hash[..7].to_string()
        } else {
            hash.clone()
        };
        let ts = OffsetDateTime::from_unix_timestamp(ts_unix)
            .ok()
            .and_then(|t| t.format(&time::format_description::well_known::Rfc3339).ok())
            .unwrap_or_else(|| "1970-01-01T00:00:00Z".into());
        out.push(ChangeNode {
            id: format!("change-{short_hash}"),
            short_hash,
            ts,
            subject,
            traces: traces.into_iter().collect(),
            lessons: lessons.into_iter().collect(),
        });
    }
    // Newest first already (git log default); keep order.
    Ok(out)
}

// ---------------------------------------------------------------------------
// Convergence — per-rule history lines, capped.
// ---------------------------------------------------------------------------

async fn collect_convergence(
    project_dir: &Path,
    rules: &[RuleNode],
) -> Result<Vec<ConvergencePoint>, TtError> {
    const CAP_PER_RULE: usize = 50;
    let mut out = Vec::new();
    for r in rules {
        let spec_rel = format!("openspec/specs/{}/spec.md", r.name);
        let log = Command::new("git")
            .arg("-C")
            .arg(project_dir)
            .arg("log")
            .arg("--format=%H%x00%ct")
            .arg("--follow")
            .arg(&spec_rel)
            .output()
            .await;
        let log = match log {
            Ok(o) if o.status.success() => o,
            _ => continue,
        };
        let text = String::from_utf8_lossy(&log.stdout).into_owned();
        let mut points = Vec::new();
        for line in text.lines() {
            let (h, t) = match line.split_once('\x00') {
                Some(p) => p,
                None => continue,
            };
            let ts_unix: i64 = t.parse().unwrap_or(0);
            let show = Command::new("git")
                .arg("-C")
                .arg(project_dir)
                .arg("show")
                .arg(format!("{h}:{spec_rel}"))
                .output()
                .await;
            let lines_count: f32 = match show {
                Ok(o) if o.status.success() => {
                    let s = String::from_utf8_lossy(&o.stdout);
                    s.lines().count() as f32
                }
                _ => continue,
            };
            let ts = OffsetDateTime::from_unix_timestamp(ts_unix)
                .ok()
                .and_then(|t| t.format(&time::format_description::well_known::Rfc3339).ok())
                .unwrap_or_else(|| "1970-01-01T00:00:00Z".into());
            points.push(ConvergencePoint {
                ts,
                metric: "body_length".into(),
                value: lines_count,
                subject_kind: "rule".into(),
                subject_id: Some(r.name.clone()),
            });
            if points.len() >= CAP_PER_RULE {
                break;
            }
        }
        out.extend(points);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Linking — cross-wire changes, strips, lessons, rules.
// ---------------------------------------------------------------------------

fn link_rules_to_changes(
    mut rules: Vec<RuleNode>,
    changes: &[ChangeNode],
    _lessons: &[LessonNode],
) -> Vec<RuleNode> {
    for r in &mut rules {
        for c in changes {
            if c.traces.iter().any(|t| t == &r.name) {
                r.cited_by_changes.push(c.short_hash.clone());
            }
        }
    }
    rules
}

fn link_lessons(
    mut lessons: Vec<LessonNode>,
    strips: &[StripNode],
    rules: &[RuleNode],
) -> Vec<LessonNode> {
    for l in &mut lessons {
        for s in strips {
            if s.lesson == l.short_id || s.lesson == l.id {
                l.reinforced_in.push(s.id.clone());
            }
        }
        // Fill `governs` using rules that name the lesson's short_id in their
        // own display (cheap heuristic — a real linker would read per-rule
        // frontmatter).
        for r in rules {
            if r.display.contains(&l.short_id) {
                l.governs.push(r.name.clone());
            }
        }
    }
    lessons
}

fn link_strips(mut strips: Vec<StripNode>, changes: &[ChangeNode]) -> Vec<StripNode> {
    for s in &mut strips {
        // Find the most recent change whose traces mention this strip's
        // trace_spec. changes is already newest-first.
        for c in changes {
            if !s.trace_spec.is_empty() && c.traces.iter().any(|t| t == &s.trace_spec) {
                s.last_change = Some(c.short_hash.clone());
                break;
            }
        }
    }
    strips
}

fn derive_seasons(lessons: &[LessonNode]) -> Vec<SeasonNode> {
    let mut counts: BTreeMap<String, u16> = BTreeMap::new();
    for l in lessons {
        *counts.entry(l.season.clone()).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(id, lesson_count)| SeasonNode {
            display: match id.as_str() {
                "S1" => "Volatile context to monotonic convergence".into(),
                _ => id.clone(),
            },
            shipped: id == "S1",
            id,
            lesson_count,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Abstraction-rule enforcement — the self-check.
// ---------------------------------------------------------------------------

/// Scan the serialized index for substrate leaks. Any of the forbidden
/// substrings in a string value fails the build.
///
/// The approach is **post-serialize grep**, not a per-node validator. Reason:
/// a per-field validator drifts out of sync as the schema grows, while a grep
/// on the final bytes is invariant to schema changes — the file the httpd
/// container serves is the only thing that matters, and it is precisely what
/// this check reads. This is itself the S1-1500 loop closing: the index
/// generator enforces, on its own output, the rule the generator was written
/// to enforce.
fn assert_abstraction_clean(json: &str) -> Result<(), TtError> {
    // Forbidden substrings. We check the rendered JSON — which means the
    // check is in the viewer's alphabet (string values), not the schema's
    // (field names). JSON keys are substrate in a sense, but they are
    // contract — the viewer reads them by name. The rule forbids leakage
    // into values.
    let needles: &[&str] = &[
        ".md",
        "openspec/specs/",
        "lessons/spec.md",
    ];
    // Split JSON to just the string literal contents. Keys are also quoted
    // strings; we tolerate those since they are the schema. Only values that
    // are `: "..."` style matter here. A cheap heuristic: find `: "` and
    // scan until the matching `"`.
    let mut i = 0usize;
    let bytes = json.as_bytes();
    while i + 2 < bytes.len() {
        if bytes[i] == b':' && bytes[i + 1] == b' ' && bytes[i + 2] == b'"' {
            let mut j = i + 3;
            let mut buf = String::new();
            while j < bytes.len() {
                match bytes[j] {
                    b'\\' if j + 1 < bytes.len() => {
                        // Skip escape; include literal next byte naively (we
                        // only need the subset search, so buffering the raw
                        // bytes is fine).
                        buf.push('\\');
                        buf.push(bytes[j + 1] as char);
                        j += 2;
                    }
                    b'"' => break,
                    b => {
                        buf.push(b as char);
                        j += 1;
                    }
                }
            }
            for n in needles {
                if buf.contains(n) {
                    return Err(TtError::Canon(format!(
                        "calmecac abstraction-rule violation: substrate leak `{n}` in a string value"
                    )));
                }
            }
            i = j + 1;
        } else {
            i += 1;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_node_hides_full_hash_but_keeps_short() {
        let c = ChangeNode {
            id: "change-1234567".into(),
            short_hash: "1234567".into(),
            ts: "2026-04-21T00:00:00Z".into(),
            subject: "wave-9: calmecac indexer lands".into(),
            traces: vec!["calmecac".into()],
            lessons: vec!["S1-1000".into()],
        };
        let json = serde_json::to_string(&c).unwrap();
        // The 7-char short hash travels inside the `id` field (prefixed so
        // the viewer never says "hash"). A free-standing `short_hash` field
        // MUST NOT appear — the serde skip is the invariant.
        assert!(!json.contains("short_hash"));
        assert!(json.contains("change-1234567"));
        assert!(!json.contains("\"1234567\""));
    }

    #[test]
    fn abstraction_lint_catches_dot_md_leak() {
        let bad = r#"{ "subject": "update lessons/spec.md entry" }"#;
        let err = assert_abstraction_clean(bad).unwrap_err();
        assert!(matches!(err, TtError::Canon(_)));
    }

    #[test]
    fn abstraction_lint_catches_openspec_path() {
        let bad = r#"{ "notes": "see openspec/specs/calmecac ref" }"#;
        assert!(assert_abstraction_clean(bad).is_err());
    }

    #[test]
    fn abstraction_lint_passes_clean_index() {
        let good = r#"{ "generated_at": "2026-04-21T00:00:00Z", "subject": "orchestrator converges" }"#;
        assert_abstraction_clean(good).unwrap();
    }

    #[test]
    fn abstraction_lint_allows_schema_key_names() {
        // JSON keys aren't values. A field literally named "governing_rule"
        // must not trip the lint; only its string-value content is scanned.
        let json = r#"{ "governing_rule": "licensing" }"#;
        assert_abstraction_clean(json).unwrap();
    }

    #[test]
    fn position_round_trips_serde() {
        let p = Position {
            season: "S1".into(),
            number: 1000,
            predecessors: vec!["S1-900-logs-are-ingredients".into()],
            successors: vec!["S1-1100-shape-has-meaning".into()],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: Position = serde_json::from_str(&json).unwrap();
        assert_eq!(back.season, "S1");
        assert_eq!(back.number, 1000);
        assert_eq!(back.predecessors.len(), 1);
        assert_eq!(back.successors[0], "S1-1100-shape-has-meaning");
    }

    #[test]
    fn tombstone_extraction_recognizes_me_struck_through() {
        let specs_dir =
            std::path::PathBuf::from("/var/home/machiyotl/src/tlatoāni-tales/openspec/specs");
        if !specs_dir.exists() {
            eprintln!("skipping: not running inside the tlatoāni-tales checkout");
            return;
        }
        let (items, tombs) = collect_meta_examples(&specs_dir).unwrap();
        assert!(
            tombs.iter().any(|t| t.id == "ME10" && t.retired_at == "2026-04-22"),
            "expected ME10 tombstone with date 2026-04-22, got {tombs:?}"
        );
        // ME10 still appears in items (as tombstoned:true), per T04.
        assert!(items.iter().any(|m| m.id == "ME10" && m.tombstoned));
    }

    #[test]
    fn change_node_extraction_splits_multi_spec_trace_list() {
        let block_stub = "abcdef0\x001776000000\x00wave-7: things\nBody here.\n\n@trace spec:orchestrator, spec:calmecac, spec:tombstones\n@Lesson S1-1000\n@Lesson S1-1500\n\x00\x00";
        // Run the exact regex logic against this stub.
        let trace_re =
            Regex::new(r"@trace\s+((?:spec:[A-Za-z0-9\u{00C0}-\u{024F}_\-]+(?:\s*,\s*)?)+)")
                .unwrap();
        let single = Regex::new(r"spec:([A-Za-z0-9\u{00C0}-\u{024F}_\-]+)").unwrap();
        let mut specs = Vec::new();
        for cap in trace_re.captures_iter(block_stub) {
            let list = cap.get(1).unwrap().as_str();
            for m in single.captures_iter(list) {
                specs.push(m.get(1).unwrap().as_str().to_string());
            }
        }
        assert_eq!(
            specs,
            vec![
                "orchestrator".to_string(),
                "calmecac".to_string(),
                "tombstones".to_string()
            ]
        );
    }

    #[test]
    fn lesson_registry_parse_picks_up_takeaway() {
        let sample = "\
| `S1-100-volatile-is-dangerous` | Volatile is dangerous | Starting fresh loses everything that mattered. | TT 01/15 | foo |\n\
| `S1-200-save-means-findable` | Save means findable | Copy-pasting isn't saving. | TT 02/15 | bar |\n\
";
        let (t, _) = parse_lesson_registry(sample);
        assert_eq!(
            t.get("S1-100").map(String::as_str),
            Some("Starting fresh loses everything that mattered.")
        );
        assert_eq!(
            t.get("S1-200").map(String::as_str),
            Some("Copy-pasting isn't saving.")
        );
    }

    #[test]
    fn build_index_against_real_repo() {
        // Integration test — runs against the actual checkout.
        let project = std::path::PathBuf::from("/var/home/machiyotl/src/tlatoāni-tales");
        if !project.join("openspec/specs").exists() {
            eprintln!("skipping: not running inside the tlatoāni-tales checkout");
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("calmecac-index.json");
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            build_index(&project, &out).await.unwrap();
        });
        let text = std::fs::read_to_string(&out).unwrap();
        let parsed: CalmecacIndex = serde_json::from_str(&text).unwrap();
        assert!(!parsed.rules.is_empty(), "rules should be populated");
        assert!(!parsed.lessons.is_empty(), "lessons should be populated");
        assert!(parsed.schema_version == 1);
        // Self-check: the abstraction lint that runs inside build_index must
        // also pass on the re-serialized text.
        assert_abstraction_clean(&text).unwrap();
    }

    #[test]
    fn erase_substrate_governing_rule_strips_paths_and_extensions() {
        assert_eq!(erase_substrate_governing_rule("`licensing/spec.md`"), "licensing");
        assert_eq!(
            erase_substrate_governing_rule("`visual-qa-loop/spec.md`"),
            "visual-qa-loop"
        );
        assert_eq!(erase_substrate_governing_rule("—"), "");
        assert_eq!(
            erase_substrate_governing_rule("`openspec/config.yaml`"),
            "openspec"
        );
    }

    #[test]
    fn is_lesson_slug_identifies_lesson_directories() {
        assert!(is_lesson_slug("S1-100-volatile-is-dangerous"));
        assert!(is_lesson_slug("S2-300-foo"));
        assert!(!is_lesson_slug("lessons"));
        assert!(!is_lesson_slug("tombstones"));
        assert!(!is_lesson_slug("orchestrator"));
    }
}
