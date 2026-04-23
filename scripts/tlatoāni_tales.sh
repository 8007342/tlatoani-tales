#!/usr/bin/env bash
# Tlatoāni Tales — canonical local viewer launcher.
#
# Single local entry point for Calmecac. Silverblue-only: refusing on any
# other OS is deliberate — see openspec/specs/calmecac/spec.md §Silverblue-
# only constraint. The refusal message is a Season 2 seed, not an apology.
#
# The filename carries a macron (Tlatoāni); no ASCII fallback is allowed
# (openspec/specs/tlatoāni-spelling/spec.md §TB03 catalogues the sibling
# ASCII-only container name). If a filesystem refuses the UTF-8 filename
# that is itself a catalogued teachable break — we do not silently
# substitute.
#
# Idempotent. Single-instance. Owns the container lifecycle (create,
# start, stop, rebuild). Exit codes per openspec/specs/calmecac/spec.md
# §Exit codes:
#
#   0  — Viewer running (or cleanly stopped via --stop).
#   1  — Precondition failed (wrong OS, missing podman, port taken).
#   2  — Container failed to start for a non-precondition reason.
#
# @trace spec:calmecac, spec:isolation
# @Lesson S1-1000
# @Lesson S1-1500

set -euo pipefail
IFS=$'\n\t'

# zone: host
# See openspec/specs/script-conventions/spec.md
# The viewer launcher runs against the host's rootless podman storage
# (podman build + podman run). Podman is NOT installed inside the
# tlatoani-tales toolbox, and nested rootless podman is fragile —
# this script belongs on the host shell.
if [[ -f /run/.toolboxenv ]]; then
  echo "ERROR: this script must run on the HOST shell (not inside any toolbox)." >&2
  echo "       exit the toolbox (Ctrl-D or 'exit') and retry." >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

readonly CONTAINER_NAME="tlatoani-tales-viewer"   # ASCII per TB03.
readonly IMAGE_TAG="tlatoani-tales-viewer:latest"
readonly DEFAULT_PORT=8088
readonly NOT_SILVERBLUE_MSG="Local viewer supports Fedora Silverblue only. Season 2 teaches why."

# ---------------------------------------------------------------------------
# CLI parsing
# ---------------------------------------------------------------------------

PORT="${DEFAULT_PORT}"
MODE="start"   # start | stop | rebuild | help

usage() {
  cat <<'EOF'
Calmecac — the upstairs library for Tlatoāni Tales.

Usage:
  tlatoāni_tales.sh [--port N] [--rebuild]
  tlatoāni_tales.sh --stop
  tlatoāni_tales.sh --help

Flags:
  --port N     Override the default port (8088).
  --stop       Stop the running viewer and exit.
  --rebuild    Rebuild the viewer image from source, then start.
  --help       Print this message.

Running this script with no flags starts the viewer if it is not already
running, and is a no-op if it is. Only Fedora Silverblue is supported —
that is the teaching, not a limitation.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --port)
      [[ $# -ge 2 ]] || { echo "--port requires a value" >&2; exit 1; }
      PORT="$2"
      shift 2
      ;;
    --port=*)
      PORT="${1#--port=}"
      shift
      ;;
    --stop)
      MODE="stop"
      shift
      ;;
    --rebuild)
      MODE="rebuild"
      shift
      ;;
    --help|-h)
      MODE="help"
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ "${MODE}" == "help" ]]; then
  usage
  exit 0
fi

# ---------------------------------------------------------------------------
# Guards
# ---------------------------------------------------------------------------

# OS guard — Silverblue-only per openspec/specs/calmecac/spec.md.
# @trace spec:calmecac
os_release="/etc/os-release"
variant_id=""
if [[ -r "${os_release}" ]]; then
  # shellcheck disable=SC1090
  variant_id="$(. "${os_release}" >/dev/null 2>&1 && printf '%s' "${VARIANT_ID:-}")"
fi
if [[ "${variant_id}" != "silverblue" ]]; then
  echo "${NOT_SILVERBLUE_MSG}"
  exit 1
fi

# Podman guard.
# @trace spec:isolation
if ! command -v podman >/dev/null 2>&1; then
  echo "podman not found on PATH." >&2
  echo "On Fedora Silverblue: rpm-ostree install podman && systemctl reboot" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(realpath "${SCRIPT_DIR}/..")"
BUNDLE_DIR="${PROJECT_DIR}/output/calmecac-bundle"
CONTAINERFILE_DIR="${PROJECT_DIR}/images/viewer"
CONTAINERFILE="${CONTAINERFILE_DIR}/Containerfile"

# ---------------------------------------------------------------------------
# Subcommand: --stop
# ---------------------------------------------------------------------------

if [[ "${MODE}" == "stop" ]]; then
  # Silent — spec says --stop exits 0 whether or not it was running.
  podman stop "${CONTAINER_NAME}" >/dev/null 2>&1 || true
  podman rm   "${CONTAINER_NAME}" >/dev/null 2>&1 || true
  echo "stopped"
  exit 0
fi

# ---------------------------------------------------------------------------
# Ensure bundle exists (build if missing)
# ---------------------------------------------------------------------------

# Bundle is "ready" only when BOTH the concept index AND the PWA entry
# point are present. The earlier check on calmecac-index.json alone let
# a prior bare indexer run satisfy the gate while leaving httpd with no
# index.html to serve. tt-calmecac::cmd_build copies calmecac/web/* into
# the bundle, so the second predicate guards that step.
if [[ ! -f "${BUNDLE_DIR}/calmecac-index.json" ]] \
   || [[ ! -f "${BUNDLE_DIR}/index.html" ]]; then
  echo "Calmecac bundle missing or incomplete — running tt-calmecac build…"
  if command -v tt-calmecac >/dev/null 2>&1; then
    tt-calmecac build
  elif command -v cargo >/dev/null 2>&1; then
    ( cd "${PROJECT_DIR}" && cargo run --quiet -p tt-calmecac -- build )
  else
    echo "tt-calmecac not installed and cargo unavailable." >&2
    echo "Build the workspace with:  cargo build --workspace" >&2
    exit 2
  fi
  if [[ ! -f "${BUNDLE_DIR}/calmecac-index.json" ]]; then
    echo "tt-calmecac build did not produce ${BUNDLE_DIR}/calmecac-index.json" >&2
    exit 2
  fi
fi

# ---------------------------------------------------------------------------
# Subcommand: --rebuild (flows into start)
# ---------------------------------------------------------------------------

if [[ "${MODE}" == "rebuild" ]]; then
  podman stop "${CONTAINER_NAME}" >/dev/null 2>&1 || true
  podman rm   "${CONTAINER_NAME}" >/dev/null 2>&1 || true
  podman rmi -f "${IMAGE_TAG}"    >/dev/null 2>&1 || true
  if ! podman build -t "${IMAGE_TAG}" -f "${CONTAINERFILE}" "${CONTAINERFILE_DIR}"; then
    echo "podman build failed for ${IMAGE_TAG}" >&2
    exit 2
  fi
fi

# ---------------------------------------------------------------------------
# Image presence — build on demand if missing
# ---------------------------------------------------------------------------

if ! podman image exists "${IMAGE_TAG}"; then
  echo "Viewer image ${IMAGE_TAG} not present — building from ${CONTAINERFILE}…"
  if ! podman build -t "${IMAGE_TAG}" -f "${CONTAINERFILE}" "${CONTAINERFILE_DIR}"; then
    echo "podman build failed for ${IMAGE_TAG}" >&2
    exit 2
  fi
fi

# ---------------------------------------------------------------------------
# Container state — idempotent start
# ---------------------------------------------------------------------------

# Already running? — echo + exit 0.
running_status="$(podman ps --filter "name=^${CONTAINER_NAME}$" --format '{{.Status}}' || true)"
if [[ -n "${running_status}" ]]; then
  echo "already running at http://localhost:${PORT}"
  exit 0
fi

# Stopped-but-exists? — start it.
if podman container exists "${CONTAINER_NAME}"; then
  if ! podman start "${CONTAINER_NAME}" >/dev/null; then
    echo "podman start ${CONTAINER_NAME} failed" >&2
    exit 2
  fi
else
  # Fresh run. Flags MUST match openspec/specs/isolation/spec.md §Canonical
  # podman run flags. --tmpfs compensates for --read-only + httpd's need
  # for a writable /tmp, /var/cache, /var/log — apache writes pid files
  # and ephemeral scoreboard bits that do not belong on the real disk.
  # This is scoped writable state, not mutable code — the tmpfs vanishes
  # on container exit, honouring the ephemeral-container rule.
  #
  # --network is intentionally left at podman's default (bridge) here,
  # NOT --network=none. The publish flag binds to 127.0.0.1 only, so the
  # container is addressable from the host browser but not from outside.
  # Viewer is the one untrusted role that MUST accept inbound HTTP on
  # localhost — see openspec/specs/isolation/spec.md §The boundary
  # (viewer row: "Bridge, bound to 127.0.0.1 only").
  #
  # tt-lint: viewer-role — exempt from --network=none (serves HTTP) and
  # --rm (start/stop lifecycle). All other DEFAULT_FLAGS still required.
  # @trace spec:isolation, spec:calmecac
  if ! podman run --detach \
       --name "${CONTAINER_NAME}" \
       --cap-drop=ALL \
       --security-opt=no-new-privileges \
       --userns=keep-id \
       --read-only \
       --tmpfs /tmp \
       --tmpfs /var/cache \
       --tmpfs /var/log \
       --tmpfs /run \
       --publish "127.0.0.1:${PORT}:8080" \
       --volume "${BUNDLE_DIR}:/usr/local/apache2/htdocs/:ro,Z" \
       "${IMAGE_TAG}" >/dev/null; then
    echo "podman run failed — see 'podman logs ${CONTAINER_NAME}' if the container exists" >&2
    exit 2
  fi
fi

# ---------------------------------------------------------------------------
# Best-effort browser open + URL echo
# ---------------------------------------------------------------------------

if [[ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]] && command -v xdg-open >/dev/null 2>&1; then
  xdg-open "http://localhost:${PORT}" >/dev/null 2>&1 || true
fi

echo "Calmecac serving at http://localhost:${PORT}"
exit 0
