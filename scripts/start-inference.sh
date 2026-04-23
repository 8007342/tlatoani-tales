#!/usr/bin/env bash
# Tlatoāni Tales — local inference runtime launcher (Season 1 MVP).
#
# Runs ComfyUI from inside the `tlatoani-tales` toolbox. The toolbox
# provides the untrusted-zone runtime for Season 1: rootless podman
# container, automatic nvidia-smi passthrough, filesystem shared with
# the host's $HOME. ComfyUI is installed at `tools/ComfyUI/` by
# `scripts/bootstrap-comfyui.sh`; FLUX-schnell and Qwen-Image weights
# live under `tools/ComfyUI/models/`.
#
# The hardened-container path (images/inference/Containerfile, full
# DEFAULT_FLAGS, CDI GPU passthrough) remains authored as Season 2
# teach-by-example material — see openspec/specs/isolation/spec.md
# §Phased isolation.
#
# Usage:
#   toolbox enter tlatoani-tales
#   cd ~/src/tlatoāni-tales
#   scripts/start-inference.sh                 # start
#   scripts/start-inference.sh --status
#   scripts/start-inference.sh --stop
#
# Idempotent, PID-file tracked. Exit codes:
#   0 — ComfyUI running (or cleanly stopped via --stop, or --status query)
#   1 — Precondition failed (wrong zone, missing install, port busy)
#   2 — ComfyUI failed to start
#
# @trace spec:script-conventions, spec:isolation, spec:orchestrator, spec:visual-qa-loop
# @Lesson S1-1300
# @Lesson S1-1500

set -euo pipefail
IFS=$'\n\t'

# zone: inside-toolbox
# See openspec/specs/script-conventions/spec.md
if [[ ! -f /run/.toolboxenv ]]; then
  echo "ERROR: this script must run INSIDE the tlatoani-tales toolbox." >&2
  echo "       toolbox enter tlatoani-tales" >&2
  echo "       cd ~/src/tlatoāni-tales" >&2
  echo "       $(basename "$0") ..." >&2
  exit 1
fi
_toolbox="$(awk -F= '/^name=/{gsub(/"/,"",$2); print $2}' /run/.containerenv 2>/dev/null || true)"
if [[ "${_toolbox}" != "tlatoani-tales" ]]; then
  echo "ERROR: inside toolbox '${_toolbox}', need 'tlatoani-tales'." >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

readonly DEFAULT_COMFY_HOST="127.0.0.1"
readonly DEFAULT_COMFY_PORT=8188

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
Tlatoāni Tales — local inference runtime launcher (Season 1 MVP, toolbox-resident).

Usage:
  start-inference.sh [--host H] [--port N]
  start-inference.sh --stop
  start-inference.sh --status
  start-inference.sh --help

Options:
  --host H       Bind ComfyUI to host H (default 127.0.0.1). Because
                 the toolbox shares the network namespace with the
                 host, the host shell reaches it on localhost
                 without any --publish ceremony.
  --port N       Bind to port N (default 8188).
  --stop         Stop the running ComfyUI, if any.
  --status       Report whether ComfyUI is running.
  --help         Print this and exit 0.

State:
  PID file     tools/inference/comfyui.pid (gitignored)
  Log file     tools/logs/comfyui.log (gitignored)

Prerequisites (provided by scripts/bootstrap-comfyui.sh):
  - tools/ComfyUI/.venv/ with torch + ComfyUI requirements installed
  - Model weights under tools/ComfyUI/models/
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
# ComfyUI install guard
# ---------------------------------------------------------------------------

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
    # Kill the whole process group first — setsid-launched start means the
    # leader IS the process group id, so `kill -- -${pid}` hits children too.
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

# Port-conflict pre-flight: if something else is already bound, bail with
# a clear message rather than let ComfyUI die ambiguously on startup.
if command -v ss >/dev/null 2>&1 \
   && ss -lntH "( sport = :${COMFY_PORT} )" 2>/dev/null | grep -q ":${COMFY_PORT}"; then
  echo "port ${COMFY_PORT} is already in use. Override with --port N or free it first." >&2
  exit 1
fi

# Launch ComfyUI directly from the venv — no `toolbox run` wrapper, we're
# already inside. setsid makes the new process a session+group leader so
# --stop can signal the whole tree with one kill(-pid, -TERM).
setsid nohup "${COMFY_DIR}/.venv/bin/python" \
  "${COMFY_DIR}/main.py" \
  --listen "${COMFY_HOST}" \
  --port   "${COMFY_PORT}" \
  > "${LOG_FILE}" 2>&1 &

pid="$!"
echo "${pid}" > "${PID_FILE}"

# Sanity: give ComfyUI a moment and verify we didn't immediately die.
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
