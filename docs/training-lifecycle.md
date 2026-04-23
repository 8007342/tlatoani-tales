# Training Lifecycle

Architectural manual for how character LoRAs are bootstrapped, trained, sealed,
consumed, and invalidated across Tlatoāni Tales. Companion to
`openspec/specs/character-loras/spec.md` — where the spec pins *what the
contract is*, this document pins *what lives where, for how long, and at whose
expense*.

This doc is descriptive, not prescriptive. Every normative rule here has a
governing spec — the table rows point at them. If this doc and a spec disagree,
the spec wins and this doc is a bug.

@trace spec:character-loras, spec:orchestrator, spec:isolation, spec:visual-qa-loop, spec:calmecac, spec:licensing, spec:trace-plate
@Lesson S1-500
@Lesson S1-1000
@Lesson S1-1500

---

## 1. Purpose & audience

Two readers.

**Primary: the author.** You write at the concept level —
*"Tlatoāni must always single-tail"*, *"TT 12/15 panel 3 needs less clutter"* —
and the pipeline converges pixels to match. You never touch Python, never curate
a batch size, never delete a cache entry. See
`feedback_tlatoāni_tales_workflow.md` for the discussion-to-convergence loop.
This document tells you which artefacts of that loop live in git, which live on
disk, which live only inside a running container, and what you will see on the
dashboard when something misbehaves.

**Secondary: a maintainer who needs to reason about disk, GPU time, trust
boundaries, or cache invalidation.** The tables below are sized to answer
"where does this live?", "who deletes it?", "how big is it?" in under thirty
seconds of scanning. Ballpark numbers are flagged as *ballpark*; measured
numbers are flagged as *measured*.

**The convergence model.** Authoring is a conversation. A conversation becomes
a typed spec mutation. A typed spec mutation becomes a content-hash cascade.
A content-hash cascade becomes a narrow re-render. The pipeline is the
*translator* that takes author intent and reduces it to the smallest possible
mechanical action. Its invariants (content addressing, typed events,
offline-by-default) exist so that "narrowing the re-render" is mechanical,
not a human judgment call.

Per `feedback_tlatoāni_tales_workflow.md`: authors decide teachings, sequence,
punchlines, tonal judgments; the pipeline decides pixels, drift minimization,
cache propagation, composition, plates, and metadata. The LoRA subsystem is
where that handoff gets enforced at the pixel level.

---

## 2. Storage-layer map

The repo sits on top of six distinct storage layers. The axis that matters is
**who is authoritative, for how long, and at what size**.

| Layer | Path (relative to repo root) | Committed? | Ephemeral? | Size | What lives here |
|---|---|---|---|---|---|
| **Git repo — archival** | `characters/<name>/references/*.png` | yes (CC BY-SA 4.0 per R05) | no | 24–40 MB / character | The reference corpus. The LoRA is reproducible from these. |
| **Git repo — archival** | `characters/<name>/lora-manifest.json` | yes (R08) | no | < 2 KB / character | The reproducibility contract (hyperparams, hashes, trigger token). The *weights* regenerate; the manifest is the commitment. |
| **Git repo — archival** | `characters/<name>/README.md` | yes (R04) | no | few KB | Human-readable character invariants + pipeline pointers. |
| **Git repo — archival** | `openspec/specs/**/*.md`, `strips/NN-slug/proposal.md`, `strips/NN-slug/panels/*.prompt.md` | yes (R04) | no | few KB each | Specs and strip proposals — the *intent* the pipeline converges against. |
| **Git repo — archival** | `images/<role>/Containerfile` | yes (R13) | no | few KB each | Container recipes. Images themselves are build artefacts. |
| **Git repo — archival** | `crates/**/*.rs`, `Cargo.toml`, `Cargo.lock` | yes (R14 for lock) | no | tens of KB to low MB | The trusted Rust workspace. |
| **Gitignored local disk — inputs** | `tools/ComfyUI/models/checkpoints/flux1-schnell-fp8.safetensors` | no (via `tools/` in `.gitignore`) | no (re-downloadable) | ~17 GB | FLUX.1-schnell fp8 all-in-one checkpoint. One per machine. |
| **Gitignored local disk — inputs** | `tools/ComfyUI/models/checkpoints/Qwen-Image/` | no | no | ~45 GB *measured* | Qwen-Image weights for stylized title-plate rendering. |
| **Gitignored local disk — outputs** | `tools/loras/<name>-v<N>.safetensors` | **no** | partially (regenerable, kept across runs) | ~150–300 MB / character | Trained LoRA weights. Regenerated from references + manifest hyperparams. |
| **Gitignored local disk — ephemeral config** | `tools/loras/<name>-train.json` | no | yes | few KB | Per-run ai-toolkit config rendered by `tt-lora`. Dropped into the trainer's read-only bind mount; regenerated every run. |
| **Gitignored local disk — ephemeral cache** | `cache/panels/<panel_hash>.png`, `cache/panels/<panel_hash>.json` | no | no (valid until a spec mutation invalidates them) | ~0.5 GB for a full Season 1 | Rendered panels keyed by content hash. Never overwritten; evicted only on bad PNG or manifest mismatch. |
| **Gitignored local disk — ephemeral outputs** | `output/Tlatoāni_Tales_NN.png`, `output/Tlatoāni_Tales_NN.json` | no | yes (rebuilt every run) | ~2 MB / strip × 15 strips | Composited final strips + metadata. `output/` is wiped safely any time. |
| **Gitignored local disk — ephemeral outputs** | `output/calmecac-bundle/` | no | yes | few MB | Viewer bundle emitted by `tt-calmecac-indexer`. Bind-mounted read-only into the viewer container. |
| **Container image** | `tlatoani-tales-trainer:<tag>` | no (built from Containerfile) | no | ~5–8 GB *ballpark* | ai-toolkit + torch + CUDA runtime. Built once, reused per character. |
| **Container image** | `tlatoani-tales-inference:<tag>` | no | no | ~5–8 GB *ballpark* | ComfyUI + FLUX + Qwen-Image + ollama VLM. Built once, reused per render. |
| **Container image** | `tlatoani-tales-viewer:<tag>` | no | no | **167 MB** *measured* first-build | Fedora-minimal + httpd, static-file server. Shipped. |
| **Container runtime — disposable** | inside `tlatoani-tales-trainer` | n/a | yes | — | Lives only while `tt-lora::train` is running. `--rm`, `--read-only`, `--network=none`. Exits, taking its RAM and any tmpfs writes with it. |
| **Container runtime — disposable** | inside `tlatoani-tales-inference` | n/a | yes | — | Lives across a render wave. Same hardening. Exits on `tt-render` shutdown. |
| **Container runtime — long-running** | inside `tlatoani-tales-viewer` | n/a | yes | — | Lives while the author is browsing Calmecac. Stateless: all state is in the read-only bundle mount. |
| **Read-only bind mount** | `characters/<name>/references/` → `/workspace/references/` in trainer | n/a | n/a | 24–40 MB | Inputs the trusted side owns; the container can read and cannot write. |
| **Read-only bind mount** | `tools/ComfyUI/models/checkpoints/flux1-schnell-fp8.safetensors` → `/workspace/base/` | n/a | n/a | ~17 GB | Base model for training. Read-only. |
| **Read-only bind mount** | `tools/loras/<name>-train.json` → `/workspace/config/` in trainer | n/a | n/a | few KB | Rendered ai-toolkit config. Read-only by design — the trainer must not rewrite the config it was given. |
| **Read-only bind mount** | `output/calmecac-bundle/` → `/usr/local/apache2/htdocs/` in viewer | n/a | n/a | few MB | Viewer content. Untrusted httpd serves bytes produced by trusted Rust. |
| **Read-write bind mount** | `tools/loras/` → `/workspace/output/` in trainer | n/a | n/a | up to ~300 MB / character | The **only** writable path visible to the trainer. The `.safetensors` lands here; `tt-lora` hashes it post-facto from the trusted side. |
| **Read-write bind mount** | `cache/panels/` → inside inference container | n/a | n/a | ~0.5 GB (full S1) | Rendered panels flow back to trusted-side-visible filesystem. |

**The only reason a file is committed is that losing it would break
reproducibility from the committed set.** The LoRA weights are not committed
because `references/ + manifest/` is a full reproduction recipe; the reference
*art* is committed because regenerating it would require re-curating (and is
legally the author's authored work per R05). The manifest is committed because
its `lora_hash` is the commitment — it names the binary without shipping it.

See the decision tree in §6.

---

## 3. Disk-size expectations

Concrete numbers for planning. Ballpark where not measured.

| Artefact | Size | Confidence | Notes |
|---|---|---|---|
| FLUX.1-schnell fp8 all-in-one | ~17 GB | *ballpark* | Single `.safetensors`. One per machine. Pinned by SHA-256 in every LoRA manifest. |
| Qwen-Image | ~45 GB | *measured* | Observed on first install. Used only for stylized title-plate text. |
| Character references (per character) | 24–40 images × ~1 MB ≈ 24–40 MB | *ballpark* | 1024×1024 PNGs. Author-curated or LoRA-less first-pass. |
| Trained LoRA (rank 16 FLUX, per character) | ~150–300 MB | *ballpark* | `.safetensors`. Landed by the trainer into the rw bind mount; hashed by `tt-lora` afterwards. |
| Panel cache (full Season 1) | ~0.5 GB | *ballpark* | 15 strips × 3 panels × ~10 MB per 1024-tall panel PNG. Grows only on spec mutation; monotonic between mutations. |
| Composited output (full Season 1) | ~2 MB × 15 ≈ 30 MB | *ballpark* | Final-PNG + METADATA.json per strip. Rebuilt from cache; wipe `output/` any time. |
| Telemetry JSONL | few MB / run | *ballpark* | `output/telemetry/<strip>.jsonl` — one line per event. Grep-first debugging. |
| Inference image | ~5–8 GB | *ballpark* | Fedora-minimal + Python 3.12 + torch (CUDA 12.4 wheels) + ComfyUI + ollama + venv. Build tooling stripped in a final layer. |
| Trainer image | ~5–8 GB | *ballpark* | Fedora-minimal + Python 3.12 + torch + ai-toolkit + venv. Same shape, different workload. |
| Viewer image | **167 MB** | *measured* | Fedora-minimal + `httpd`. Measured on first build. Ships as-is. |
| **Total MVP footprint** | **~80–100 GB** | *ballpark* | Dominated by Qwen-Image (45 GB), then FLUX (17 GB), then two GPU container images (~14 GB combined). Everything else rounds off. |

The ~80–100 GB is the *cold-install* number. A warm install that has already
downloaded weights and built images adds only per-character LoRAs (300 MB each)
and per-panel cache entries (10 MB each) between runs.

**What *shrinks* the footprint**: deleting `cache/panels/`, `output/`, and
`tools/loras/`. All three regenerate from committed artefacts + re-training
wall-time. A fresh checkout on a new machine that has weights already cached
(`tools/ComfyUI/models/`) rebuilds the rest in under two hours *ballpark* per
character.

---

## 4. Lifecycle walk-through

Step-by-step narrative from *"author decides Tlatoāni must always single-tail"*
to *"`tt-render` reaches into `tools/loras/tlatoani-v1.safetensors`"*. This is
the canonical path; every step cites the spec that governs it.

### 4.1 Author curates / bootstraps reference sheets

Reference images land under `characters/<name>/references/` as committed art.
Two valid origins:

1. **Author-curated.** Preferred — the LoRA inherits an author signature.
2. **LoRA-less FLUX bootstrap.** First-pass renders of the canon prompts,
   filtered through the VLM reference gate. Every image that *itself* fails
   canon (double-tail Tlatoāni, named Covi) is rejected before it contaminates
   the corpus. See `character-canon/spec.md` for invariants and
   `character-loras/spec.md §Reference gate` for the gate.

File naming: `<name>-<pose>-<expression>-NN.png`. ASCII-only, kebab-case.
Licensing: R05 (CC BY-SA 4.0) for PNGs, R07 for any SVG. The directory slug is
**ASCII** (`tlatoani/`) — see TB03 and `characters/README.md`.

The corpus targets 24–40 images per character — under 24 underfits, over 40
pays training time for no quality gain on a stylized character. See
`character-loras/spec.md §Reference sheet corpus` for the axes (pose variety,
expression variety, prop coverage, background, lighting).

### 4.2 Reference gate

Before training, every reference runs through the VLM (ollama) against
`character-canon/spec.md`. This is not training — it is *input hygiene*.

- Runs inside the `tlatoani-tales-inference` container (the same one that
  hosts the VLM at render time).
- Cross-boundary: the trusted `tt-qa` client posts the reference image to
  `http://127.0.0.1:11434/api/chat`; the container is `--network=none`.
- A reject writes a telemetry event (`QaEvent::CheckResult { pass: false }`);
  the reference is not deleted automatically, but the manifest's
  `dataset_hash` will not include it until the author confirms or replaces it.

Garbage in, garbage out. The LoRA is only as disciplined as its corpus.

### 4.3 Training config rendered from manifest hyperparameters

`tt-lora::LoraTrainer::render_ai_toolkit_config` takes the manifest's
`hyperparams` block and produces an ai-toolkit config JSON at
`tools/loras/<name>-train.json`. The file is **gitignored and ephemeral** —
re-rendered every run from the committed manifest. See `crates/tt-lora/src/lib.rs`.

Baseline v1 hyperparameters (per `character-loras/spec.md §Hyperparameters`):

| Param | Value |
|---|---|
| rank | 16 |
| alpha | 16 |
| steps | 2500 (spec calls for 2000–3000; v1 manifest pins 2500) |
| learning_rate | 1e-4 |
| batch_size | 1 |
| optimizer | AdamW8bit |
| precision | bf16 |
| resolution | 1024 |
| caption_dropout | 0.05 |

The config file is **not** the source of truth. The manifest is. If the config
file is deleted, `tt-lora` regenerates it on the next train. If the manifest is
deleted, reproducibility is broken — the manifest is in git for this reason.

### 4.4 `tt-lora::train` launches the trainer container

The trusted-zone Rust wrapper (`crates/tt-lora`) verifies the image exists
(`ensure_image`), composes the full `podman run` argv (`compose_podman_argv`),
and spawns the subprocess. The composed argv carries every flag from
`tt_core::podman::DEFAULT_FLAGS` — `--cap-drop=ALL`,
`--security-opt=no-new-privileges`, `--userns=keep-id`, `--read-only`,
`--network=none` — plus the role-specific mounts:

| Mount | Mode | Purpose |
|---|---|---|
| `characters/<name>/references/` → `/workspace/references/` | ro | Training inputs |
| `tools/ComfyUI/models/checkpoints/flux1-schnell-fp8.safetensors` → `/workspace/base/` | ro | Base model |
| `tools/loras/<name>-train.json` → `/workspace/config/train.json` | ro | Rendered config |
| `tools/loras/` → `/workspace/output/` | rw | **The only writable path** |

GPU passthrough is via CDI (`--device=nvidia.com/gpu=all`). No host paths
outside the four above are visible inside the container.

See `isolation/spec.md §Canonical podman run flags` for why each flag is
non-negotiable, and `openspec/specs/character-loras/spec.md §Trust boundary`
for the security posture.

### 4.5 Training runs

ai-toolkit's entrypoint (`/usr/local/bin/trainer-entrypoint.sh` inside the
image — see `images/trainer/Containerfile`) reads `TT_LORA_CONFIG` and invokes
`run.py` against the rendered config. stdout is the **only** observation
channel back across the boundary.

`tt-lora::parse_progress` compiles a static regex
(`(?i)step\s+(\d+)\s*/\s*(\d+)\s+loss\s+([0-9]+\.?[0-9]*)`) once per process
and emits typed events on the shared `tt-events` bus:

| Event | When |
|---|---|
| `LoraEvent::TrainStarted { character, version, config_hash }` | Before `podman run` spawns |
| `LoraEvent::StepProgress { character, step, total_steps, loss }` | One per matched stdout line |
| `LoraEvent::SanityRenderDone { character, prompt, drift_score }` | After each post-training sanity render |
| `LoraEvent::Trained { character, manifest }` | On successful sealing |
| `LoraEvent::Failed { character, reason }` | On infra or canon failure |

Wall time on RTX A5000 (24 GB) is roughly **45–90 minutes per character at
2000 steps** *ballpark* — first run slower due to text-encoder caching. The
LoRA lands at `tools/loras/<name>-v<N>.safetensors`.

### 4.6 Container stops; manifest is sealed and committed

The container is `--rm`. On exit its writable layer evaporates; everything it
produced that survives is in the rw bind mount.

`tt-lora` then, **in the trusted zone after the container has exited**:

1. Reads `tools/loras/<name>-v<N>.safetensors`.
2. Computes the SHA-256 of the bytes.
3. Writes `characters/<name>/lora-manifest.json` with `lora_hash` set to the
   computed digest, `trained_at` set to the current ISO-8601 UTC timestamp,
   and `sanity_render_scores` set to the drift summary from the sanity
   renders.
4. Emits `LoraEvent::Trained { character, manifest }`.

The sealed manifest is **committed to git**. The weights are **not**. This is
the architectural commitment: *the hash is the artefact*. The untrusted
trainer cannot lie about what bytes it produced, because the hash is computed
across the boundary by code it cannot influence. See the module doc comment on
`crates/tt-lora/src/lib.rs` — this is operational `@Lesson S1-1500`.

A reader who clones the repo gets: references + manifest + Containerfile +
Cargo workspace. From those, they can rebuild the trainer image, re-run
training, and compare their `lora_hash` against the manifest's. That *diff* is
the proof of the contract. That is the reproducibility promise from
`character-loras/spec.md §Out of scope`.

### 4.7 `tt-render` loads the LoRA at inference time

During a render wave, `tt-render` walks `characters/` and discovers every
manifest. For each, it:

1. Reads `characters/<name>/lora-manifest.json`.
2. Takes `lora_hash` out of the manifest.
3. Folds the manifest hash (via `tt_hashing::lora_manifest_hash`) into every
   `panel_hash` that *names* that character. Panel hash construction:

```
panel_hash = sha256(
  canonical(panel_prompt)              ||
  global_style_hash                    ||
  character_lora_hashes[present]       ||  // from lora-manifest.json
  seed                                 ||
  base_model_hash                      ||
  qwen_image_hash_opt                  ||
  schema_version
)
```

4. At render time, `tt-comfy` submits a ComfyUI workflow JSON that references
   `tools/loras/<name>-v<N>.safetensors` by path. The inference container has
   `tools/loras/` bind-mounted read-only; ComfyUI loads the LoRA into the
   FLUX pipeline.

Retraining → new `output.sha256` → new manifest hash → new panel hash →
silent, monotonic, bookkeeping-free cache invalidation of every panel that
named that character. That mechanism is `@Lesson S1-500` made operational.

---

## 5. Convergent vs measured

Two orthogonal axes run through the pipeline. Confusing them is the most
common author-facing bug.

### Convergent

Every artefact whose value is *derived monotonically* from upstream content.
Downstream invalidates when upstream changes; a given upstream produces exactly
one downstream state.

| Artefact | Derived from |
|---|---|
| `lora_hash` | references + manifest hyperparams + base model |
| Panel prompts | strip proposal + style bible + character canon |
| Panel renders (`cache/panels/<hash>.png`) | panel hash (which folds in everything above) |
| Composited strips (`output/Tlatoāni_Tales_NN.png`) | three panel hashes + title plate + chrome plates |
| `output/Tlatoāni_Tales_NN.json` (metadata) | strip proposal + registry lookups + rendered plates |
| `output/calmecac-bundle/calmecac-index.json` | every spec + every strip + every metadata file + git log |

Every convergent artefact is **content-addressed**. Mutate any input; the hash
changes; the downstream is stale; the orchestrator narrows the re-render to
exactly the affected cells. No global-invalidation sledgehammer.

This is `@Lesson S1-1000` materialized: the cache is not a dashboard, it is an
observability scaffold whose every transition is legible. When a strip
regenerates, telemetry tells you which hash changed and which cache entries
were evicted. Nothing happens silently.

### Measured

Every artefact whose value is *telemetry* — a measurement emitted by a
pipeline step. Measured artefacts feed the convergent machinery's decisions
but are themselves not inputs to hashes.

| Signal | Emitted by | Consumed by |
|---|---|---|
| Drift score per panel (`drift_score < 0.05`, `0.05–0.20`, `> 0.20`) | `tt-qa` against the VLM | Verdict: `Stable` / `Reroll` / `Escalate` / `NeedsHuman` |
| Sanity-render drift mean / max per LoRA version | `tt-lora` after training | Manifest `sanity_render_scores` field; decides promotion |
| Render wall time per panel | `tt-comfy` | Telemetry JSONL; surfaced in Calmecac's boring dashboards |
| Cache hit/miss per panel | Cache manager on the event bus | Author-facing progress + hit-ratio tracking |
| Reroll depth per panel | `tt-qa` / orchestrator | Escalation ladder (moondream → qwen2.5-vl); `needs-human` threshold |
| Step progress / loss during training | `tt-lora::parse_progress` | CLI progress bar, Calmecac live-watch |

Measurements drive the pipeline; the author reviews semantics. A drift of
0.06 is a machine decision ("reroll"); a drift of 0.50 that the machine
cleared as acceptable is an author concern ("why?"). The boring dashboards
in Calmecac exist for the latter case.

### Where the two axes meet

The manifest is the handoff point. It is **committed** (convergent — part of
the hash chain) *and* carries the `sanity_render_scores` summary (measured —
derived from telemetry). The measurement decided whether to promote the
training run; once promoted, the measurement freezes into a committed value
that any future reproducer can compare against.

---

## 6. Git vs runtime decision tree

A rubric for every new artefact the project produces.

1. **Would losing this break reproducibility from the committed set?**
   → **Commit it.**
   Example: references, manifests, Containerfiles, Cargo.lock.

2. **Can we rebuild this in under an hour of compute from the committed set?**
   → **Don't commit. Gitignore and let `.gitignore` do the work.**
   Example: ai-toolkit config files, output PNGs, cache entries, compiled
   Rust artefacts (`target/`), the Calmecac bundle.

3. **Is this an expensive one-time build artefact that *could* be shipped but
   doesn't have to be?**
   → **Don't commit. Ship a manifest of it and a script that rebuilds it.**
   Example: trained LoRA weights (150–300 MB each), container images (5–8 GB
   each), model checkpoints (17–45 GB each). The *binary* stays out; the
   *reproduction recipe* goes in.

4. **Is the binary too big to commit even if we wanted to?**
   → **Commit a manifest. The hash in the manifest is the commitment.**
   This is the LoRA case. A 300 MB safetensors is too big for a casual git
   push; its SHA-256 is 32 bytes. Commit the 32 bytes.

**Corollary rule.** If the manifest commits a hash and the committed inputs
do not reproduce that hash byte-for-byte, the committed state is broken and
must be fixed before the next merge. `tt-render verify` will catch the
simplest forms of this (manifest vs filesystem divergence); deeper forms
(committed references that no longer reproduce the hash) require a retrain
and a manifest update, which is itself a visible commit.

**Author trap to avoid.** Do not commit rendered panels, the `output/`
directory, or the `calmecac-bundle/`. They are regeneration targets. Gitignore
already protects you; a well-meaning `git add .` on a clean checkout does not
go wrong because the bundle and output live at the repo root under gitignored
paths.

---

## 7. Discussion interface

The author never touches pixels. The author touches concepts. The pipeline
translates.

### The loop

1. **Author** (concept level): *"TT 12/15, panel 3 needs less clutter."*
2. **Translator** (currently Claude; see §8 for the local-thinking-service
   successor): types a spec mutation. Options, in rough order of surgical
   precision:
   - Edit `strips/12-*/panels/panel-3.prompt.md` — reword the prompt to say
     fewer props, plainer background.
   - Tighten a VLM threshold — e.g. add a `palette.paper-bg` check with a
     stricter confidence floor.
   - Add a new check to `visual-qa-loop/spec.md` — e.g. `panel.prop-density`
     scored against a clutter heuristic.
   - Refine `character-canon/spec.md` if the clutter comes from prop
     proliferation that was never invariant.
   - Touch `style-bible/spec.md` if this is a global tone shift, not a
     per-panel concern.
3. **Commit the mutation.** Message carries `@trace spec:<name>` and
   `@Lesson S1-NNN`. The commit is a Lamport tick — monotonically later than
   every preceding commit; its hash is the tick value.
4. **Content-hash cascade.** `tt-specs` re-parses; `tt-hashing` recomputes
   `global_style_hash`, `character_lora_hash`, and every `panel_hash`; diff
   against `cache/panels/`; `CacheHit` / `CacheMiss` events flow; the
   re-render scope is whichever panels the cascade marked stale.
5. **Narrow re-render.** Usually one or two panels. Sometimes a whole strip.
   Rarely (style-bible mutation) the whole season.
6. **VLM convergence.** `tt-qa` critiques; drift score is the feedback
   signal; failing checks become the next iteration's prompt addendum. Max
   five rerolls on the fast VLM, then five on the heavier one, then
   `NeedsHuman` and exit 10.
7. **Author reviews in Calmecac.** The Comic view shows the new composited
   strip; clicking a plate drills into the Lesson or Rule view. The
   Convergence tab shows the shape-over-time of whichever metric moved.
8. **Done or another round.** The author's next concept-level critique loops
   the whole thing again.

### Calmecac hookups the author sees explicitly

- **Comic view (`/strip/NN`).** The final composited strip. Three plates are
  clickable; bottom-left drills into Lesson / Rule, bottom-right back to
  `www.`. Primary observability touchpoint after a render.
- **Rule view (`/rule/character-loras`).** Gallery of every rendered artefact
  that cites the LoRA contract — the manifest visualised, the panels named,
  the last training run's sanity scores. A visual rule, not boring.
- **Rule view (`/rule/visual-qa-loop`).** Boring dashboard. Body length over
  changes, citation count over changes, per-check pass rate over iterations,
  drift score distributions across panels. The author consults this when a
  reroll depth number feels wrong.
- **Convergence tab (per-rule timelines).** The shape-over-time for every
  `@trace spec:X` series. When a threshold is tightened, the tab shows
  exactly which panels rerolled and how drift scores moved.
- **Conversation view.** Commits-as-observability. The author's own
  conversation, filtered by `@Lesson S1-NNN` or `@trace spec:<name>`, with
  concept-level per-commit summaries. *"Argument trail"* under a lesson view
  shows the threaded evolution of the teaching. `www.tlatoani-tales.com`
  readers can't see this layer; `calmecac.tlatoani-tales.com` viewers can.
- **Strip view → "How this strip arrived at this state."** Lists every
  commit that affected this strip's panel_hash. Lesson spec edits, style
  bible edits, character canon refinements, LoRA manifest updates, the
  strip's own proposal edits. The commit chain that produced this pixel is
  one click away. This is the clearest operational expression of
  `@Lesson S1-1500` (proof-by-self-reference) Calmecac offers.

### Long-term

The translator is today Claude running on an external API. Tomorrow it is a
local thinking-service (ollama-hosted, or a Tillandsias-hosted offline model
— see §8). The shift is architecturally small: the trusted-zone crate that
*is* the translator becomes another HTTP client talking to localhost. Every
other invariant (content addressing, trust boundary, VLM QA) carries over.

The trusted-toolbox / untrusted-container boundary is specifically designed
to make this a drop-in swap. The translator is just another HTTP client in
the trusted zone.

---

## 8. Future: Tillandsias as runtime

Brief note; not load-bearing; flagged for curation.

`~/src/tillandsias/` orchestrates containerized dev environments invisibly.
Its enclave pattern is already the model for `isolation/spec.md`'s trust
zones. The future arc:

1. Today: Tlatoāni Tales runs inside an ad-hoc Fedora Silverblue
   `tlatoani-tales` toolbox with untrusted Podman containers around it.
2. Near term: the orchestrator invokes Tillandsias's enclave helpers for
   image builds and runtime wiring, instead of raw `podman` calls.
3. Long term: the whole pipeline runs inside a Tillandsias enclave. A single
   command spins up the trusted Rust workspace, the GPU-passthrough inference
   container, the trainer container, and the viewer container with a single
   audit-able flag surface.

**Branding teaser** (author's phrasing, flagged for author curation; *not
canonized*): *"Made by Tlatoāni"* as a Tillandsias framework label — a
Season 3+ candidate. The teaching would be about how the comic's production
pipeline becomes a product in its own right. This is a **lesson seed**, not a
canonized plan. It appears here so that when Season 3 authoring begins, the
substrate and the naming are already present in the repo.

---

## 9. Known gaps as of this commit

Honest inventory. These gaps are expected; the pipeline is designed to
tolerate them until they close.

| Gap | Current state | What unblocks |
|---|---|---|
| Container images built | **viewer** shipped (167 MB measured); **inference** rebuilding after upstream ollama URL move from `.tgz` to `.tar.zst` (see `images/inference/Containerfile` — `OLLAMA_VERSION=v0.21.1`); **trainer** building | First successful end-to-end `podman build` of inference + trainer images on the author's machine |
| Character reference sheets | **None** yet under `characters/tlatoani/references/` or `characters/covi/references/` — only `.gitkeep` placeholders | First pass can be LoRA-less FLUX renders of canon prompts, filtered through the reference-gate VLM. Then author curation replaces or supplements. |
| Trained LoRAs | **Not trained yet.** `characters/tlatoani/lora-manifest.json` carries all-zero hashes, no `lora_hash`, no `trained_at`, no `sanity_render_scores` | Reference corpora + trainer image + one `cargo run -p tt-lora-train` per character |
| `tt-render::process_strip` workflow-JSON generation | Being authored in **parallel wave 11** | Panel-cache wiring lands in parallel; then end-to-end render is unblocked |
| First end-to-end render | Blocked on every row above | After all of the above: one `tt-render --only 01` produces `output/Tlatoāni_Tales_01.png` + sibling `.json` + updated `calmecac-index.json` |
| `tt-lora::train` ai-toolkit orchestration end-to-end | `todo!("ai-toolkit orchestration lands with images/trainer/Containerfile")` in `crates/tt-lora/src/lib.rs` — config rendering, argv composition, manifest I/O, progress parsing all implemented and tested | Trainer image materialises; stdout-to-event wiring switches from unit-tested-in-isolation to integration-tested live |

The pipeline shape is fully specified; the last mile is mechanical.

---

## 10. References

Pointers into the specs for each lifecycle part.

| Lifecycle concern | Spec |
|---|---|
| LoRA training, corpus shape, manifest schema, versioning | `openspec/specs/character-loras/spec.md` |
| Character invariants (visual, behavioural) | `openspec/specs/character-canon/spec.md` |
| Trust zones, container hardening, `podman run` flag canon | `openspec/specs/isolation/spec.md` |
| Orchestrator CLI, event bus, render flow, exit codes | `openspec/specs/orchestrator/spec.md` |
| VLM critique, drift scoring, rerolls, thresholds | `openspec/specs/visual-qa-loop/spec.md` |
| Reader-facing observability viewer, boring dashboards, Conversation view | `openspec/specs/calmecac/spec.md` |
| Lesson registry (S1-100 … S1-1500), tombstones, citation forms | `openspec/specs/lessons/spec.md` |
| Per-file license rules (R01 … R18), REUSE path | `openspec/specs/licensing/spec.md` |
| Title / trace+lesson / episode plates, plate QA checks | `openspec/specs/trace-plate/spec.md` |
| Character corpus directory layout, TB03 ASCII rail | `characters/README.md`, `characters/tlatoani/README.md` |
| Rust crate that wraps the trainer | `crates/tt-lora/src/lib.rs` |
| Trainer image recipe | `images/trainer/Containerfile` |
| Inference image recipe | `images/inference/Containerfile` |
| Viewer image recipe | `images/viewer/Containerfile` |

---

## 11. Trace

@trace spec:character-loras, spec:orchestrator, spec:isolation, spec:visual-qa-loop, spec:calmecac, spec:licensing, spec:trace-plate

@Lesson S1-500 *(edits-that-reconcile — the LoRA hash arriving in the panel cache key is the literal reconciliation path)*
@Lesson S1-1000 *(dashboards-must-add-observability — the convergent/measured split is the boring-dashboard's subject)*
@Lesson S1-1500 *(proof-by-self-reference — the trainer is across the trust boundary yet the hash is computed by trusted code after the bytes are sealed)*
