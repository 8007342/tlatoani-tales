#!/usr/bin/env bash
# Tlatoāni Tales — local inference runtime launcher (Season 1 MVP).
#
# Runs ComfyUI inside the existing `tlatoani-tales` toolbox. The toolbox
# already provides the untrusted-zone runtime at the level we need for
# Season 1: rootless podman container, automatic nvidia-smi passthrough,
# filesystem shared with the host's $HOME so model weights are reachable
# without bespoke bind mounts. ComfyUI is installed at `tools/ComfyUI/`
# by `scripts/bootstrap-comfyui.sh`; FLUX-schnell and Qwen-Image weights
# live under `tools/ComfyUI/models/`.
#
# Why toolbox instead of the hardened podman container
# ---------------------------------------------------------------------------
# The hardened-container path (images/inference/Containerfile, full
# DEFAULT_FLAGS, CDI GPU passthrough) was authored as Season 2 teaching
# material — the literal demonstration of `podman-run-drop-privileges`.
# It requires layering `nvidia-container-toolkit` and a `nvidia-ctk cdi
# generate` step, which is Silverblue-reboot-level state change. Season 1
# MVP ships without that prerequisite: `nvidia-smi` is already on the
# host, the toolbox already sees the GPU, and the author's feedback was
# explicit — *"we don't always know better, use what already works"*.
#
# The trust boundary is still preserved conceptually: the trusted Rust
# workspace runs in this host shell (or in the same toolbox's cargo
# workflow) and calls into the ComfyUI HTTP API. The untrusted Python
# runtime lives inside the toolbox and doesn't leak beyond it. The
# migration to full hardening is a Season 2 lesson, not an MVP blocker.
# See openspec/specs/isolation/spec.md §Phased isolation.
#
# Idempotent, PID-file-tracked, single-instance. Exit codes:
#   0 — ComfyUI running (or cleanly stopped via --stop, or --status query)
#   1 — Precondition failed (wrong OS, missing toolbox, missing install)
#   2 — ComfyUI failed to start
#
# @trace spec:isolation, spec:orchestrator, spec:visual-qa-loop
# @Lesson S1-1300
# @Lesson S1-1500

set -euo pipefail
IFS=$'\n\t'

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

readonly TOOLBOX_NAME="tlatoani-tales"         # ASCII per TB03.
readonly DEFAULT_COMFY_HOST="127.0.0.1"
readonly DEFAULT_COMFY_PORT=8188
readonly NOT_SILVERBLUE_MSG="Local inference supports Fedora Silverblue only. Season 2 teaches why."

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly PROJECT_DIR

readonly COMFY_DIR="${PROJECT_DIR}/tools/ComfyUI"
readonly STATE_DIR="${PROJECT_DIR}/tools/inference"
readonly LOG_DIR="${PROJECT_DIR}/tools/logs"
readonly PID_FILE="${STATE_DIR}/comfyui.pid"
readonly LOG_FILE="${LOG_DIR}/comfyui.log"

# ---------------------------------------------------------------------------
# CLI parsing
# ---------------------------------------------------------------------------

COMFY_HOST="${DEFAULT_COMFY_HOST}"
COMFY_PORT="${DEFAULT_COMFY_PORT}"
MODE="start"

usage() {
  cat <<'EOF'
Tlatoāni Tales — local inference runtime launcher (Season 1 MVP, toolbox-based).

Runs ComfyUI inside the `tlatoani-tales` toolbox, which already provides
GPU passthrough automatically. No podman container, no CDI, no
hardened-flag ceremony — see openspec/specs/isolation/spec.md
§Phased isolation for why.

Usage:
  start-inference.sh [--host H] [--port N]
  start-inference.sh --stop
  start-inference.sh --status
  start-inference.sh --help

Options:
  --host H       Bind ComfyUI to host H inside the toolbox (default 127.0.0.1).
  --port N       Bind to port N (default 8188). The port is already
                 reachable from the host because toolbox shares the
                 network namespace with the host in rootless podman.
  --stop         Stop the running ComfyUI, if any.
  --status       Report whether ComfyUI is running.
  --help         Print this and exit 0.

State:
  PID file     tools/inference/comfyui.pid (gitignored)
  Log file     tools/logs/comfyui.log (gitignored)

Prerequisites (both provided by scripts/bootstrap-comfyui.sh):
  - Toolbox `tlatoani-tales` exists
  - ComfyUI installed at tools/ComfyUI/ with a working .venv
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --host)
      COMFY_HOST="${2:?--host requires a value}"
      shift 2
      ;;
    --port)
      COMFY_PORT="${2:?--port requires a value}"
      shift 2
      ;;
    --stop)
      MODE="stop"
      shift
      ;;
    --status)
      MODE="status"
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
# Precondition guards
# ---------------------------------------------------------------------------

if [[ ! -r /etc/os-release ]] \
   || ! grep -qE '^VARIANT_ID=("?)silverblue\1$' /etc/os-release; then
  echo "${NOT_SILVERBLUE_MSG}" >&2
  exit 1
fi

if ! command -v toolbox >/dev/null 2>&1; then
  echo "toolbox not found. Silverblue ships it by default; check 'which toolbox'." >&2
  exit 1
fi

# `toolbox list --containers` prints one row per container. The name is
# the second whitespace-separated field. We match exactly.
if ! toolbox list --containers 2>/dev/null \
      | awk 'NR>1 {print $2}' \
      | grep -qx "${TOOLBOX_NAME}"; then
  cat >&2 <<EOF
Toolbox '${TOOLBOX_NAME}' not found. Create with:
  toolbox create ${TOOLBOX_NAME}
Then bootstrap ComfyUI:
  scripts/bootstrap-comfyui.sh
EOF
  exit 1
fi

if [[ ! -x "${COMFY_DIR}/.venv/bin/python" ]]; then
  cat >&2 <<EOF
ComfyUI venv missing at:
  ${COMFY_DIR}/.venv
Bootstrap with:
  scripts/bootstrap-comfyui.sh
EOF
  exit 1
fi

mkdir -p "${STATE_DIR}" "${LOG_DIR}"

# ---------------------------------------------------------------------------
# PID helpers
# ---------------------------------------------------------------------------

read_pid() {
  if [[ -f "${PID_FILE}" ]]; then
    cat "${PID_FILE}" 2>/dev/null
  fi
}

pid_alive() {
  local pid="$1"
  [[ -n "${pid}" ]] && kill -0 "${pid}" 2>/dev/null
}

# ---------------------------------------------------------------------------
# --status
# ---------------------------------------------------------------------------

if [[ "${MODE}" == "status" ]]; then
  pid="$(read_pid)"
  if pid_alive "${pid}"; then
    echo "running:"
    echo "  ComfyUI  http://${COMFY_HOST}:${COMFY_PORT}"
    echo "  pid      ${pid}"
    echo "  log      ${LOG_FILE}"
    exit 0
  fi
  echo "not running"
  exit 0
fi

# ---------------------------------------------------------------------------
# --stop
# ---------------------------------------------------------------------------

if [[ "${MODE}" == "stop" ]]; then
  pid="$(read_pid)"
  if pid_alive "${pid}"; then
    echo "stopping pid ${pid}…"
    # Kill the whole process group so the toolbox-run wrapper + child
    # python together go down cleanly.
    kill -TERM -- "-${pid}" 2>/dev/null || kill -TERM "${pid}" 2>/dev/null || true
    for _ in 1 2 3 4 5; do
      if ! pid_alive "${pid}"; then break; fi
      sleep 1
    done
    if pid_alive "${pid}"; then
      echo "force-killing pid ${pid}"
      kill -KILL -- "-${pid}" 2>/dev/null || kill -KILL "${pid}" 2>/dev/null || true
    fi
  fi
  rm -f "${PID_FILE}"
  echo "stopped"
  exit 0
fi

# ---------------------------------------------------------------------------
# --start (default) — idempotent
# ---------------------------------------------------------------------------

existing="$(read_pid)"
if pid_alive "${existing}"; then
  echo "already running:"
  echo "  ComfyUI  http://${COMFY_HOST}:${COMFY_PORT}"
  echo "  pid      ${existing}"
  echo "  log      ${LOG_FILE}"
  exit 0
fi

if [[ -n "${existing}" ]]; then
  echo "stale pid file (pid ${existing} not alive) — cleaning."
  rm -f "${PID_FILE}"
fi

# Port-conflict pre-flight: if something else is already bound, bail
# with a clear message rather than let ComfyUI die ambiguously.
if command -v ss >/dev/null 2>&1 \
   && ss -lntH "( sport = :${COMFY_PORT} )" 2>/dev/null | grep -q ":${COMFY_PORT}"; then
  echo "port ${COMFY_PORT} is already in use on the host. Override with --port N or free it." >&2
  exit 1
fi

# Launch via `setsid nohup` so we can kill the entire process group
# cleanly on --stop. The toolbox-run wrapper stays alive while Python
# runs; SIGTERM on the group cascades into the python child.
setsid nohup toolbox run -c "${TOOLBOX_NAME}" bash -c \
  "cd '${COMFY_DIR}' && exec '${COMFY_DIR}/.venv/bin/python' main.py --listen '${COMFY_HOST}' --port '${COMFY_PORT}'" \
  > "${LOG_FILE}" 2>&1 &

pid="$!"
echo "${pid}" > "${PID_FILE}"

# Give ComfyUI a moment to start and die-early if the environment is
# broken (missing weights, python import error, etc).
sleep 2
if ! pid_alive "${pid}"; then
  echo "ComfyUI failed to start — see ${LOG_FILE} (tail below):" >&2
  tail -n 20 "${LOG_FILE}" >&2 || true
  rm -f "${PID_FILE}"
  exit 2
fi

echo "started:"
echo "  ComfyUI  http://${COMFY_HOST}:${COMFY_PORT}"
echo "  pid      ${pid}"
echo "  log      ${LOG_FILE}"
