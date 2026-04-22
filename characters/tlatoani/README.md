# Tlatoāni — reference corpus

The wise-guide axolotl sage. This directory holds everything the LoRA
training pipeline needs to reproduce Tlatoāni's visual identity.

> **A note on spelling.** The character's name is **Tlatoāni** (with macron)
> in all prose. The directory is `tlatoani/` (ASCII) because it is also the
> `character` field in `lora-manifest.json`, the stem of
> `tools/loras/tlatoani-v<N>.safetensors`, and the argument passed to the
> trainer container — all of which ride Podman's LDH grammar and
> `CharacterName`'s ASCII-kebab-case validator
> (`crates/tt-lora/src/lib.rs`). See **TB03** in
> `openspec/specs/character-loras/spec.md` §Trust boundary.

## Current state

**Corpus empty — awaiting author-curated sheets.**

The `references/` directory contains only a `.gitkeep` placeholder. First-pass
population can be either:

1. Author-curated art (preferred — the LoRA then inherits an author
   signature, not a model signature).
2. A first-pass LoRA-less FLUX render of the canon description, filtered
   through the reference-gate VLM check
   (`openspec/specs/visual-qa-loop/spec.md` and
   `openspec/specs/character-loras/spec.md` §Reference gate). Any image that
   itself fails canon (double tail, wrong crown, wrong tunic motif) is
   rejected before it can contaminate training.

Until the corpus lands, `lora-manifest.json` carries placeholder zero-hashes
and no `lora_hash` / `sanity_render_scores` / `trained_at` fields (the
optional sealing fields in `LoraManifest`, populated by `tt-lora` after a
successful train).

## Invariants (character-canon, not negotiable)

From `openspec/specs/character-canon/spec.md` §Tlatoāni — every reference
image must honour these, or the reference gate rejects it:

- **Single tail only.** Never double tail. This is the canonical FLUX failure
  mode the LoRA exists to defeat (see
  `openspec/specs/character-loras/spec.md` §Purpose).
- Sleek crown with subtle Aztec geometric references.
- Tunic/shirt with Aztec geometric motif, **blue-green**.
- Calm expression, wise half-lidded eyes.
- Small axolotl sage — wisdom carried by posture more than motion.
- Simple sandals optional.

## Reference corpus shape

From `openspec/specs/character-loras/spec.md` §Reference sheet corpus:

| Axis                | Target                                          |
|---------------------|-------------------------------------------------|
| Image count         | **24–40** PNG files                             |
| Resolution          | **1024×1024 native** (FLUX trains at 1024)      |
| Pose variety        | ≥ 8 distinct poses (standing, sitting, walking, reaching, turning, looking up, back-turn, three-quarter) |
| Expression variety  | ≥ 5 (calm default, thinking, surprised, half-lidded wise, …) |
| Prop coverage       | Every canon prop at least once: umbrella, notebook, compass, lantern, ruler, hourglass, scroll |
| Background          | Plain / paper / soft blur — never sheet-crowded |
| Lighting            | Warm, soft, no hard rim lighting                |

### File naming

```
tlatoani-<pose>-<expression>-NN.png
```

Numeric two-digit suffix so the trainer doesn't collapse near-duplicates by
name. Examples:

- `tlatoani-standing-calm-03.png`
- `tlatoani-sitting-thinking-07.png`
- `tlatoani-walking-halflidded-12.png`
- `tlatoani-holding-umbrella-calm-01.png`

Per-image caption sidecars (`.txt`, one line) are generated at training time
by `tt-lora` from the filename — see the caption template in
`character-loras/spec.md` §Caption template. **Do not pre-commit captions**;
they are trainer inputs, not reference art.

## Trigger token

```
TlhAxolotlSage
```

PascalCase, ASCII-only, prompt-safe. The `Tlh` prefix is a rare English
bigram (won't collide with natural prompts); `AxolotlSage` anchors species
+ role. Appears in every training caption and every inference prompt
invoking Tlatoāni. Trigger tokens are a stable published identity — a
token change = new LoRA = version bump. See
`character-loras/spec.md` §Trigger tokens.

## What the LoRA training pipeline expects

Governing spec: `openspec/specs/character-loras/spec.md`. In short:

1. **Reference gate.** Every PNG here is run through the visual-QA VLM
   against `character-canon/spec.md` §Tlatoāni. Rejections never enter
   training.
2. **Trainer.** `ai-toolkit` inside the `tlatoani-tales-trainer` container —
   `--cap-drop=ALL`, `--security-opt=no-new-privileges`, `--userns=keep-id`,
   `--network=none`. Fully offline.
3. **Hyperparams.** Baseline v1 in `lora-manifest.json::hyperparams`: rank
   16, alpha 16, 2500 steps, LR 1e-4, batch size 1, AdamW8bit.
4. **Output.** `tools/loras/tlatoani-v<N>.safetensors` (gitignored). SHA-256
   is computed by `tt-lora` in the trusted zone after the container exits
   and sealed into `lora-manifest.json::lora_hash` — the untrusted trainer
   cannot lie about the bytes it produced (`@Lesson S1-1500`).
5. **Cache integration.** That `lora_hash` folds into every panel's cache
   key (`panel-cache/spec.md`, ME09). Retraining invalidates every cached
   panel containing Tlatoāni, mechanically — no human bookkeeping
   (`@Lesson S1-500`).

## Pipeline command (once the corpus is populated)

```bash
# From the workspace root, inside the trusted toolbox:
cargo run -p tt-lora-train -- \
    --character tlatoani \
    --manifest characters/tlatoani/lora-manifest.json
```

@trace spec:character-canon, spec:character-loras, spec:visual-qa-loop, spec:isolation
@Lesson S1-500
@Lesson S1-1500
