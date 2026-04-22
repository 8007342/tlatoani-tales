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

log "installing huggingface-hub CLI"
pip install --quiet --upgrade "huggingface-hub[cli]"

mkdir -p \
  "${MODELS_DIR}/unet" \
  "${MODELS_DIR}/vae" \
  "${MODELS_DIR}/clip" \
  "${MODELS_DIR}/checkpoints" \
  "${MODELS_DIR}/loras"

# -----------------------------------------------------------------------------
# FLUX.1-schnell — base model (Apache 2.0)
# Split into: transformer (unet), VAE, and two text encoders (clip_l + t5xxl).
# Using fp8 T5 variant to stay well inside 24GB VRAM.
# -----------------------------------------------------------------------------
log "downloading FLUX.1-schnell transformer (fp8, ~12GB)"
huggingface-cli download Comfy-Org/flux1-schnell \
  flux1-schnell-fp8.safetensors \
  --local-dir "${MODELS_DIR}/checkpoints"

log "downloading FLUX VAE (~335MB)"
huggingface-cli download black-forest-labs/FLUX.1-schnell \
  ae.safetensors \
  --local-dir "${MODELS_DIR}/vae"

log "downloading CLIP-L (~250MB)"
huggingface-cli download comfyanonymous/flux_text_encoders \
  clip_l.safetensors \
  --local-dir "${MODELS_DIR}/clip"

log "downloading T5-XXL fp8 (~4.9GB)"
huggingface-cli download comfyanonymous/flux_text_encoders \
  t5xxl_fp8_e4m3fn.safetensors \
  --local-dir "${MODELS_DIR}/clip"

# -----------------------------------------------------------------------------
# Qwen-Image — text-rendering specialist for speech bubbles + episode plate.
# Used as a second pass for panels with on-panel text.
# -----------------------------------------------------------------------------
log "downloading Qwen-Image (Apache 2.0, ~20GB)"
huggingface-cli download Qwen/Qwen-Image \
  --local-dir "${MODELS_DIR}/checkpoints/Qwen-Image" \
  --include "*.safetensors" "*.json" "*.txt"

log "download complete"
du -sh "${MODELS_DIR}"/* 2>/dev/null | sort -h
