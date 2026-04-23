#!/usr/bin/env bash
# Downloads image-gen model weights into the ComfyUI models tree.
# Idempotent — `hf download` skips files already present.
#
# Usage:
#   toolbox enter tlatoani-tales
#   cd ~/src/tlatoāni-tales
#   scripts/download-models.sh
#
# @trace spec:script-conventions, spec:orchestrator
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
COMFY_DIR="${PROJECT_DIR}/tools/ComfyUI"
MODELS_DIR="${COMFY_DIR}/models"
VENV="${COMFY_DIR}/.venv"

log() { printf '[%s] %s\n' "$(date +%H:%M:%S)" "$*"; }

# shellcheck disable=SC1091
source "${VENV}/bin/activate"

log "ensuring huggingface_hub is current (ships the 'hf' CLI)"
pip install --quiet --upgrade huggingface_hub

mkdir -p \
  "${MODELS_DIR}/unet" \
  "${MODELS_DIR}/vae" \
  "${MODELS_DIR}/clip" \
  "${MODELS_DIR}/checkpoints" \
  "${MODELS_DIR}/loras"

# -----------------------------------------------------------------------------
# FLUX.1-schnell — base model (Apache 2.0)
# The Comfy-Org fp8 variant is the ALL-IN-ONE checkpoint: transformer, VAE,
# and text encoders baked together (~17GB). Fits comfortably in 24GB VRAM
# and avoids the license-gated black-forest-labs/FLUX.1-schnell repo.
# -----------------------------------------------------------------------------
log "downloading FLUX.1-schnell all-in-one fp8 (~17GB)"
hf download Comfy-Org/flux1-schnell \
  flux1-schnell-fp8.safetensors \
  --local-dir "${MODELS_DIR}/checkpoints"

# -----------------------------------------------------------------------------
# Qwen-Image — text-rendering specialist for speech bubbles + episode plate.
# Used as a second pass for panels with on-panel text.
# -----------------------------------------------------------------------------
log "downloading Qwen-Image (Apache 2.0, full repo ~45GB observed)"
# Earlier revision passed --include "*.safetensors" "*.json" "*.txt" as one
# flag; the hf CLI takes the first token as the pattern and treats the rest
# as explicit filenames, silently skipping the weights. Pulling the full
# repo is simpler and safer.
hf download Qwen/Qwen-Image \
  --local-dir "${MODELS_DIR}/checkpoints/Qwen-Image"

log "download complete"
du -sh "${MODELS_DIR}"/* 2>/dev/null | sort -h
