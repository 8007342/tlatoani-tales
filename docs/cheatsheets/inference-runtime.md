# Inference runtime — cheatsheet (Season 1 MVP)

<!-- @trace spec:isolation, spec:orchestrator, spec:visual-qa-loop, spec:character-loras -->
<!-- @Lesson S1-1300, S1-1500 -->

Operational reference for running ComfyUI + (optionally) ollama for the Tlatoāni Tales render pipeline. **Season 1 MVP ships a toolbox-based implementation** — ComfyUI runs inside the existing `tlatoani-tales` toolbox, which already has GPU passthrough. The hardened-podman-container path (`images/inference/Containerfile`, full `DEFAULT_FLAGS`, CDI) remains authored as Season 2 teach-by-example material; see `openspec/specs/isolation/spec.md` §Phased isolation for why.

## TL;DR

```
scripts/start-inference.sh                   # start ComfyUI inside the toolbox (idempotent)
scripts/start-inference.sh --status          # is it running?
scripts/start-inference.sh --stop            # stop cleanly
curl -s http://localhost:8188/system_stats   # ComfyUI healthcheck
```

## What the launcher does

1. Refuses on any OS that isn't Fedora Silverblue (`VARIANT_ID=silverblue` in `/etc/os-release`).
2. Checks the `tlatoani-tales` toolbox exists and `tools/ComfyUI/.venv/bin/python` is present (both provisioned by `scripts/bootstrap-comfyui.sh`).
3. Reads `tools/inference/comfyui.pid`. If a live process matches, reports "already running" and exits 0. If the pid file is stale, cleans it up.
4. Refuses if the requested port is already bound on the host.
5. Launches ComfyUI under `setsid nohup toolbox run -c tlatoani-tales …` so the whole process group is killable from the parent. Writes the leader PID to the pid file, streams stdout/stderr to `tools/logs/comfyui.log`.
6. Waits 2 seconds and verifies the process is still alive; if not, tails the log and exits 2.

**Idempotency invariants** (all verified in the script):

- Start twice → second call reports "already running" and exits 0.
- Start → `--stop` → start → second start gets a fresh process.
- Start, process dies externally → next start cleans the stale PID and begins fresh.
- Port already bound → exits 1 with a clear message before forking anything.

## What the script does NOT do (and why that's fine for Season 1)

| Feature | Why deferred |
|---|---|
| Hardened `podman run` with `--cap-drop=ALL --read-only --userns=keep-id` | The toolbox is a podman container itself; Season 2 will teach the migration to full hardening as an on-screen lesson. |
| CDI GPU passthrough (`--device nvidia.com/gpu=all`) | Requires Silverblue-layering `nvidia-container-toolkit` + `nvidia-ctk cdi generate` + reboot. Toolbox handles GPU automatically today. |
| `--network=none` for inference | ComfyUI is HTTP-served; `--network=none` would have no namespace for `--publish` to forward into. See isolation/spec.md §Network mode per role. Trainer **does** use `--network=none` via `tt-lora`. |
| ollama integration | ollama isn't installed in the toolbox yet. For smoke-testing the render pipeline we run with `TT_QA=off` (skip VLM drift scoring); ollama will be added as the next increment. |

## Process model

```
<your shell>                    <-- you invoke start-inference.sh here
  └─ setsid nohup                <-- new session + process group
       └─ toolbox run            <-- wrapper that enters the container
            └─ podman exec       <-- under the hood
                 └─ bash -c      <-- runs cd + exec python
                      └─ python main.py --listen 127.0.0.1 --port 8188
```

- The **PID file** stores the toolbox-run wrapper's PID (parent of everything below).
- `--stop` sends SIGTERM to the whole process group (kill -- -${pid}), which cascades down to the Python process cleanly.
- Force-kill kicks in after 5 seconds if SIGTERM isn't respected.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| `Local inference supports Fedora Silverblue only.` | Not on Silverblue, or `/etc/os-release` missing `VARIANT_ID=silverblue` | That's the whole point. Season 2 teaches why. |
| `Toolbox 'tlatoani-tales' not found` | Toolbox never created | `toolbox create tlatoani-tales && scripts/bootstrap-comfyui.sh` |
| `ComfyUI venv missing at …/.venv` | Bootstrap never ran or venv got trashed | `scripts/bootstrap-comfyui.sh` |
| `port 8188 is already in use` | Prior instance didn't clean up, or another service is on 8188 | `scripts/start-inference.sh --stop`, or pick `--port N` |
| Starts then dies within 2s | Broken weights / broken pip env / broken CUDA setup | `tail -100 tools/logs/comfyui.log` — the tail is printed automatically when early-death is detected |
| `ComfyUI 200 OK` on `/system_stats` but `/prompt` fails | Wrong endpoint — some ComfyUI builds expose it at `/api/prompt` | check via `curl -s http://localhost:8188/object_info | head` first |
| Stale pid file blocks a restart | Process was killed externally (reboot, OOM) | The script detects this automatically and cleans up; worst case `rm tools/inference/comfyui.pid` manually |

## Trust boundary cross-references

- Trusted-side HTTP clients: `crates/tt-comfy/` (ComfyUI), `crates/tt-qa/` (ollama — pending)
- Hardening-flag source of truth: `crates/tt-core/src/lib.rs` → `tt_core::podman::DEFAULT_FLAGS`
- Viewer (already hardened, full-flag): `scripts/tlatoāni_tales.sh`
- Season 2 hardened-inference target: `images/inference/Containerfile` (authored + image built, not the active path)

## See also

- `openspec/specs/isolation/spec.md` — full boundary contract, including §Phased isolation
- `openspec/specs/orchestrator/spec.md` — how `tt-render` consumes this runtime
- `openspec/specs/visual-qa-loop/spec.md` — VLM checks (will engage when ollama lands in the toolbox)
- `docs/training-lifecycle.md` — end-to-end storage-layer map
