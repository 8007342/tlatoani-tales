# Script conventions

## Purpose

Every shell script in `scripts/` declares one of two execution zones and enforces it at startup. The two zones are not interchangeable, and a script that doesn't declare which zone it belongs to is ambiguous — which means some user will run it in the wrong place and get a confusing failure. We close that gap with a simple invariant: **every script starts with a zone-guard**.

Authored in response to an author smoke-test: `start-inference.sh` (which was supposed to run from the host and wrap its work in `toolbox run`) and `bootstrap-comfyui.sh` (which was supposed to run inside the toolbox and expected the venv's Python directly) had incompatible mental models for the same `scripts/` directory. Consistency is non-negotiable; the spec is how we keep it.

`@trace spec:script-conventions`
`@Lesson S1-1500`

## Invariants

- Every `scripts/*.sh` starts with a **zone-guard comment + check** that declares one of:
  - `# zone: inside-toolbox` — must run inside the `tlatoani-tales` toolbox
  - `# zone: host` — must run on the host shell (outside any toolbox)
- Scripts without a zone declaration are **canon violations** (`tt-render verify` exits 10 with rule `script.zone-guard`).
- The guard **exits non-zero with a clear, actionable error message** when run in the wrong zone. It never silently falls through.
- Zone assignments are committed to the repo (see §Canonical assignment below). New scripts pick a zone at authoring time.

## The two zones

| Zone | Marker | Use for | Why it's this way |
|---|---|---|---|
| **inside-toolbox** | `# zone: inside-toolbox` | Any script that reads `tools/ComfyUI/.venv/`, invokes `python`, runs `pip`, `hf`, or any tool provisioned by `bootstrap-comfyui.sh` | The Python environment, model weights, and CUDA toolkit live in the toolbox. Running these from the host shell either fails immediately (no venv on host) or silently uses a different Python (subtle, hard to debug). |
| **host** | `# zone: host` | Scripts that invoke `podman build`, `podman run`, or otherwise interact with the host's rootless podman storage to manage containers | `podman` is not installed inside the toolbox (verified 2026-04-23). Nested rootless podman would share user-namespace state awkwardly anyway. The host IS the container orchestrator. |

## How each zone detects itself

| Check | Meaning | Rationale |
|---|---|---|
| `/run/.toolboxenv` exists | inside a toolbox | Canonical file `toolbox` creates to self-identify. Stable across Silverblue versions. |
| `/run/.containerenv` → `name=tlatoani-tales` | inside *our* toolbox | Distinguishes from other user toolboxes. `hostname` defaults to `toolbx` and is unreliable. |
| `/run/.toolboxenv` missing | host shell | Inverse. Host shell never creates this file. |

## Canonical guard form

### `# zone: inside-toolbox`

```bash
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
```

### `# zone: host`

```bash
# zone: host
# See openspec/specs/script-conventions/spec.md
if [[ -f /run/.toolboxenv ]]; then
  echo "ERROR: this script must run on the HOST shell (not inside any toolbox)." >&2
  echo "       exit the toolbox (Ctrl-D or 'exit') and retry." >&2
  exit 1
fi
```

## Canonical assignment (Season 1 MVP)

| Script | Zone | Why |
|---|---|---|
| `scripts/bootstrap-comfyui.sh` | `inside-toolbox` | installs python3.12, pip deps, clones ComfyUI into the toolbox |
| `scripts/download-models.sh`   | `inside-toolbox` | uses the venv's `hf` CLI to pull weights |
| `scripts/start-inference.sh`   | `inside-toolbox` | launches the venv's python directly — no more `toolbox run` wrapper |
| `scripts/tlatoāni_tales.sh`    | `host`           | `podman build` + `podman run` the viewer container against host's storage |

Any future `scripts/*.sh` is allocated a zone at authoring time. No script may remain zone-unspecified.

## Enforcement — `tt-lint`

`tt-lint` rule `script.zone-guard` walks `scripts/*.sh` and verifies each carries one of the two `# zone: ...` declarations. Missing or ambiguous = canon violation (exit 10 from `tt-render verify`). The rule lives alongside the other isolation lints; the pattern mirrors the existing `# tt-lint: viewer-role` / `inference-role` pragmas in spirit — lint teeth that enforce spec-level invariants at source level.

## Why two zones instead of one

An earlier revision tried to standardize on "inside-toolbox for everything." That broke the viewer launcher because `podman` is not installed inside the toolbox and nested rootless podman has real quirks. Rather than install podman inside and deal with user-namespace layering, this spec **acknowledges two legitimate zones** and enforces that every script is honest about which it needs.

This is a small **proof-by-self-reference** beat (S1-1500 material): the convergence methodology the comic teaches applies to the scripts that render the comic. We don't paper over architectural differences with workarounds; we declare them, enforce them, and keep them consistent.

Candidate meta-example (NOT canonized — author curation pending): *"Consistency over uniformity"* — consistent is the rule every script follows the same shape; uniform would have been forcing every script into the same zone at the cost of architectural honesty. We pick consistent.

## Trace

`@trace spec:script-conventions, spec:isolation, spec:orchestrator, spec:licensing`
`@Lesson S1-1300`
`@Lesson S1-1500`
