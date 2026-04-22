//! Canonical `podman run` hardening flags — the single source of truth.
//!
//! Every untrusted container launch in the trusted Rust zone goes through
//! this constant. Drift between this list and
//! `openspec/specs/isolation/spec.md` §Canonical `podman run` flags is
//! itself a canon lint failure (`isolation.flag-source-of-truth`).
//!
// @trace spec:isolation
// @Lesson S1-1500

/// The non-negotiable flag list applied to every untrusted `podman run`
/// invocation. Call sites compose role-specific `--name`, `--volume`, image
/// tag, and `--device` arguments on top of this.
///
/// Network mode defaults to `none` (fully offline inference). Override only
/// at the explicit image-build or weight-pull callsite, annotated there with
/// the rationale per `isolation/spec.md` §Trust test procedure.
pub const DEFAULT_FLAGS: &[&str] = &[
    "--rm",
    "--cap-drop=ALL",
    "--security-opt=no-new-privileges",
    "--userns=keep-id",
    "--read-only",
    "--network=none",
];

/// Container name prefix — ASCII-only per teachable break TB03 in
/// `openspec/specs/tlatoāni-spelling/spec.md`. The gap between the comic's
/// `Tlatoāni` and this container's `tlatoani` is evidence, not oversight.
pub const CONTAINER_NAME_PREFIX: &str = "tlatoani-tales-";
