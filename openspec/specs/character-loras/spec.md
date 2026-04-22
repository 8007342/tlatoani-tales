# Character LoRAs

## Purpose

Per-character LoRAs enforce visual identity across the whole series. FLUX.1-schnell alone cannot hold Tlatoāni's silhouette stable over N strips — the **double-tail failure mode** is the canonical example: the base model drifts toward two tails because axolotls in its training data usually have two external gill tufts that read as tails. Every strip rendered without the LoRA fights this drift by prompt alone, and loses.

The LoRAs are the weights side of the contract. `character-canon/spec.md` is the prose side. Changing either invalidates every cached panel containing that character — the LoRA hash is part of the panel cache key (see ME09).

This spec governs how those LoRAs are trained, versioned, audited, and consumed.

## Trust boundary

Training is **Python, over the trust boundary**. Our own code stays in Rust; the Python trainer (`ai-toolkit`) runs inside a hardened disposable container that cannot reach the trusted toolbox, the network, or anything outside explicit bind mounts. See `isolation/spec.md` for the full zone model.

- **Container name:** `tlatoani-tales-trainer` — ASCII-only, deliberately. See TB03: the gap between the comic's `Tlatoāni` (macron) and the container's `tlatoani` is the teachable break — trust zones are paranoid about encoding, and our container names ride the lowest-common-denominator rail.
- **Image:** hardened tiny Fedora-minimal or Alpine base; Containerfile at `images/trainer/Containerfile`. Pinned Python, ai-toolkit git SHA, CUDA runtime. Nothing else.
- **Security flags (non-negotiable):** `--cap-drop=ALL --security-opt=no-new-privileges --userns=keep-id --network=none`. Training is fully offline; the base model and references are bind-mounted before the container starts, and nothing reaches out after.
- **Bind mounts:**
  - `characters/<name>/references/` → read-only input
  - `tools/ComfyUI/models/checkpoints/flux1-schnell-fp8.safetensors` → read-only base model
  - a dedicated output volume that surfaces the trained weights at `tools/loras/<name>-v<N>.safetensors` → write
- **The VLM reference gate runs across the same boundary.** The pre-training canon check uses the ollama VLM described in `visual-qa-loop/spec.md`; ollama is likewise across the trust boundary. Neither the trainer nor the VLM ever runs inside the trusted toolbox.

This boundary is a Season 2 **meta-example candidate** (flag for author curation — do not canonize): *LoRA training runs fully offline and sandboxed — proof of the trust boundary in action, and of the project's captioned claim that every strip was rendered with offline models.*

## Rust wrapper — `tt-lora`

`tt-lora` is the trusted-zone Rust crate that *is* the LoRA subsystem as far as the rest of the workspace is concerned. Nothing else in our code talks to ai-toolkit directly.

Following the `tt-*` crate convention (see `orchestrator/spec.md`), `tt-lora` is a **library crate**; a small binary `tt-lora-train` exposes it on the CLI, and `tt-render` depends on the library to manage LoRA lifecycle inline during a render wave.

Responsibilities:

- **Container lifecycle.** Start the `tlatoani-tales-trainer` container (idempotent — reuse if already running), verify bind mounts, tear down (or keep warm — see open question below) on completion.
- **Subprocess driver.** Launch ai-toolkit inside the container with a rendered config file (hyperparams from the manifest), capture stdout + stderr, parse ai-toolkit's progress lines into typed events.
- **Event emission** on the shared `tt-events` event bus. Typed `LoraEvent` variants:
  - `LoraEvent::TrainStarted { character, version, config_hash }`
  - `LoraEvent::StepProgress { character, step, total_steps, loss }`
  - `LoraEvent::SanityRenderDone { character, prompt, drift_score }`
  - `LoraEvent::Trained { character, manifest }`
  - `LoraEvent::Failed { character, reason }`
  Subscribers include the CLI progress renderer, Calmecac live-watch, and telemetry sinks. Nothing polls; everything observes.
- **Manifest I/O.** Read and write the character's `lora-manifest.json` (schema unchanged — see below).
- **LoRA hash.** Compute the SHA-256 of the trained `.safetensors` and fold it into the manifest's `output.sha256`. This is the value the panel cache keys on (see ME09 and the cache-key section below).
- **Error taxonomy.** `thiserror`-based; distinguishes infra (container failed to start, OOM, CUDA missing — exit 30 at the orchestrator layer) from canon (drift too high after iteration budget — exit 10).

Anything Python lives on the far side of the subprocess boundary; `tt-lora`'s public API is pure Rust.

## Invariants

- **One LoRA per character.** Tlatoāni and Covi each have their own. No combined "cast" LoRA — keeps failure modes isolated and retraining cheap.
- **Trigger token is stable.** Once published, a trigger token never changes meaning. New identity = new token + new LoRA name.
- **Reference sheets are committed art.** Under `characters/<name>/references/`. Covered by licensing R05 (CC BY-SA 4.0). The LoRA is reproducible from them.
- **Trained weights are NOT committed.** Under `tools/loras/` (gitignored). They're build artefacts. The manifest is committed; the weights regenerate.
- **Manifest is the source of truth.** If the manifest says LoRA hash `abc123…`, any renderer that produces a different hash for the same inputs is broken.
- **Version bumps are explicit.** v1 → v2 is a human decision, committed, visible. No silent retrains.

## Reference sheet corpus

Per character, living under `characters/<name>/references/`:

| Axis | Target | Notes |
|---|---|---|
| Image count | 24–40 | Below 24 = under-fit; above 40 = slow training with no quality gain for a character this simple |
| Resolution | 1024×1024 native | FLUX trains at 1024; downscaling on load is fine, upscaling is not |
| Pose variety | ≥ 8 distinct poses | Standing, sitting, walking, reaching, turning, looking up, back-turn, three-quarter |
| Expression variety | ≥ 5 | Calm (Tlatoāni default), thinking, surprised, blushing (Covi), half-lidded wise |
| Prop variety (Tlatoāni) | cover all canon props | umbrella, notebook, compass, lantern, ruler, hourglass, scroll — at least once each |
| Background | plain / paper / soft blur | Never sheet-crowded backgrounds; they contaminate the LoRA |
| Lighting | consistent with style bible | Warm, soft, no hard rim lighting |

File naming: `<name>-<pose>-<expression>-<NN>.png` (e.g. `tlatoāni-standing-calm-03.png`). Numeric suffix so the trainer doesn't collapse near-duplicates by name.

### Reference gate

Before training: every reference image is run through the visual-QA VLM against `character-canon/spec.md`. A reference image that *itself* fails canon (a double-tail, a Covi with a named face) is rejected. Garbage in, garbage out — the LoRA is only as disciplined as its corpus.

## Training pipeline

### Base model

- `flux1-schnell-fp8.safetensors` at `tools/ComfyUI/models/checkpoints/`. Pinned by SHA-256 in the manifest.

### Trainer

**`ai-toolkit` (ostris)**, invoked as a subprocess inside `tlatoani-tales-trainer` by the `tt-lora` Rust wrapper. Chosen because:

- First-class FLUX.1 support, including schnell variants.
- Runs in ~10–20 GB VRAM at rank 16 — comfortably fits the A5000's 24 GB.
- Config-file driven → reproducible; hyperparams live in the manifest.
- Maintained, community-tested on exactly this base model.

Alternative considered: `fluxgym` (nicer UI, same underlying kohya-trainer). Rejected because we want headless reproducibility, not a Gradio panel. If the author later wants a UI for authoring refs, fluxgym can read the same dataset — no lock-in.

Our code never imports ai-toolkit. The config file is rendered by `tt-lora` from the manifest's `hyperparams` block, dropped into the read-only bind mount, and consumed by the Python trainer. Its stdout is the only channel back.

### Hyperparameters (baseline — v1)

| Parameter | Value | Rationale |
|---|---|---|
| LoRA rank | 16 | Enough capacity for a stylized character; avoids over-fitting |
| LoRA alpha | 16 | `alpha == rank` is the ai-toolkit recommended default for FLUX |
| Network type | LoRA (not LoKR / LoHa) | Maximum toolchain compatibility |
| Steps | 2000–3000 | Small dataset; converges fast. Monitor sanity renders at 1000 / 1500 / 2000 |
| Learning rate | 1e-4 | ai-toolkit default for FLUX character LoRAs |
| Batch size | 1 | FLUX is VRAM-heavy even at fp8; B=1 is safest on 24 GB |
| Optimizer | AdamW8bit | Memory-efficient, well-tested |
| Resolution | 1024 | Matches reference corpus |
| Mixed precision | bf16 | Standard for FLUX training |
| Caption dropout | 0.05 | Small amount; keeps token association strong without memorizing |

Expected wall time on RTX A5000 (24 GB): **~45–90 minutes per character at 2000 steps.** First run will be slower (caching text encoder outputs, etc.).

### Trigger tokens

Form: **PascalCase compound, rare enough to not collide with natural English.**

| Character | Trigger | Why |
|---|---|---|
| Tlatoāni | `TlhAxolotlSage` | `Tlh` prefix is not a common English bigram; `AxolotlSage` anchors species + role. ASCII-only (prompt-safe). |
| Covi | `CoviFigure` | `Covi` is the character's actual slug; `Figure` keeps it species-ambiguous per canon. |

Trigger tokens appear in every training caption and every inference prompt. They're the handle the LoRA hangs on. Changing them = new LoRA = version bump.

### Caption template

Per reference image, a `.txt` sidecar:

```
TlhAxolotlSage, <pose>, <expression>, <prop-if-any>, warm paper background, cartoon linework
```

Minimal, structured, trigger token first. No over-description — the LoRA should learn *what stays the same across all images* (the character), and captions hint only at *what varies*.

## Artefacts

### Trained weights

`tools/loras/<name>-v<N>.safetensors` — gitignored. Examples:

- `tools/loras/tlatoāni-v1.safetensors`
- `tools/loras/covi-v1.safetensors`

### Manifest (committed)

`characters/<name>/lora-manifest.json` — the reproducibility contract:

```jsonc
{
  "character":         "tlatoāni",
  "version":           1,
  "trigger_token":     "TlhAxolotlSage",
  "base_model": {
    "name":            "flux1-schnell-fp8",
    "sha256":          "…"
  },
  "dataset": {
    "reference_dir":   "characters/tlatoāni/references/",
    "image_count":     32,
    "dataset_hash":    "sha256:…"
  },
  "hyperparams": {
    "rank":            16,
    "alpha":           16,
    "steps":           2000,
    "lr":              1e-4,
    "batch_size":      1,
    "optimizer":       "AdamW8bit",
    "resolution":      1024,
    "precision":       "bf16",
    "caption_dropout": 0.05
  },
  "output": {
    "path":            "tools/loras/tlatoāni-v1.safetensors",
    "sha256":          "…",
    "size_bytes":      …
  },
  "sanity": {
    "prompts":         ["TlhAxolotlSage holding umbrella, paper background"],
    "drift_scores":    [0.03],
    "verdict":         "promoted"
  },
  "trained_at":        "2026-05-01T12:00:00Z",
  "trainer":           "ai-toolkit@<git-sha>"
}
```

`dataset_hash` = SHA-256 over the sorted list of (filename, content-hash) pairs. Changes if any reference is added, removed, or modified.

## Cache key integration

The panel cache (planned: `panel-cache/spec.md`; see ME05) keys each rendered panel on:

```
panel_hash = sha256(
  style_bible_rev,
  character_canon_rev,
  lora_hash(tlatoāni-v<N>),       // <-- from lora-manifest.json
  lora_hash(covi-v<N>),           //     same
  base_model_hash,
  prompt,
  seed,
  sampler_params
)
```

Retraining a LoRA → new `output.sha256` → new `panel_hash` → every cached panel that named that character is silently invalidated on next render. No manual bookkeeping.

This is **ME09** made operational. Cite ME09 in any strip that leans on this mechanic. `@Lesson S1-500` — the edit (a retrained LoRA) reconciles through the cache, not through a human remembering which files to delete. The LoRA hash arriving in the cache key via `tt-lora`'s manifest write is the literal reconciliation path.

The fact that the trainer is across a trust boundary yet still produces a hash our trusted cache can key on is itself `@Lesson S1-1500` (proof-by-self-reference): the untrusted zone cannot lie about its output, because the hash is computed by `tt-lora` in the trusted zone after the bind-mounted file is sealed.

## Workflow

```
  author curates references
         │
         ▼
  VLM reference gate (ollama, across the trust boundary)
  — rejects any reference that itself fails character-canon
         │
         ▼
  tt-lora: start tlatoani-tales-trainer container (idempotent;
           bind mounts ro refs + ro base model + rw output; --network=none)
         │
         ▼
  tt-lora: render ai-toolkit config from manifest hyperparams,
           launch subprocess inside container,
           parse stdout → LoraEvent::StepProgress on tt-events bus
         │
         ▼
  sanity renders (fixed prompts, fixed seeds) → LoraEvent::SanityRenderDone
         │
         ▼
  visual-qa-loop scores sanity renders
         │
  ┌──────┴──────┐
  │             │
  ▼             ▼
drift low    drift high
 promote      iterate (refs? hyperparams? refs first, then rank)
  │
  ▼
  tt-lora: compute output.sha256, write lora-manifest.json,
           emit LoraEvent::Trained { manifest },
           stop container (default) OR keep warm (see open question)
         │
         ▼
commit manifest — new panel_hash for every affected panel
```

Sanity prompts are defined per character in the manifest under `sanity.prompts`. Fixed — they do not change across versions, so v1 / v2 / v3 produce comparable drift scores on the same reference renderings.

## Versioning

- `v1` is the first published LoRA that cleared the sanity gate.
- `v2` is minted when: references change meaningfully, hyperparams change, or canon changes in a way that requires re-training.
- **Strip proposals MAY pin a specific LoRA version** in their frontmatter (`lora_versions: { tlatoāni: 1, covi: 1 }`). Omitted = latest. Pinning freezes reproducibility for archival strips.
- Promoting `v2` does not retroactively invalidate strips that pinned `v1`. Pinning is a CRDT: a strip's pinned version is a stable cell, and the cache respects it.

## Out of scope

- Training LoRAs for props or backgrounds — props are covered by the prompt + symbol-dictionary; backgrounds are covered by the style bible.
- Training the VLM. Visual-QA uses pretrained weights (see `visual-qa-loop/spec.md`).
- Distributing LoRAs publicly. They're build artefacts; if a reader wants them they train their own from the committed references. This is the reproducibility promise.

## Open questions

- **Keep-warm vs stop-per-character.** Default is stop after each character trains — clean, disposable, matches the trust-boundary aesthetic. But Tlatoāni + Covi back-to-back (or iterative retraining during authoring) pays the container-start cost twice per cycle. Options:
  - *Stop-per-character* (default) — paranoid, slow on iteration.
  - *Keep-warm for the whole training session* — `tt-lora` leaves the container running until the training binary exits or a TTL elapses.
  - *Pool* — pre-warm one container at startup of any `tt-render` invocation that might retrain.
  Author preference pending. The trust properties are identical in all three (the container is still `--network=none` with dropped caps); the tradeoff is purely latency vs clean-slate paranoia.
- **Meta-example curation.** The trust-boundary-as-LoRA-pipeline framing is flagged above as a Season 2 candidate. Not canonized until the author curates.

## Trace

`@trace spec:character-loras, spec:character-canon, spec:visual-qa-loop, spec:meta-examples, spec:orchestrator, spec:isolation`
`@Lesson S1-500`
`@Lesson S1-1500`
