# Covi — reference corpus

The learner. The reader's proxy. Reference sheets for Covi's LoRA live here.

> **Spelling note.** Covi is ASCII by design — both the character's slug
> (`covi`) and the in-strip representation (no name shown in-strip at all;
> Covi is the author/pipeline handle only). The directory and manifest field
> `"covi"` match the ASCII slug required by Podman's LDH grammar and
> `CharacterName` in `crates/tt-lora/src/lib.rs`. See **TB03** for the
> broader convention.

## Current state

**Corpus empty — awaiting author-curated sheets.**

The `references/` directory contains only a `.gitkeep` placeholder. First-pass
population can be either:

1. Author-curated art.
2. A first-pass LoRA-less FLUX render of the canon description, filtered
   through the reference-gate VLM check. Rejections (named Covi, Covi
   clearly disambiguated as human-or-AI, over-detailed face) never enter
   training.

Until the corpus lands, `lora-manifest.json` carries placeholder zero-hashes
and no `lora_hash` / `sanity_render_scores` / `trained_at` fields.

## Invariants (character-canon, not negotiable)

From `openspec/specs/character-canon/spec.md` §Covi — every reference image
must honour these:

- **White ambiguous figure — human OR AI, never disambiguated.** This is
  Covi's defining invariant. The reference gate rejects any image that
  reads unambiguously as "human" or unambiguously as "robot/android".
- **No name shown in-strip.** Covi is the reader's proxy. References must
  not include text, name tags, or overt identity markers.
- Simple rounded design, minimal facial features.
- Simple shirt and casual clothes.
- Expressive posture — the whole emotional arc rides the body.
- Sometimes blushes when realizing a mistake.
- Always humble, always learning, always in a good mood.

## Reference corpus shape

From `openspec/specs/character-loras/spec.md` §Reference sheet corpus:

| Axis                | Target                                          |
|---------------------|-------------------------------------------------|
| Image count         | **24–40** PNG files                             |
| Resolution          | **1024×1024 native**                            |
| Pose variety        | ≥ 8 distinct poses — lean into posture expressiveness (shrug, reach, head-tilt, recoil, back-turn, leaning-in, walking-away, pointing) |
| Expression variety  | ≥ 5 — the Covi-specific spec lists **blushing** as a required mode |
| Props               | Minimal. Covi is *carried* by props in-strip, not defined by them. |
| Background          | Plain / paper / soft blur                       |
| Lighting            | Warm, soft, no hard rim lighting                |

### File naming

```
covi-<pose>-<expression>-NN.png
```

Examples:

- `covi-standing-calm-01.png`
- `covi-shrug-thinking-04.png`
- `covi-headTilt-blushing-02.png`
- `covi-backturn-humble-06.png`

## Trigger token

```
CoviFigure
```

PascalCase, ASCII-only. `Covi` is the character's actual slug; `Figure`
deliberately keeps the species/identity ambiguous per canon — the LoRA
should learn *a figure named Covi*, not *a human named Covi* nor *an AI
named Covi*. See `character-loras/spec.md` §Trigger tokens.

## What the LoRA training pipeline expects

Same pipeline as Tlatoāni — see `characters/tlatoani/README.md` for the
full walkthrough. Summary:

1. VLM reference gate per-image against `character-canon/spec.md` §Covi.
2. `ai-toolkit` inside `tlatoani-tales-trainer` (hardened, offline,
   `--network=none`).
3. Baseline v1 hyperparams in `lora-manifest.json::hyperparams` (rank 16,
   alpha 16, 2500 steps, LR 1e-4, batch 1, AdamW8bit).
4. Output `tools/loras/covi-v<N>.safetensors` (gitignored), SHA-256 sealed
   by `tt-lora` in the trusted zone.
5. That hash folds into every panel's cache key — retraining invalidates
   every Covi-containing panel mechanically (ME09, `@Lesson S1-500`).

## Pipeline command (once the corpus is populated)

```bash
cargo run -p tt-lora-train -- \
    --character covi \
    --manifest characters/covi/lora-manifest.json
```

@trace spec:character-canon, spec:character-loras, spec:visual-qa-loop, spec:isolation
@Lesson S1-500
@Lesson S1-1500
