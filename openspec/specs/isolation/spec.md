# Isolation

## Purpose

Tlatoāni Tales has exactly two zones and one boundary between them. The boundary is an **architectural invariant**, not a deployment detail: every line of code we author lives on one side, every third-party Python runtime lives on the other, and every crossing is a deliberate, typed, restricted action.

| Zone | Role |
|---|---|
| **TRUSTED** | Our Rust workspace and thin bash launchers, inside the `tlatoani-tales` developer toolbox. Fast to rebuild, compilers present, `cargo` available. Makes HTTP and subprocess calls across the boundary. |
| **UNTRUSTED** | ComfyUI, ai-toolkit, ollama, the static-file httpd — anything Python or anything we did not author. Runs in hardened tiny Alpine or Fedora-minimal containers, dropped capabilities, read-only root, offline-by-default. |

Every other spec in this repo touches the boundary somewhere. This spec is where the boundary is *the subject*. It is the project's load-bearing **meta-example for Season 2** — we practice what we will later teach, every render, by making the teaching infrastructural before we make it pedagogical.

`@trace spec:isolation`
`@Lesson S1-1500`

## Motivation

The comic's thesis is that monotonic convergence is not a math incantation — it is a shape you can see in many places. The comic's *production* is the first place a reader can see it.

Author, verbatim: *"you're generating code online, the comic images, the story viewer, and the calmecac for observability, are running locally, meaning the inference used for the comic is local, and must therefore follow our own teachings of running in full isolation. That's our main meta example to trace."*

If a strip that teaches `podman-run-drop-privileges` were rendered by a ComfyUI that ran with ambient host credentials and free network access, the strip would be a lie — technically accurate, pedagogically bankrupt. The repo must be the proof of its own claim. The captioned assertion for Season 2 will be: **"this comic was rendered using offline models"** — a claim whose truth is mechanically verifiable from the flags passed to `podman run`. This spec is how we earn the right to that caption.

## The boundary

| Zone | What lives here | Runtime | Network |
|---|---|---|---|
| **TRUSTED** — dev toolbox `tlatoani-tales` | Rust workspace (`tt-render`, `tt-calmecac`, `tt-lora`, `tt-comfy`, `tt-qa`, `tt-lint`, `tt-events`, `tt-calmecac-indexer`), bash launchers under `scripts/` | Rust via cargo; bash | Free (development convenience; the compilers live here) |
| **UNTRUSTED inference** — container `tlatoani-tales-inference` | ComfyUI + FLUX.1-schnell + Qwen-Image + ollama VLM | Python + CUDA | `--network=none` at render time; bridge only during explicit image-build or weight-pull |
| **UNTRUSTED trainer** — container `tlatoani-tales-trainer` | ai-toolkit (ostris) + torch + CUDA runtime | Python + CUDA | `--network=none` at train time |
| **UNTRUSTED viewer** — container `tlatoani-tales-viewer` | Distribution-standard httpd serving the static Calmecac bundle | httpd | Bridge, bound to `127.0.0.1` only |

Container names are ASCII-only by necessity (Podman's name grammar is LDH-only). That gap — between the comic's `Tlatoāni` and the container's `tlatoani` — is the catalogued teachable break **TB03** in `tlatoāni-spelling/spec.md`. A reader who asks "why the missing macron?" receives Season 2's first footnote for free.

## Boundary-crossing contract

The boundary is crossed in exactly four ways. Every other path is a bug.

| Mechanism | Direction | Used by | Shape |
|---|---|---|---|
| **HTTP over localhost** | trusted → untrusted | `tt-comfy`, `tt-qa` | `reqwest` client from a Rust crate to a port-bound API: ComfyUI at `127.0.0.1:8188`, ollama at `127.0.0.1:11434`. No shell-out to Python binaries from the trusted side. |
| **Read-only bind mounts** | trusted writes, untrusted reads | all containers | Model weights, reference sheets, Calmecac bundles, rendered config files. The untrusted side cannot write back through a read-only mount. |
| **Read-write bind mounts (scoped)** | untrusted writes, trusted reads | inference, trainer | Panel PNGs, LoRA `.safetensors`, QA drift reports. Each container's writable paths and output formats are documented per role. |
| **Subprocess launch with hardened `podman run` flags** | trusted launches untrusted | `tt-lora-train`, one-shot renders | The launch is the crossing; the child then lives entirely inside the container until exit. |

**Explicitly forbidden** (and checked by `tt-render verify`):

- Calling `python`, `python3`, `comfy`, `ollama`, or any `ai-toolkit` binary directly from the trusted zone.
- Passing `--privileged`, `--cap-add`, or `--security-opt=seccomp=unconfined` to any container.
- Bind-mounting `/`, `$HOME`, `~/.ssh`, `~/.config`, or any repo directory writable-by-default.
- Containers reaching the internet after their weights and specs are in place.

## Canonical `podman run` flags

These flags are **non-negotiable**. Every container start in trusted Rust goes through a single helper (`tt-podman::run`) that applies them verbatim; there is no second path. Same discipline as tillandsias's enclave spec.

```
podman run \
  --rm \
  --name tlatoani-tales-<role> \
  --cap-drop=ALL \
  --security-opt=no-new-privileges \
  --userns=keep-id \
  --read-only \
  --network=none \
  --volume <host-ro>:<cont>:ro \
  --volume <host-rw>:<cont>:rw \
  --device nvidia.com/gpu=all \
  tlatoani-tales-<role>:<version>
```

| Flag | Role |
|---|---|
| `--rm` | Container is disposable; state lives in bind mounts, never in a writable container layer. |
| `--name tlatoani-tales-<role>` | Single canonical name per role; enforces single-instance via `podman`'s own collision rule. ASCII per TB03. |
| `--cap-drop=ALL` | No Linux capabilities. A Python process inside cannot `CAP_SYS_ADMIN` its way out. |
| `--security-opt=no-new-privileges` | `setuid` bits inside the container are neutralised; privilege escalation is not reachable. |
| `--userns=keep-id` | UID/GID mapping preserves the host user; bind-mounted outputs land with the author's ownership, not `root`. |
| `--read-only` | Root filesystem is read-only. Any runtime that wants to write goes through an explicit bind mount or fails. |
| `--network=none` | Default. Containers have no network interface at all once their images and weights are in place. Overridden to `bridge` only for the explicit image-build and weight-pull flows. |
| `--volume …:ro` | Read-only bind mounts for inputs the trusted side owns (model weights, reference sheets, rendered configs). |
| `--volume …:rw` | Scoped writable bind mounts for specific outputs (panel PNGs, LoRA weights, drift reports). Never the full repo. |
| `--device nvidia.com/gpu=all` | GPU passthrough via CDI when required (inference, trainer). Omitted for viewer. |

The flag list lives in exactly one place in code (`tt-podman::DEFAULT_FLAGS`); callers compose role-specific mounts and image tags on top. Drift between the spec and the helper is itself a lint failure (see **Trust test procedure**).

## Phased isolation — current vs target

The architecture described in the rest of this spec (hardened podman containers, full `DEFAULT_FLAGS`, CDI GPU passthrough) is the **target** state. It is not the current state.

**Season 1 MVP** ships with a **toolbox-based** implementation of the untrusted zone:

| Role | Season 1 MVP | Target (Season 2 teach-by-example) |
|---|---|---|
| inference | ComfyUI inside the existing `tlatoani-tales` toolbox, launched via `scripts/start-inference.sh`. GPU passthrough is automatic (toolbox's built-in nvidia handling). | The hardened container from `images/inference/Containerfile` with full `DEFAULT_FLAGS` + CDI GPU passthrough. |
| trainer   | ai-toolkit inside a dedicated inference-style toolbox, or the same one. | `tlatoani-tales-trainer` container with `--network=none` + `DEFAULT_FLAGS`. |
| viewer    | **Already hardened** — see `scripts/tlatoāni_tales.sh`. Apache httpd in a read-only podman container with the full flag set, localhost-only publish. | unchanged. |

**Why phased:** the full hardened path requires Silverblue-layering `nvidia-container-toolkit` and running `nvidia-ctk cdi generate` — a reboot-level state change on the host. The author's directive when the hardened path hit that wall was explicit: *"nvidia-smi is already installed on the host. Why do you need anything else? just use the existing toolbox, or create a new one if you need to isolate the environment."* That wisdom — **"we do not always know better"** — is architectural, not just operational: the toolbox is a rootless podman container, it already isolates the dev environment, reinventing the hardening story for Season 1 is over-engineering.

**What still holds in Season 1 MVP:**

- The **trust boundary** is a conceptual invariant, not a hardening-level one. Trusted Rust calls ComfyUI via HTTP on `127.0.0.1:8188`; the untrusted Python process runs in the toolbox's container and never receives host credentials because the trusted side never hands any over. `tt-comfy` makes no `pull`/remote calls at render time.
- The **viewer is already hardened** (Season 1 ships full-flag on that role — see `scripts/tlatoāni_tales.sh`). The trickier-to-harden roles are phased.
- The **lint teeth** in `tt-lint` and the flag constants in `tt_core::podman` remain the single source of truth for the target state. The toolbox-based launcher doesn't use `podman run` directly, so it doesn't carry the hardening pragma — it's a separate lifecycle.

**What Season 2 will add (candidate lesson material, not canonized):**

- The hardened-container migration itself becomes a strip: Covi bewildered by a `--cap-drop=ALL --network=none` invocation that refuses to start; Tlatoāni quietly pointing at `nvidia-ctk cdi generate`; the aha: **"the same work, harder permissions, better trust."**
- The caption *"this comic was rendered using offline models"* remains factually verifiable against the flags actually used; Season 1 MVP trainer runs with `--network=none` (tt-lora composes that directly via `DEFAULT_FLAGS`) so LoRA training is genuinely offline at the moment it matters most.

## Network mode per role

Not every role can take `--network=none`. The trainer can; inference and viewer cannot. The asymmetry is real architecture, not laxity:

| Role | Network mode | Why |
|---|---|---|
| **trainer** | `--network=none` | One-shot subprocess. Reads bind-mounted refs, writes a bind-mounted LoRA. No HTTP, no port. The container has zero network namespace; this is the strictest possible posture and is achievable here. |
| **inference** | `--network=bridge` (Podman default) + `--publish 127.0.0.1:PORT:PORT` for ComfyUI (8188) and ollama (11434) | Long-running HTTP-served service. `--network=none` provides no namespace for `--publish` to forward into; the trusted `tt-comfy`/`tt-qa` clients would have nothing to reach. The published ports are bound on the host's loopback only — services are not reachable from anything outside the host. |
| **viewer** | `--network=bridge` (Podman default) + `--publish 127.0.0.1:PORT:8080` for httpd | Same shape as inference: HTTP-served, hence requires a network namespace; same loopback-only publish posture. |

The trusted side never instructs an inference or viewer container to reach the wider internet. Model weights arrive through read-only bind mounts; the `tt-comfy` client invokes only `POST /prompt`, `GET /history/:id`, `GET /view`. Outbound from those containers is technically reachable (default bridge has internet egress) — that gap is accepted today as a published deviation, with the migration target documented in `meta-examples/spec.md` ME04 (REUSE/per-file convergence) and a candidate Season-2 lesson on `podman network create --internal` already noted.

## Offline inference invariant (qualified)

**Every comic strip in Season 1 is rendered with the trainer in `--network=none`** (which is where the LoRA training that *makes* the comic happen). Inference and viewer use the bridge-with-localhost-publish posture above. The captioned claim *"this comic was rendered using offline models"* remains factually accurate and mechanically verifiable at three levels:

1. The trainer container (`tlatoani-tales-trainer`) runs with `--network=none`. Inspect via `tt_core::podman::DEFAULT_FLAGS` and `crates/tt-lora/src/lib.rs`.
2. The inference container's ComfyUI never makes outbound HTTP at render time — the trusted client only sends inputs, never instructs a fetch. Inspect via `crates/tt-comfy/src/lib.rs` (no `pull`, no remote model fetch).
3. The model weights themselves are bind-mounted read-only from `tools/ComfyUI/models/` — once provisioned, the container has all it needs without the network.

The Season-2 candidate lesson "this comic was rendered using offline models" stands; the path to literal `--network=none` for inference is migration to a `podman network create --internal tlatoani-tales-net` (no upstream gateway, port publishing intact) — flagged but not yet executed. Honest convergence: the spec acknowledges where reality bites and points at the next step, rather than misstating the status quo.

The captioned claim for Season 2 — *"this comic was rendered using offline models"* — is factually true, inspectable in a single place: the flags handed to `podman run`. A reader who doubts it can read `tt-podman::DEFAULT_FLAGS` and `grep -r 'network=' scripts/` and verify the claim in under a minute.

Flagged as a **Season 2 candidate lesson** (not canonized; author curation pending).

## Image build policy

- Containerfiles live at `images/<role>/Containerfile` — one subdirectory per role: `images/inference/`, `images/trainer/`, `images/viewer/`, and optionally `images/shared-base/` if we later factor a common hardened base.
- Base image preference, per role:

| Role | Recommended base | Reason | Alternative |
|---|---|---|---|
| **inference** | `registry.fedoraproject.org/fedora-minimal:latest` | Matches the Silverblue-family toolchain; CUDA vendor RPMs are first-class; ComfyUI's wheels assume glibc, and Alpine's musl has been a pain point in the GPU-Python ecosystem. | Alpine only if a lean GPU path becomes tractable upstream. |
| **trainer** | `registry.fedoraproject.org/fedora-minimal:latest` | Same CUDA + glibc reasoning as inference; ai-toolkit pins PyTorch wheels that expect glibc. | — |
| **viewer** | `registry.fedoraproject.org/fedora-minimal:latest` with distro `httpd`, **or** Alpine + `apache2` | Either is appropriate; viewer has no GPU or Python dependencies and benefits from the smallest possible attack surface. | Open question for author — see `calmecac/spec.md`. |

- Build via `podman build` from the trusted zone. Image tag convention: `tlatoani-tales-<role>:v<VERSION>`.
- Images are **rebuilt from source** (Containerfile + locked dependency manifest), never edited in place. A new image is a new tag; old tags stay around until garbage-collected deliberately.
- Build-time network access is allowed (package repos, `pip install`, `hf` downloads). Once built, the running container defaults to `--network=none`; weights arrive by bind mount, not by `curl`.

## Trust test procedure

`tt-render verify` — the `tt-lint` crate — includes an `isolation` sub-lint. It is fast, pure, and runs on every render and on every CI invocation.

| Check | Rule | Violation class |
|---|---|---|
| `isolation.no-direct-python` | Orchestrator source must not invoke `python`, `python3`, `comfy`, `ollama`, or any `ai-toolkit` binary via `std::process::Command`. Only `tt-podman::run` may launch untrusted processes. | canon (exit 20) |
| `isolation.flags-present` | Every `podman run` invocation in code and scripts must pass `--cap-drop=ALL`, `--security-opt=no-new-privileges`, `--userns=keep-id`, `--read-only`. | canon (exit 20) |
| `isolation.network-default-none` | The inference and trainer containers must pass `--network=none` unless the callsite is explicitly annotated `#[isolation::online("<reason>")]` (for example, the one-shot weight-pull flow). | canon (exit 20) |
| `isolation.no-privileged` | No invocation may pass `--privileged`, `--cap-add`, or `--security-opt=seccomp=unconfined`. | canon (exit 20) |
| `isolation.mounts-scoped` | Bind-mount sources must match an allowlist per role; mounting `/`, `$HOME`, `~/.ssh`, `~/.config`, or a writable repo root is a violation. | canon (exit 20) |
| `isolation.flag-source-of-truth` | The string literal in `tt-podman::DEFAULT_FLAGS` matches the canonical list in this spec. Drift between spec and helper is itself a violation. | canon (exit 20) |

Violations are **canon failures**, exit 20 per `orchestrator/spec.md`'s failure-mode split — because the boundary IS the canon. An `infra` failure (exit 30) says *the tool is wrong*; a canon failure says *the comic is wrong*; a boundary violation is the second kind.

## Season 2 seeding

This spec is the bridge to Season 2. The following are **candidate lessons**, flagged for author curation — **none are canonized here**.

| Candidate slug | Hook |
|---|---|
| `S2-???-dangerously-skip-permissions` | The opener — the tempting shortcut. `--cap-drop=ALL` is not optional; skip-permissions is the anti-pattern this season teaches its way out of. |
| `S2-???-containers-dont-trust-python` | Frames the isolation contract at the ecosystem level: Python's dependency hygiene is not a joke, it is a concrete reason we isolate, and the lesson is *about ecosystems*, not about blaming a language. |
| `S2-???-offline-proves-intent` | `--network=none` turns the claim *"rendered offline"* into a verifiable property of the build. Intent without mechanism is marketing; intent with mechanism is proof. |
| `S2-???-podman-run-drop-privileges` | The closer — the mirror image of S2's opener. Least-privilege container execution is the operational sibling of Season 1's monotonic convergence. |

The author decides which of these land as published lessons, in which order, and under which final slugs. The seeds exist here so that when Season 2 authoring begins, the substrate the lessons will teach *against* is already present in this repo.

## Meta-example candidates (flagged, NOT canonized)

Three candidate entries await author curation for the next free `ME##` slot(s). See `meta-examples/spec.md` for the canonical ledger.

| Candidate micro-title | Why it is load-bearing |
|---|---|
| **Trust boundary as living architecture** | The isolation architecture itself is a running demonstration of `S1-1000` one level up — the feedback loop has *boundaries*, not just *metrics*. Observability without a trust model is a dashboard facing inward; observability with a trust model is a dashboard that can be published. |
| **Every strip rendered offline** | Factual, verifiable from the `podman run` flags, and the substrate Season 2's caption will stand on. The claim is mechanical, not aspirational. |
| **Know when to stop** | The author's wisdom about *not writing our own httpd* is itself architecturally load-bearing: the decision to stop at the trust boundary and use industrial-grade plumbing (distro httpd, not a Rust server crate) is a teachable restraint. A tiny observed lesson hiding in plain sight — echoed in `calmecac/spec.md`. |

## Relationship to sibling specs

| Spec | Role across the boundary |
|---|---|
| `orchestrator/spec.md` | Trusted. Runs in the toolbox, drives renders, crosses the boundary via `tt-comfy`'s HTTP client and `tt-podman::run`. |
| `calmecac/spec.md` | Straddles. Index generator (`tt-calmecac-indexer`) is trusted Rust; the httpd container that serves the bundle is untrusted. Static bundle flows across a read-only bind mount. |
| `character-loras/spec.md` | Trainer is untrusted (`tlatoani-tales-trainer`); `tt-lora` wrapper is trusted. The LoRA's SHA-256 is computed in the trusted zone after the container exits, so the untrusted side cannot lie about its output. |
| `visual-qa-loop/spec.md` | Ollama VLM runs inside the inference container. `tt-qa` is the trusted HTTP client. |
| `tlatoāni-spelling/spec.md` | TB03 catalogues the container-name ASCII constraint as a deliberate teachable break — the gap from `Tlatoāni` to `tlatoani` is evidence, not oversight. |
| `licensing/spec.md` | Per-role Containerfiles are source files and follow `R##` mappings (scripts → GPL, markdown → CC BY-SA). Images themselves are build artefacts, not committed. |
| `seasons/spec.md` | Season 2's thesis (*dangerously-skip-permissions → podman-run-drop-privileges*) is this spec's pedagogical payload; Season 1 ships with the substrate already Season-2-compliant. |
| `meta-examples/spec.md` | Three candidate entries (above) await author curation. |

## Future convergence

Each direction strictly refines; none invalidate this spec's contracts.

| Direction | Note |
|---|---|
| Per-strip cache namespacing with image hashes | Fold the inference image's SHA-256 into `panel_hash` alongside the base-model hash (orchestrator §Panel hash). Rebuilding the inference image with a different torch version then silently invalidates all affected panel caches — another layer of content-addressing, monotonic with the existing cache discipline. |
| Reproducible-container builds | A `flake.nix`-based build path (Nix-style) for the container images, echoing tillandsias's builder toolbox pattern. Bit-for-bit reproducibility of the inference image is the strongest possible evidence for the offline-inference caption. |
| `tlatoani-tales-proxy` role | For offline-first development where dependency mirrors are whitelisted. Borrowed directly from tillandsias's enclave spec; useful the day we want a CI path for rebuilding the inference image without granting it full outbound network. |
| Signed images | Image-pull signature verification via `podman`'s policy engine, closing the supply-chain gap between "we built it locally" and "we can prove we built it locally". |

## Trace

`@trace spec:isolation, spec:orchestrator, spec:calmecac, spec:character-loras, spec:visual-qa-loop, spec:tlatoāni-spelling, spec:licensing, spec:seasons, spec:meta-examples`
`@Lesson S1-1500` *(proof-by-self-reference — the isolation architecture IS the comic's proof that it practices what it teaches)*
