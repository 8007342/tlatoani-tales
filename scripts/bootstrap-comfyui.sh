#!/usr/bin/env bash
# Bootstraps ComfyUI inside the tlatoani-tales toolbox.
# Idempotent — safe to re-run.
#
# Usage:
#   toolbox enter tlatoani-tales
#   cd ~/src/tlatoāni-tales
#   scripts/bootstrap-comfyui.sh
#
# @trace spec:script-conventions, spec:orchestrator, spec:character-loras
# @Lesson S1-1500

set -euo pipefail

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

PROJECT_DIR="/var/home/machiyotl/src/tlatoāni-tales"
TOOLS_DIR="${PROJECT_DIR}/tools"
COMFY_DIR="${TOOLS_DIR}/ComfyUI"
VENV_DIR="${COMFY_DIR}/.venv"
LOG_DIR="${PROJECT_DIR}/tools/logs"

mkdir -p "${TOOLS_DIR}" "${LOG_DIR}"

log() { printf '[%s] %s\n' "$(date +%H:%M:%S)" "$*"; }

# Step 1: ensure system deps inside toolbox
# We pin Python 3.12 because PyTorch stable wheels don't yet cover 3.13/3.14
# which is what Fedora 43 ships by default.
log "checking system deps"
need_install=()
command -v git        >/dev/null || need_install+=(git)
command -v python3.12 >/dev/null || need_install+=(python3.12)
command -v gcc        >/dev/null || need_install+=(gcc)

if [[ ${#need_install[@]} -gt 0 ]]; then
  log "installing: ${need_install[*]}"
  sudo dnf install -y "${need_install[@]}"
fi

# Step 2: clone ComfyUI
if [[ ! -d "${COMFY_DIR}/.git" ]]; then
  log "cloning ComfyUI"
  git clone --depth 1 https://github.com/comfyanonymous/ComfyUI.git "${COMFY_DIR}"
else
  log "ComfyUI already cloned at ${COMFY_DIR}"
fi

# Step 3: venv (Python 3.12) + python deps
# If venv exists but uses the wrong Python, rebuild it.
if [[ -d "${VENV_DIR}" ]] && ! "${VENV_DIR}/bin/python" --version 2>&1 | grep -q "3.12"; then
  log "venv uses wrong Python — rebuilding"
  rm -rf "${VENV_DIR}"
fi
if [[ ! -d "${VENV_DIR}" ]]; then
  log "creating venv (Python 3.12)"
  python3.12 -m venv "${VENV_DIR}"
fi

# shellcheck disable=SC1091
source "${VENV_DIR}/bin/activate"

log "upgrading pip"
pip install --quiet --upgrade pip wheel setuptools

log "installing PyTorch (CUDA 12.4 wheels — forward-compat with driver 580 / CUDA 13)"
pip install --quiet torch torchvision --index-url https://download.pytorch.org/whl/cu124

log "installing ComfyUI requirements"
pip install --quiet -r "${COMFY_DIR}/requirements.txt"

# Step 4: GPU sanity check
log "GPU sanity check"
python3 - <<'PY'
import torch
print(f"  torch {torch.__version__}")
print(f"  cuda available: {torch.cuda.is_available()}")
if torch.cuda.is_available():
    print(f"  device: {torch.cuda.get_device_name(0)}")
    print(f"  vram total: {torch.cuda.get_device_properties(0).total_memory / 1e9:.1f} GB")
PY

log "bootstrap complete"
