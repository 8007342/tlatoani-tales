#!/usr/bin/env bash
# Downloads image-gen model weights into the ComfyUI models tree.
# Idempotent — huggingface-cli skips files already present.
#
# @trace spec:image-gen-runtime
set -euo pipefail

PROJECT_DIR="/var/home/machiyotl/src/tlatoani-tales"
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
log "downloading Qwen-Image (Apache 2.0, ~20GB)"
hf download Qwen/Qwen-Image \
  --local-dir "${MODELS_DIR}/checkpoints/Qwen-Image" \
  --include "*.safetensors" "*.json" "*.txt"

log "download complete"
du -sh "${MODELS_DIR}"/* 2>/dev/null | sort -h
