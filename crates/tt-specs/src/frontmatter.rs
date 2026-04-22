//! YAML frontmatter splitting for spec/proposal markdown files.
//!
//! A file may begin with a `---\n<yaml>\n---\n` fenced block. If so, we
//! parse the YAML into a typed [`Frontmatter`] and hand back the markdown
//! body that follows. If the file starts with ordinary markdown, the
//! frontmatter is `None` and the body is the whole file — Tlatoāni Tales'
//! current spec prose sits in this second mode.
//!
//! Governing specs: `openspec/specs/lessons/spec.md`,
//! `openspec/specs/lesson-driven-development/spec.md`.
//
// @trace spec:lessons, spec:lesson-driven-development
// @Lesson S1-400

use serde::{Deserialize, Serialize};

/// Parsed YAML frontmatter. Flexible — callers pluck fields by name
/// rather than pattern-matching a closed struct, since per-surface
/// schemas (spec vs. strip proposal) differ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    /// Raw YAML value. `serde_yaml::Value::Mapping` in practice.
    pub raw: serde_yaml::Value,
}

impl Frontmatter {
    /// Wrap an already-parsed `serde_yaml::Value`.
    pub fn from_value(raw: serde_yaml::Value) -> Self {
        Self { raw }
    }

    /// Look up a top-level string field. Returns `None` if the key is
    /// missing, not a string, or the raw value is not a mapping.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.raw.as_mapping()?.get(serde_yaml::Value::String(key.into()))?.as_str()
    }

    /// Look up a top-level list-of-strings field. Returns `None` when
    /// the key is absent or not a sequence; non-string members are
    /// skipped silently.
    pub fn get_list_str(&self, key: &str) -> Option<Vec<String>> {
        let seq = self
            .raw
            .as_mapping()?
            .get(serde_yaml::Value::String(key.into()))?
            .as_sequence()?;
        Some(
            seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
        )
    }
}

/// Split a file's contents into optional frontmatter + body.
///
/// If `content` starts with `---` on its own line, look for the closing
/// `---` on another line; the YAML between the fences is parsed. On
/// parse failure the whole input is returned as the body with `None`
/// for frontmatter — tolerant by design: spec prose with an incidental
/// `---` horizontal rule at file start is unlikely but survivable.
pub fn split(content: &str) -> (Option<Frontmatter>, String) {
    // Accept `---\n` or `---\r\n` at the very start.
    let rest = if let Some(r) = content.strip_prefix("---\n") {
        r
    } else if let Some(r) = content.strip_prefix("---\r\n") {
        r
    } else {
        return (None, content.to_string());
    };

    // Find the closing fence — a line containing exactly `---` (no
    // trailing content on that line).
    let mut end: Option<usize> = None;
    let mut cursor = 0usize;
    for line in rest.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if trimmed == "---" {
            end = Some(cursor);
            break;
        }
        cursor += line.len();
    }

    let Some(fence_start) = end else {
        // No closing fence — treat whole file as body, no frontmatter.
        return (None, content.to_string());
    };

    let yaml_src = &rest[..fence_start];
    // Skip the `---` closing line and any CR/LF after it.
    let after_fence = &rest[fence_start..];
    let after_fence = after_fence
        .strip_prefix("---\n")
        .or_else(|| after_fence.strip_prefix("---\r\n"))
        .unwrap_or_else(|| after_fence.trim_start_matches("---"));

    match serde_yaml::from_str::<serde_yaml::Value>(yaml_src) {
        Ok(val) => (Some(Frontmatter::from_value(val)), after_fence.to_string()),
        Err(_) => (None, content.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_leading_fenced_yaml() {
        let src = "---\nfoo: 1\nbar: hi\n---\n# Title\n\nBody.\n";
        let (fm, body) = split(src);
        let fm = fm.expect("expected frontmatter");
        assert_eq!(fm.get_str("bar"), Some("hi"));
        assert!(body.starts_with("# Title"));
    }

    #[test]
    fn returns_none_when_no_fence() {
        let src = "# Title\n\nBody.\n";
        let (fm, body) = split(src);
        assert!(fm.is_none());
        assert_eq!(body, src);
    }

    #[test]
    fn tolerant_of_parse_failure() {
        // A YAML tab-indentation error should not blow up the loader.
        let src = "---\n\tfoo: 1\n---\n# Title\n";
        let (fm, body) = split(src);
        assert!(fm.is_none());
        assert_eq!(body, src);
    }

    #[test]
    fn get_list_str_works() {
        let src = "---\ntags:\n  - a\n  - b\n  - 3\n---\nbody";
        let (fm, _) = split(src);
        let fm = fm.expect("fm");
        assert_eq!(fm.get_list_str("tags"), Some(vec!["a".to_string(), "b".to_string()]));
    }

    #[test]
    fn no_closing_fence_is_not_frontmatter() {
        let src = "---\nfoo: 1\n# forgot closing fence\n";
        let (fm, body) = split(src);
        assert!(fm.is_none());
        assert_eq!(body, src);
    }
}
