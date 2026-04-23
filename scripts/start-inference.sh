#!/usr/bin/env bash
# Tlatoāni Tales — canonical local inference launcher.
#
# Starts the long-running `tlatoani-tales-inference` container that exposes
# ComfyUI (port 8188) and ollama (port 11434) to the trusted toolbox over
# localhost. The trusted Rust crates `tt-comfy` and `tt-qa` reach in over
# `reqwest`. The container itself runs hardened per
# `openspec/specs/isolation/spec.md` §Canonical podman run flags.
#
# Why not `--network=none`?
#   `--network=none` gives the container no network namespace at all, which
#   means `--publish` cannot forward a port to the host (there is nothing
#   to forward from). Inference is HTTP-served — the trusted client must
#   reach the in-container API or the whole pipeline stops. So inference
#   uses the default Podman bridge network, with `--publish` bound on
#   `127.0.0.1` only so the ports are reachable from the host's loopback
#   and nowhere else.
#
#   The container is still **offline-equivalent** in practice: nothing in
#   the trusted side instructs ComfyUI/ollama to fetch from the internet
#   at run time. Model weights arrive through read-only bind mounts. The
#   `--cap-drop=ALL --read-only --userns=keep-id` flags still hold.
#
#   Trainer is the role that genuinely uses `--network=none` — it is a
#   one-shot subprocess that reads bind-mounted refs and writes a
#   bind-mounted LoRA, no HTTP at all. That asymmetry is real architecture,
#   not an oversight. See `openspec/specs/isolation/spec.md` §Network mode
#   per role.
#
# Idempotent. Single-instance. Owns the container lifecycle.
#
# Exit codes:
#   0  — Inference container running (or cleanly stopped via --stop).
#   1  — Precondition failed (wrong OS, missing podman, image absent, port taken).
#   2  — Container failed to start for a non-precondition reason.
#
# @trace spec:isolation, spec:orchestrator, spec:visual-qa-loop
# @Lesson S1-1300
# @Lesson S1-1500

set -euo pipefail
IFS=$'\n\t'

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

readonly CONTAINER_NAME="tlatoani-tales-inference"   # ASCII per TB03.
readonly IMAGE_TAG="tlatoani-tales-inference:latest"
readonly DEFAULT_COMFY_PORT=8188
readonly DEFAULT_OLLAMA_PORT=11434
readonly NOT_SILVERBLUE_MSG="Local inference supports Fedora Silverblue only. Season 2 teaches why."

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly PROJECT_DIR

readonly MODELS_DIR="${PROJECT_DIR}/tools/ComfyUI/models"
readonly OLLAMA_MODELS_DIR="${PROJECT_DIR}/tools/ollama-models"

# ---------------------------------------------------------------------------
# CLI parsing
# ---------------------------------------------------------------------------

COMFY_PORT="${DEFAULT_COMFY_PORT}"
OLLAMA_PORT="${DEFAULT_OLLAMA_PORT}"
MODE="start"

usage() {
  cat <<'EOF'
Tlatoāni Tales — local inference runtime launcher.

Usage:
  start-inference.sh [--comfy-port N] [--ollama-port N]
  start-inference.sh --stop
  start-inference.sh --rebuild
  start-inference.sh --help

The first form starts the inference container in the untrusted zone with
ComfyUI on 127.0.0.1:8188 and ollama on 127.0.0.1:11434 (defaults).
Idempotent: if already running, logs the URLs and exits 0.

Options:
  --comfy-port N      Override the host-side ComfyUI port (default 8188).
  --ollama-port N     Override the host-side ollama port (default 11434).
  --stop              Stop and remove the container; exit 0.
  --rebuild           Remove the local image, rebuild from
                      images/inference/Containerfile, then start fresh.
  --help              Print this and exit 0.

The container picks up model weights via read-only bind mount of
${MODELS_DIR/$HOME/~}. Ollama models persist in
${OLLAMA_MODELS_DIR/$HOME/~} (gitignored).

See openspec/specs/isolation/spec.md and
docs/cheatsheets/inference-runtime.md for the boundary contract.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --comfy-port)
      COMFY_PORT="${2:?--comfy-port requires a value}"
      shift 2
      ;;
    --ollama-port)
      OLLAMA_PORT="${2:?--ollama-port requires a value}"
      shift 2
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
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

# ---------------------------------------------------------------------------
# OS guard — Silverblue only
# ---------------------------------------------------------------------------

if [[ ! -r /etc/os-release ]] \
   || ! grep -qE '^VARIANT_ID=("?)silverblue\1$' /etc/os-release; then
  echo "${NOT_SILVERBLUE_MSG}" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Podman guard
# ---------------------------------------------------------------------------

if ! command -v podman >/dev/null 2>&1; then
  echo "podman not found. Install with: rpm-ostree install podman" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# --stop
# ---------------------------------------------------------------------------

if [[ "${MODE}" == "stop" ]]; then
  podman stop "${CONTAINER_NAME}" >/dev/null 2>&1 || true
  podman rm   "${CONTAINER_NAME}" >/dev/null 2>&1 || true
  echo "stopped"
  exit 0
fi

# ---------------------------------------------------------------------------
# --rebuild
# ---------------------------------------------------------------------------

if [[ "${MODE}" == "rebuild" ]]; then
  echo "rebuilding ${IMAGE_TAG} from ${PROJECT_DIR}/images/inference/Containerfile…"
  podman stop "${CONTAINER_NAME}" >/dev/null 2>&1 || true
  podman rm   "${CONTAINER_NAME}" >/dev/null 2>&1 || true
  podman rmi -f "${IMAGE_TAG}" >/dev/null 2>&1 || true
  podman build -t "${IMAGE_TAG}" \
    -f "${PROJECT_DIR}/images/inference/Containerfile" \
    "${PROJECT_DIR}/images/inference/"
fi

# ---------------------------------------------------------------------------
# Image guard
# ---------------------------------------------------------------------------

if ! podman image exists "${IMAGE_TAG}"; then
  echo "Inference image ${IMAGE_TAG} not present — building from ${PROJECT_DIR}/images/inference/Containerfile…"
  podman build -t "${IMAGE_TAG}" \
    -f "${PROJECT_DIR}/images/inference/Containerfile" \
    "${PROJECT_DIR}/images/inference/"
fi

# ---------------------------------------------------------------------------
# Bind-mount targets
# ---------------------------------------------------------------------------

if [[ ! -d "${MODELS_DIR}" ]]; then
  echo "Model weights directory missing: ${MODELS_DIR}" >&2
  echo "Provision via scripts/download-models.sh first." >&2
  exit 1
fi

mkdir -p "${OLLAMA_MODELS_DIR}"

# ---------------------------------------------------------------------------
# Already-running guard
# ---------------------------------------------------------------------------

if podman ps --filter "name=^${CONTAINER_NAME}$" --format '{{.Names}}' \
     | grep -qx "${CONTAINER_NAME}"; then
  echo "already running:"
  echo "  ComfyUI  http://localhost:${COMFY_PORT}"
  echo "  ollama   http://localhost:${OLLAMA_PORT}"
  exit 0
fi

# ---------------------------------------------------------------------------
# Stopped-but-exists branch
# ---------------------------------------------------------------------------

if podman container exists "${CONTAINER_NAME}"; then
  podman start "${CONTAINER_NAME}" >/dev/null
  echo "started:"
  echo "  ComfyUI  http://localhost:${COMFY_PORT}"
  echo "  ollama   http://localhost:${OLLAMA_PORT}"
  exit 0
fi

# ---------------------------------------------------------------------------
# Fresh run
# ---------------------------------------------------------------------------

# tt-lint: inference-role — long-running HTTP-served service. Exempt from
# --network=none (would prevent --publish forwarding) and --rm (we keep the
# container around for start/stop lifecycle). Every other DEFAULT_FLAGS
# entry from tt_core::podman remains required.
#
# COMFY_HOST and OLLAMA_HOST are overridden to 0.0.0.0 inside the container
# so the in-container services bind on the bridge interface — without that,
# they bind only to the in-container loopback and --publish has nothing to
# forward to.
#
# @trace spec:isolation, spec:orchestrator
if ! podman run --detach \
  --name "${CONTAINER_NAME}" \
  --cap-drop=ALL \
  --security-opt=no-new-privileges \
  --userns=keep-id \
  --read-only \
  --publish "127.0.0.1:${COMFY_PORT}:8188" \
  --publish "127.0.0.1:${OLLAMA_PORT}:11434" \
  --env COMFY_HOST=0.0.0.0 \
  --env COMFY_PORT=8188 \
  --env OLLAMA_HOST=0.0.0.0:11434 \
  --tmpfs /tmp \
  --tmpfs /opt/ComfyUI/temp \
  --tmpfs /opt/ComfyUI/output \
  --tmpfs /opt/ComfyUI/.hf-cache \
  --volume "${MODELS_DIR}:/opt/ComfyUI/models:ro,Z" \
  --volume "${OLLAMA_MODELS_DIR}:/opt/ollama/models:rw,Z" \
  --device nvidia.com/gpu=all \
  "${IMAGE_TAG}" \
  >/dev/null
then
  echo "podman run failed — see 'podman logs ${CONTAINER_NAME}' if the container exists" >&2
  exit 2
fi

echo "started:"
echo "  ComfyUI  http://localhost:${COMFY_PORT}"
echo "  ollama   http://localhost:${OLLAMA_PORT}"
