//! Tlatoāni Tales — shared type foundation.
//!
//! This crate exports the newtype wrappers, error taxonomy, podman hardening
//! flag constants, and a handful of path helpers used by the rest of the
//! `tt-*` workspace. Nothing here performs I/O; every other crate depends on
//! these types for its public surface.
//!
//! Governing spec: `openspec/specs/orchestrator/spec.md`. See also
//! `isolation/spec.md`, `lessons/spec.md`, `seasons/spec.md`, and
//! `tombstones/spec.md` for the identity and boundary contracts encoded here.
//!
// @trace spec:orchestrator, spec:isolation, spec:lessons, spec:seasons, spec:tombstones
// @Lesson S1-1300

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

pub mod podman;

// ---------------------------------------------------------------------------
// Error taxonomy
// ---------------------------------------------------------------------------

/// Classification of a failure — the orchestrator exit codes hinge on this.
///
/// See `openspec/specs/orchestrator/spec.md` §Failure modes for the canon.
/// `Infra` → exit 30, `InfraSubcode` → exit 31 (bind-mount permission denied
/// sub-code), `Canon` → exit 10, `CanonNeedsHuman` → exit 20, `Usage` → exit
/// 2. Binary crates read this via `TtError::class` to pick the process exit
/// code.
// @trace spec:orchestrator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureClass {
    /// Canon failure — the comic is wrong (drift past threshold, tombstoned
    /// lesson referenced, spec invariant broken).
    Canon,
    /// Canon failure escalated to a human author — rerolls exhausted.
    CanonNeedsHuman,
    /// Infra failure — the tool is wrong (container unreachable, model
    /// missing, bind-mount denied).
    Infra,
    /// Infra sub-code — bind-mount permission denied specifically.
    InfraSubcode,
    /// Usage error — bad args, wrong invocation.
    Usage,
}

impl FailureClass {
    /// Process exit code associated with this failure class.
    ///
    /// Mirrors `orchestrator/spec.md` §Exit codes.
    pub fn exit_code(&self) -> u8 {
        match self {
            FailureClass::Canon => 10,
            FailureClass::CanonNeedsHuman => 20,
            FailureClass::Infra => 30,
            FailureClass::InfraSubcode => 31,
            FailureClass::Usage => 2,
        }
    }
}

/// Canonical error taxonomy for Tlatoāni Tales library crates.
///
/// Binary crates lean on `anyhow::Error` at the edges and preserve the
/// `FailureClass` via context. Every variant answers the question *"which
/// exit code does this produce?"* via [`TtError::class`].
// @trace spec:orchestrator
// @Lesson S1-1300
#[derive(Debug, thiserror::Error)]
pub enum TtError {
    /// Lesson identifier is malformed (format or character violation).
    #[error("invalid lesson id `{0}`: {1}")]
    InvalidLessonId(String, String),

    /// Spec name is malformed (kebab-case / path / extension violation).
    #[error("invalid spec name `{0}`: {1}")]
    InvalidSpecName(String, String),

    /// Strip id is outside the legal range (zero rejected).
    #[error("invalid strip id `{0}`: {1}")]
    InvalidStripId(u16, String),

    /// A hex panel hash failed to parse.
    #[error("invalid panel hash: {0}")]
    InvalidHash(String),

    /// Attempted to use a zone/role in a position the spec forbids.
    #[error("zone/role misuse: {0}")]
    ZoneMisuse(String),

    /// A `podman run` invocation is missing one of the canonical hardening
    /// flags. See `isolation/spec.md` §Canonical flags.
    #[error("podman flag lint failure: {0}")]
    PodmanFlagLint(String),

    /// Infrastructure failure (container unreachable, model missing, etc.).
    #[error("infra failure: {0}")]
    Infra(String),

    /// Infra sub-code — bind-mount permission denied.
    #[error("infra (bind-mount permission denied): {0}")]
    InfraPermissionDenied(String),

    /// Canon failure — the comic is wrong.
    #[error("canon failure: {0}")]
    Canon(String),

    /// Canon failure escalated to a human author.
    #[error("canon needs-human: {0}")]
    CanonNeedsHuman(String),

    /// Usage error (bad args, subcommand mis-spelled).
    #[error("usage error: {0}")]
    Usage(String),

    /// Parse error — consuming crates map specific failures here.
    #[error("parse error: {0}")]
    Parse(String),

    /// Underlying I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl TtError {
    /// The failure class that governs the exit code for this error.
    ///
    /// Binary crates read the class, compose whatever logging they want, then
    /// `std::process::exit(class.exit_code())`.
    pub fn class(&self) -> FailureClass {
        match self {
            TtError::InvalidLessonId(..)
            | TtError::InvalidSpecName(..)
            | TtError::InvalidStripId(..)
            | TtError::InvalidHash(_)
            | TtError::PodmanFlagLint(_)
            | TtError::Canon(_)
            | TtError::Parse(_) => FailureClass::Canon,
            TtError::CanonNeedsHuman(_) => FailureClass::CanonNeedsHuman,
            TtError::Infra(_) | TtError::Io(_) => FailureClass::Infra,
            TtError::InfraPermissionDenied(_) => FailureClass::InfraSubcode,
            TtError::Usage(_) | TtError::ZoneMisuse(_) => FailureClass::Usage,
        }
    }
}

impl From<hex::FromHexError> for TtError {
    fn from(e: hex::FromHexError) -> Self {
        TtError::InvalidHash(e.to_string())
    }
}

/// Convenience alias used throughout the Tlatoāni Tales workspace.
pub type Result<T> = std::result::Result<T, TtError>;

// ---------------------------------------------------------------------------
// SeasonId
// ---------------------------------------------------------------------------

/// Season identifier (Season 1 → `SeasonId { n: 1 }`).
///
/// Canonical form per `openspec/specs/seasons/spec.md` — a season is a
/// coherent thesis, not a marketing unit. Season numbers step by 1 starting
/// at 1; `Display` renders as `S<n>`.
// @trace spec:seasons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SeasonId {
    /// Season number — 1 for Season 1, 2 for Season 2, and so on.
    pub n: u16,
}

impl SeasonId {
    /// Construct a season id. Zero is rejected — there is no Season 0.
    pub fn new(n: u16) -> Result<Self> {
        if n == 0 {
            return Err(TtError::Parse(
                "season number must be >= 1 (there is no Season 0)".into(),
            ));
        }
        Ok(Self { n })
    }
}

impl fmt::Display for SeasonId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "S{}", self.n)
    }
}

// ---------------------------------------------------------------------------
// LessonId
// ---------------------------------------------------------------------------

/// Identifier for a published lesson, e.g.
/// `"S1-100-volatile-is-dangerous"`.
///
/// Grammar per `openspec/specs/lessons/spec.md` §Canonical naming:
///
/// ```text
/// S<n>-<NNN>-<slug>
/// ```
///
/// where `<n>` is the season number (≥ 1), `<NNN>` is the lesson index
/// stepped by 100 starting at 100, and `<slug>` is a kebab-case phrase —
/// lowercase ASCII letters, digits, hyphens, and nothing else.
///
/// The short form `S<n>-<NNN>` (grep-friendly code citations) is exposed via
/// [`LessonId::short`]; the slug alone via [`LessonId::slug`].
///
/// Tombstoned legacy `lesson_<snake>` slugs are deliberately rejected — see
/// `lessons/spec.md` §Tombstoned old-slug form.
// @trace spec:lessons
// @Lesson S1-1300
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LessonId(String);

impl LessonId {
    /// Parse and validate a lesson id from its canonical full form
    /// `S<n>-<NNN>-<slug>`.
    ///
    /// Rejects the tombstoned `lesson_<snake>` form and any slug containing
    /// non-kebab-case characters.
    pub fn new(s: &str) -> Result<Self> {
        let invalid = |msg: &str| TtError::InvalidLessonId(s.to_string(), msg.to_string());

        if s.is_empty() {
            return Err(invalid("empty"));
        }
        if s.starts_with("lesson_") {
            return Err(invalid(
                "tombstoned `lesson_<snake>` form — use `S<n>-<NNN>-<slug>` instead",
            ));
        }

        // Split into exactly three parts: "S<n>" "<NNN>" "<slug…>".
        // The slug itself may contain hyphens, so we split only on the first two.
        let mut it = s.splitn(3, '-');
        let season_part = it.next().ok_or_else(|| invalid("missing season"))?;
        let number_part = it.next().ok_or_else(|| invalid("missing number"))?;
        let slug_part = it.next().ok_or_else(|| invalid("missing slug"))?;

        // Season: "S<n>", n >= 1.
        let rest = season_part
            .strip_prefix('S')
            .ok_or_else(|| invalid("season part must start with `S`"))?;
        let season_n: u16 = rest
            .parse()
            .map_err(|_| invalid("season number is not a u16"))?;
        if season_n == 0 {
            return Err(invalid("season must be >= 1"));
        }

        // Number: exactly NNN digits (we tolerate 3 or 4 digits to admit
        // S1-1000..S1-1500 while still rejecting garbage).
        if number_part.is_empty() || !number_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(invalid("lesson number must be all ASCII digits"));
        }
        if number_part.len() < 3 || number_part.len() > 4 {
            return Err(invalid("lesson number must be 3 or 4 digits"));
        }
        let number_n: u16 = number_part
            .parse()
            .map_err(|_| invalid("lesson number is not a u16"))?;
        if number_n == 0 {
            return Err(invalid("lesson number must be >= 100"));
        }

        // Slug: non-empty kebab-case ([a-z0-9-]+, no leading/trailing/double hyphen).
        if slug_part.is_empty() {
            return Err(invalid("empty slug"));
        }
        if slug_part.starts_with('-') || slug_part.ends_with('-') {
            return Err(invalid("slug must not begin or end with `-`"));
        }
        if slug_part.contains("--") {
            return Err(invalid("slug must not contain `--`"));
        }
        if !slug_part
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(invalid(
                "slug must be kebab-case (lowercase ASCII letters, digits, hyphens only)",
            ));
        }

        Ok(LessonId(s.to_string()))
    }

    /// Short, grep-friendly form used in code citations (e.g. `"S1-100"`).
    ///
    /// Per `lessons/spec.md` §`@Lesson` citation forms — this is the form for
    /// code comments and commit messages. The full form is for plates and
    /// captions; see [`LessonId::as_str`].
    pub fn short(&self) -> &str {
        // Two hyphens bound the short form: "S<n>-<NNN>-<slug…>".
        let mut hyphens = 0usize;
        for (i, b) in self.0.bytes().enumerate() {
            if b == b'-' {
                hyphens += 1;
                if hyphens == 2 {
                    return &self.0[..i];
                }
            }
        }
        &self.0
    }

    /// The slug portion (everything after `S<n>-<NNN>-`).
    pub fn slug(&self) -> &str {
        let mut hyphens = 0usize;
        for (i, b) in self.0.bytes().enumerate() {
            if b == b'-' {
                hyphens += 1;
                if hyphens == 2 {
                    return &self.0[i + 1..];
                }
            }
        }
        ""
    }

    /// The governing [`SeasonId`]. Infallible post-construction because
    /// validation already accepted the season number.
    pub fn season(&self) -> SeasonId {
        // Parse the "S<n>" prefix — validation ensures this succeeds.
        let up_to_hyphen = self.0.split('-').next().unwrap_or("S1");
        let n: u16 = up_to_hyphen.trim_start_matches('S').parse().unwrap_or(1);
        SeasonId { n }
    }

    /// The lesson number (e.g. `100` for `S1-100-volatile-is-dangerous`).
    pub fn number(&self) -> u16 {
        let mut parts = self.0.splitn(3, '-');
        let _ = parts.next();
        parts.next().and_then(|s| s.parse().ok()).unwrap_or(0)
    }

    /// Full canonical string form.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LessonId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ---------------------------------------------------------------------------
// SpecName
// ---------------------------------------------------------------------------

/// Name of an OpenSpec spec — the thing a `@trace spec:<name>` annotation
/// cites.
///
/// Shape: kebab-case, UTF-8 accepted (the Tlatoāni project deliberately does
/// not impose ASCII-only on spec names — see the rust-preference and spelling
/// memory feedback). Examples: `"orchestrator"`, `"visual-qa-loop"`,
/// `"tlatoāni-spelling"`. Rejected: anything containing a path separator, an
/// extension, whitespace, or that is empty.
// @trace spec:orchestrator
// @Lesson S1-1500
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SpecName(String);

impl SpecName {
    /// Parse and validate a spec name.
    pub fn new(s: &str) -> Result<Self> {
        let invalid = |msg: &str| TtError::InvalidSpecName(s.to_string(), msg.to_string());

        if s.is_empty() {
            return Err(invalid("empty"));
        }
        if s.chars().any(char::is_whitespace) {
            return Err(invalid("must not contain whitespace"));
        }
        if s.contains('/') || s.contains('\\') {
            return Err(invalid("must not contain path separators"));
        }
        if s.contains('.') {
            return Err(invalid("must not contain `.` (no extensions, no dotted paths)"));
        }
        if s.starts_with('-') || s.ends_with('-') {
            return Err(invalid("must not begin or end with `-`"));
        }
        if s.contains("--") {
            return Err(invalid("must not contain `--`"));
        }
        // Kebab-case over UTF-8: no ASCII uppercase, no underscores, each code
        // point must be either alphanumeric (UTF-8-aware) or a hyphen.
        for c in s.chars() {
            if c == '-' {
                continue;
            }
            if c.is_alphanumeric() && !c.is_uppercase() {
                continue;
            }
            return Err(invalid(
                "must be kebab-case over UTF-8 (lowercase letters, digits, hyphens; macrons OK)",
            ));
        }
        Ok(SpecName(s.to_string()))
    }

    /// Full canonical string form.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SpecName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ---------------------------------------------------------------------------
// PanelHash
// ---------------------------------------------------------------------------

/// Content-addressed hash (SHA-256) for a rendered panel or other cache cell.
///
/// Renders as lowercase hex via [`Display`] and [`Debug`]. Serde form is the
/// hex string — `PanelHash` rides a human-readable JSON line through
/// telemetry with no loss. Content addressing is the project's CRDT
/// materialised (`@Lesson S1-500`, `@Lesson S1-1400`).
// @trace spec:orchestrator, spec:hashing
// @Lesson S1-1400
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PanelHash([u8; 32]);

impl PanelHash {
    /// Wrap a raw 32-byte SHA-256 digest.
    pub fn from_bytes(b: [u8; 32]) -> Self {
        Self(b)
    }

    /// Parse a 64-character lowercase-hex SHA-256 digest.
    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(TtError::InvalidHash(format!(
                "expected 32 bytes (64 hex chars), got {}",
                bytes.len()
            )));
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(Self(out))
    }

    /// Render as lowercase hex.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Access the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for PanelHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

impl fmt::Debug for PanelHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PanelHash({})", hex::encode(self.0))
    }
}

impl Serialize for PanelHash {
    fn serialize<S>(&self, ser: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ser.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for PanelHash {
    fn deserialize<D>(de: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(de)?;
        PanelHash::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// StripId
// ---------------------------------------------------------------------------

/// Strip identifier — the 1-based episode number (e.g. 01..15 for Season 1).
///
/// Zero is rejected to close off an entire class of off-by-one bugs. Renders
/// zero-padded to two digits via [`Display`] (`"01"`, `"15"`).
// @trace spec:trace-plate, spec:orchestrator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StripId(u16);

impl StripId {
    /// Construct a strip id. Rejects 0.
    pub fn new(n: u16) -> Result<Self> {
        if n == 0 {
            return Err(TtError::InvalidStripId(
                n,
                "strip numbers are 1-based; 0 is not a valid strip".into(),
            ));
        }
        Ok(Self(n))
    }

    /// Raw numeric value.
    pub fn as_u16(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for StripId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Zone & Role
// ---------------------------------------------------------------------------

/// The security zone a piece of code lives in.
///
/// Governed by `openspec/specs/isolation/spec.md`. The orchestrator's crates
/// run in `Trusted`; ComfyUI / ai-toolkit / ollama / httpd runtimes run in
/// `Untrusted(Role)` containers.
// @trace spec:isolation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Zone {
    /// Our Rust code, inside the `tlatoani-tales` toolbox.
    Trusted,
    /// A hardened disposable container hosting untrusted code.
    Untrusted(Role),
}

/// Role an untrusted container is playing in the render pipeline.
///
/// Container names for each role are ASCII-only — a catalogued teachable
/// break (**TB03** in `tlatoāni-spelling/spec.md`): Podman's name grammar is
/// LDH-only, so `Tlatoāni` becomes `tlatoani` in the container namespace.
/// That gap is evidence, not oversight.
// @trace spec:isolation, spec:tlatoāni-spelling
// @Lesson S1-1500
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// ComfyUI + FLUX + Qwen-Image + ollama VLM.
    Inference,
    /// ai-toolkit LoRA trainer.
    Trainer,
    /// Distribution-standard httpd serving the Calmecac bundle.
    Viewer,
}

impl Role {
    /// Canonical container name for this role. ASCII-only per TB03.
    pub fn container_name(&self) -> &'static str {
        match self {
            Role::Inference => "tlatoani-tales-inference",
            Role::Trainer => "tlatoani-tales-trainer",
            Role::Viewer => "tlatoani-tales-viewer",
        }
    }
}

// ---------------------------------------------------------------------------
// Traits
// ---------------------------------------------------------------------------

/// Types that carry an optional `@trace spec:<name>` tag.
///
/// Every event on the orchestrator bus implements this — filtering the bus
/// by spec is how telemetry, CLI UI, and Calmecac live-watch each select the
/// slice of reality they care about.
// @trace spec:orchestrator
pub trait SpecTag {
    /// The spec tag governing this value, if any.
    fn spec_tag(&self) -> Option<&SpecName>;
}

/// Types that carry an optional `@Lesson <Sn-NNN>` tag.
///
/// Mirror of [`SpecTag`] for the reader-facing lesson citation.
// @trace spec:lessons
pub trait LessonTag {
    /// The lesson this value is in service of, if any.
    fn lesson_tag(&self) -> Option<&LessonId>;
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// Absolute path of the Tlatoāni Tales project root.
///
/// Computes upward from `CARGO_MANIFEST_DIR` (which always points at
/// `crates/tt-core/`) by taking its grandparent. Falls back to the
/// `TLATOANI_TALES_ROOT` env var or the author's canonical path if the
/// manifest dir is not available at runtime (e.g. in a distributed binary).
pub fn project_root() -> PathBuf {
    if let Ok(env) = std::env::var("TLATOANI_TALES_ROOT") {
        return PathBuf::from(env);
    }
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // crates/tt-core/ -> ../../
    Path::new(manifest_dir)
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("/var/home/machiyotl/src/tlatoāni-tales"))
}

/// `{project_root}/output/` — rendered PNGs + metadata sidecars land here.
pub fn output_dir() -> PathBuf {
    project_root().join("output")
}

/// `{project_root}/cache/` — content-addressed panel cache.
pub fn cache_dir() -> PathBuf {
    project_root().join("cache")
}

/// `{project_root}/strips/` — per-strip proposals.
pub fn strips_dir() -> PathBuf {
    project_root().join("strips")
}

/// `{project_root}/openspec/specs/` — authoritative OpenSpec tree.
pub fn specs_dir() -> PathBuf {
    project_root().join("openspec").join("specs")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- LessonId ---------------------------------------------------------

    #[test]
    fn lesson_id_parses_canonical_form() {
        let id = LessonId::new("S1-100-volatile-is-dangerous").unwrap();
        assert_eq!(id.as_str(), "S1-100-volatile-is-dangerous");
        assert_eq!(id.short(), "S1-100");
        assert_eq!(id.slug(), "volatile-is-dangerous");
        assert_eq!(id.season(), SeasonId { n: 1 });
        assert_eq!(id.number(), 100);
        // Display round-trip.
        assert_eq!(id.to_string(), "S1-100-volatile-is-dangerous");
    }

    #[test]
    fn lesson_id_parses_four_digit_number() {
        let id = LessonId::new("S1-1500-proof-by-self-reference").unwrap();
        assert_eq!(id.short(), "S1-1500");
        assert_eq!(id.slug(), "proof-by-self-reference");
        assert_eq!(id.number(), 1500);
    }

    #[test]
    fn lesson_id_display_roundtrips_through_new() {
        let original = "S2-300-podman-run-drop-privileges";
        let id = LessonId::new(original).unwrap();
        let reparsed = LessonId::new(&id.to_string()).unwrap();
        assert_eq!(id, reparsed);
    }

    #[test]
    fn lesson_id_rejects_tombstoned_legacy_slug() {
        let err = LessonId::new("lesson_volatile_is_dangerous").unwrap_err();
        assert!(matches!(err, TtError::InvalidLessonId(..)));
        assert_eq!(err.class(), FailureClass::Canon);
    }

    #[test]
    fn lesson_id_rejects_various_malformed_inputs() {
        for bad in [
            "",
            "volatile-is-dangerous",      // no season prefix
            "1-100-volatile-is-dangerous", // missing S
            "S0-100-volatile-is-dangerous", // season zero
            "S1-abc-volatile-is-dangerous", // non-numeric number
            "S1-100-",                     // empty slug
            "S1-100--double-hyphen",       // leading on slug
            "S1-100-Upper-Case",            // uppercase in slug
            "S1-100-slug_with_underscore", // underscore in slug
            "S1-10-short",                  // 2-digit number
            "S1-12345-toolong",             // 5-digit number
            "S1-000-zero",                  // zero number
        ] {
            assert!(
                LessonId::new(bad).is_err(),
                "expected error for input {bad:?}"
            );
        }
    }

    // -- SpecName ---------------------------------------------------------

    #[test]
    fn spec_name_accepts_plain_ascii() {
        let name = SpecName::new("orchestrator").unwrap();
        assert_eq!(name.as_str(), "orchestrator");
        assert_eq!(name.to_string(), "orchestrator");
    }

    #[test]
    fn spec_name_accepts_utf8_with_macron() {
        // Tlatoāni-spelling is a real spec name in this project.
        let name = SpecName::new("tlatoāni-spelling").unwrap();
        assert_eq!(name.as_str(), "tlatoāni-spelling");
    }

    #[test]
    fn spec_name_accepts_hyphenated_kebab() {
        SpecName::new("visual-qa-loop").unwrap();
        SpecName::new("character-loras").unwrap();
    }

    #[test]
    fn spec_name_rejects_path_like() {
        for bad in [
            "foo.md",
            "foo/bar",
            "foo\\bar",
            "",
            " foo",
            "Orchestrator",
            "foo_bar",
            "-leading",
            "trailing-",
            "foo--bar",
            ".hidden",
        ] {
            assert!(SpecName::new(bad).is_err(), "expected error for {bad:?}");
        }
    }

    // -- PanelHash --------------------------------------------------------

    #[test]
    fn panel_hash_hex_roundtrip() {
        let bytes = [
            0xab, 0xcd, 0xef, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
            0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19,
            0x1a, 0x1b, 0x1c, 0x1d,
        ];
        let h = PanelHash::from_bytes(bytes);
        let hex = h.to_hex();
        assert_eq!(hex.len(), 64);
        let back = PanelHash::from_hex(&hex).unwrap();
        assert_eq!(h, back);
        assert_eq!(format!("{h}"), hex);
    }

    #[test]
    fn panel_hash_rejects_wrong_length() {
        assert!(PanelHash::from_hex("deadbeef").is_err());
        assert!(PanelHash::from_hex("").is_err());
        assert!(PanelHash::from_hex("zz".repeat(32).as_str()).is_err());
    }

    #[test]
    fn panel_hash_serde_is_hex_string() {
        let h = PanelHash::from_bytes([0u8; 32]);
        let json = serde_json::to_string(&h).unwrap();
        assert_eq!(json, format!("\"{}\"", "0".repeat(64)));
        let back: PanelHash = serde_json::from_str(&json).unwrap();
        assert_eq!(h, back);
    }

    // -- StripId ----------------------------------------------------------

    #[test]
    fn strip_id_rejects_zero() {
        assert!(StripId::new(0).is_err());
    }

    #[test]
    fn strip_id_zero_pads() {
        assert_eq!(StripId::new(1).unwrap().to_string(), "01");
        assert_eq!(StripId::new(15).unwrap().to_string(), "15");
        assert_eq!(StripId::new(99).unwrap().to_string(), "99");
        // Three-digit renders as-is (no truncation).
        assert_eq!(StripId::new(150).unwrap().to_string(), "150");
    }

    // -- SeasonId ---------------------------------------------------------

    #[test]
    fn season_id_formats_and_rejects_zero() {
        assert_eq!(SeasonId::new(1).unwrap().to_string(), "S1");
        assert_eq!(SeasonId::new(2).unwrap().to_string(), "S2");
        assert!(SeasonId::new(0).is_err());
    }

    // -- Role / Zone ------------------------------------------------------

    #[test]
    fn role_container_names_are_ascii() {
        for r in [Role::Inference, Role::Trainer, Role::Viewer] {
            let name = r.container_name();
            assert!(
                name.is_ascii(),
                "container name {name} must be ASCII (TB03)"
            );
            assert!(name.starts_with("tlatoani-tales-"));
        }
    }

    // -- FailureClass -----------------------------------------------------

    #[test]
    fn failure_class_exit_codes_match_spec() {
        assert_eq!(FailureClass::Canon.exit_code(), 10);
        assert_eq!(FailureClass::CanonNeedsHuman.exit_code(), 20);
        assert_eq!(FailureClass::Infra.exit_code(), 30);
        assert_eq!(FailureClass::InfraSubcode.exit_code(), 31);
        assert_eq!(FailureClass::Usage.exit_code(), 2);
    }

    #[test]
    fn tt_error_class_mapping() {
        let e = TtError::Infra("unreachable".into());
        assert_eq!(e.class(), FailureClass::Infra);
        let e = TtError::Canon("drift".into());
        assert_eq!(e.class(), FailureClass::Canon);
        let e = TtError::CanonNeedsHuman("rerolls exhausted".into());
        assert_eq!(e.class(), FailureClass::CanonNeedsHuman);
        let e = TtError::InfraPermissionDenied("bind mount denied".into());
        assert_eq!(e.class(), FailureClass::InfraSubcode);
        let e = TtError::Usage("bad arg".into());
        assert_eq!(e.class(), FailureClass::Usage);
    }

    // -- podman::lint_flags ----------------------------------------------

    #[test]
    fn podman_lint_catches_missing_flag() {
        // Missing --network=none.
        let flags = [
            "--rm",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--userns=keep-id",
            "--read-only",
        ];
        let err = podman::lint_flags(&flags).unwrap_err();
        assert!(matches!(err, TtError::PodmanFlagLint(_)));
        assert_eq!(err.class(), FailureClass::Canon);
    }

    #[test]
    fn podman_lint_accepts_all_canonical_flags() {
        podman::lint_flags(podman::DEFAULT_FLAGS).unwrap();
    }

    #[test]
    fn podman_lint_accepts_flags_with_extras() {
        // Callers routinely add --name, --volume, --device, etc.
        let mut v: Vec<&str> = podman::DEFAULT_FLAGS.to_vec();
        v.push("--name=tlatoani-tales-inference");
        v.push("--volume=/host:/cont:ro");
        podman::lint_flags(&v).unwrap();
    }

    #[test]
    fn podman_container_name_matches_role() {
        assert_eq!(
            podman::container_name(Role::Inference),
            "tlatoani-tales-inference"
        );
        assert_eq!(
            podman::container_name(Role::Trainer),
            "tlatoani-tales-trainer"
        );
        assert_eq!(
            podman::container_name(Role::Viewer),
            "tlatoani-tales-viewer"
        );
    }

    // -- path helpers -----------------------------------------------------

    #[test]
    fn project_root_looks_reasonable() {
        let root = project_root();
        // Either the env var override or the derived path must exist on disk.
        // We only assert structure, not existence (CI might run elsewhere).
        assert!(
            output_dir().starts_with(&root)
                && cache_dir().starts_with(&root)
                && strips_dir().starts_with(&root)
                && specs_dir().starts_with(&root)
        );
        assert!(specs_dir().ends_with("openspec/specs"));
    }
}
