# Inference runtime — cheatsheet

<!-- @trace spec:isolation, spec:orchestrator, spec:visual-qa-loop, spec:character-loras -->
<!-- @Lesson S1-1300, S1-1500 -->

Operational reference for the long-running `tlatoani-tales-inference` container that hosts ComfyUI (FLUX + Qwen-Image) and ollama (VLM). Power-user, scannable, < 10 seconds to find what you need.

## TL;DR

```
scripts/start-inference.sh                      # start (idempotent)
scripts/start-inference.sh --stop               # stop + remove
scripts/start-inference.sh --rebuild            # rebuild image, then start
curl -s http://localhost:8188/system_stats      # ComfyUI healthcheck
curl -s http://localhost:11434/api/tags         # ollama healthcheck
podman logs tlatoani-tales-inference            # tail the container's stdout
```

## What runs where

| Process | Inside container | Reachable from host at |
|---|---|---|
| ComfyUI HTTP API | `0.0.0.0:8188` | `http://127.0.0.1:8188` |
| ollama HTTP API  | `0.0.0.0:11434` | `http://127.0.0.1:11434` |

Both bind `0.0.0.0` inside the container so `--publish 127.0.0.1:PORT:PORT` from podman can forward; the published port is bound to the host's loopback only — the services are not reachable from anything outside the host.

## Hardening flags applied (per `openspec/specs/isolation/spec.md`)

| Flag | Why |
|---|---|
| `--cap-drop=ALL` | No Linux capabilities |
| `--security-opt=no-new-privileges` | setuid neutralised |
| `--userns=keep-id` | bind-mount writes land as the host user, not root |
| `--read-only` | root FS read-only; writable surfaces via `--tmpfs` + scoped binds |
| `--publish 127.0.0.1:PORT:PORT` | localhost-only port exposure (×2 — ComfyUI + ollama) |
| `--device nvidia.com/gpu=all` | CDI GPU passthrough |

`--rm` and `--network=none` are deliberately **not** applied — see `openspec/specs/isolation/spec.md` §Network mode per role. Inference is a long-running HTTP-served service; it uses the start/stop lifecycle and the default Podman bridge with localhost-only published ports. Trainer is the role that uses `--network=none`.

## Bind mounts

| Host path | Container path | Mode | Purpose |
|---|---|---|---|
| `tools/ComfyUI/models/` | `/opt/ComfyUI/models` | `ro,Z` | FLUX + Qwen-Image weights |
| `tools/ollama-models/`  | `/opt/ollama/models`  | `rw,Z` | ollama VLM models, persistent across restarts |

`tmpfs` overlays for ComfyUI's writable scratch (`/opt/ComfyUI/temp`, `/opt/ComfyUI/output`, `/opt/ComfyUI/.hf-cache`) — ephemeral by design.

## First-time provision checklist

1. Build the image once: `scripts/start-inference.sh` will auto-build on first run.
2. Confirm model weights are present:
   ```
   ls tools/ComfyUI/models/checkpoints/   # expect flux1-schnell-fp8.safetensors + Qwen-Image/
   ```
3. Pull a VLM into the persistent ollama dir (one-time, requires network):
   ```
   curl -s http://localhost:11434/api/pull -d '{"name":"moondream:2b"}'
   ```
   Pull lands in `tools/ollama-models/`; subsequent runs reuse it.

## Common troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| `podman run failed` | Image missing or built for a different arch | `--rebuild` |
| `Address already in use` on 8188 | Existing ComfyUI or stale container | `--stop` then start, or pick `--comfy-port` |
| `nvidia.com/gpu=all not found` | CDI not configured for rootless podman | Run `nvidia-ctk cdi generate --output=/etc/cdi/nvidia.yaml` (Silverblue: layer it via rpm-ostree) |
| ComfyUI returns `404 /prompt` | Wrong port — ComfyUI listens on `/api/prompt` in some builds | check `curl http://localhost:8188/system_stats` first |
| `/opt/ComfyUI/models: permission denied` | SELinux relabel missing on the bind | `:Z` is in the script; if you mounted by hand, add it |
| Models not persisting in ollama | Forgot the `tools/ollama-models` bind mount | use `start-inference.sh`, not a hand-rolled `podman run` |

## Trust boundary cross-references

- The trusted-side HTTP clients live in `crates/tt-comfy/` (ComfyUI) and `crates/tt-qa/` (ollama).
- The hardening-flag list is the canonical source at `crates/tt-core/src/lib.rs` `tt_core::podman::DEFAULT_FLAGS` — drift between this script and that constant is a `tt-lint` canon failure.
- The `# tt-lint: inference-role` pragma in this script's `podman run` block is what tells `tt-lint` that `--network=none` and `--rm` are intentionally absent for this role (long-running HTTP-served service).

## See also

- `openspec/specs/isolation/spec.md` — full boundary contract
- `openspec/specs/orchestrator/spec.md` — how `tt-render` consumes this container
- `openspec/specs/visual-qa-loop/spec.md` — VLM checks driven against the ollama side
- `openspec/specs/character-loras/spec.md` — LoRA bind-mount path under `tools/loras/`
- `docs/training-lifecycle.md` — end-to-end storage-layer map
- `images/inference/Containerfile` — how the image is built
