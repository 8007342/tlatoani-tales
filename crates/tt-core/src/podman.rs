//! Canonical `podman run` hardening flags — the single source of truth.
//!
//! Every untrusted container launch in the trusted Rust zone goes through
//! this module. Drift between [`DEFAULT_FLAGS`] and
//! `openspec/specs/isolation/spec.md` §Canonical `podman run` flags is itself
//! a canon lint failure (`isolation.flag-source-of-truth`) — `tt-lint`
//! enforces the pairing from both sides.
//!
// @trace spec:isolation
// @Lesson S1-1500

use crate::{Role, Result, TtError};

/// Container name prefix — ASCII-only per teachable break **TB03** in
/// `openspec/specs/tlatoāni-spelling/spec.md`. The gap between the comic's
/// `Tlatoāni` and this container's `tlatoani` is evidence, not oversight.
pub const CONTAINER_NAME_PREFIX: &str = "tlatoani-tales-";

/// The non-negotiable flag list applied to every untrusted `podman run`
/// invocation.
///
/// Call sites compose role-specific `--name`, `--volume`, image tag, and
/// `--device` arguments on top. Network mode defaults to `none` (fully
/// offline inference). It is only overridden at the explicit image-build
/// or weight-pull callsite, and only when annotated per `isolation/spec.md`
/// §Trust test procedure.
pub const DEFAULT_FLAGS: &[&str] = &[
    "--rm",
    "--cap-drop=ALL",
    "--security-opt=no-new-privileges",
    "--userns=keep-id",
    "--read-only",
    "--network=none",
];

/// The canonical container name for a [`Role`]. Thin wrapper around
/// [`Role::container_name`] — present here so callers in the `podman`
/// submodule read uniformly.
pub fn container_name(role: Role) -> String {
    role.container_name().to_string()
}

/// Verify that a flag list contains every entry in [`DEFAULT_FLAGS`].
///
/// Used by `tt-lint` to enforce the isolation boundary: any trusted-zone
/// construction of a `podman run` argv must preserve the full hardening
/// list. Extra flags (role-specific `--name`, `--volume`, `--device`) are
/// permitted; omissions are canon failures (exit 10 via [`TtError::class`]).
pub fn lint_flags(flags: &[&str]) -> Result<()> {
    for required in DEFAULT_FLAGS {
        if !flags.iter().any(|f| f == required) {
            return Err(TtError::PodmanFlagLint(format!(
                "missing canonical hardening flag `{required}` — see \
                 openspec/specs/isolation/spec.md §Canonical `podman run` flags"
            )));
        }
    }
    Ok(())
}
