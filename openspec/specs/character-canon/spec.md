# Character Canon

## Purpose

Fixes the visual and behavioral identity of recurring characters. Changes here cascade through every strip — mutation propagates via the cache layer.

## Tlatoāni — the wise guide

### Visual (invariant)

- Small axolotl sage.
- **Single tail only.** Never double tail. Common generator failure mode.
- Calm expression, wise half-lidded eyes.
- Sleek crown with subtle Aztec geometric references.
- Tunic/shirt with Aztec geometric motif, blue-green.
- Simple sandals optional.
- Carries wisdom through posture more than motion.

### Behavior

- Never raises voice, never panics.
- Speaks in short profound lines. Often one sentence. Sometimes silence is the punchline.
- Delivers the lesson in panel 3 as a simple observation, not a lecture.
- Posture does the teaching as much as the dialogue.

### Props (symbolic, used sparingly)

- Umbrella — protection from nonsense / chaos
- Notebook — durable truth / committed memory
- Compass — direction / convergence
- Lantern — clarity
- Ruler — metrics
- Hourglass — ordering / Lamport time *(new in TT #04)*
- Scroll — spec / contract *(new in TT #05)*

## Covi — the learner

### Visual (invariant)

- White ambiguous figure (human OR AI — never disambiguated).
- Simple rounded design, minimal facial features.
- Simple shirt and casual clothes.
- Expressive posture conveys the whole emotional arc.
- Sometimes blushes when realizing a mistake.
- **No name shown in-strip.** Reader's proxy.

### Behavior (invariant)

- Always humble.
- Always learning — ends every strip slightly wiser than they started.
- Always in a good mood — even the initial stumble is played with good humor, never despair.
- Represents: the reader, an AI user, an AI agent itself, an apprentice. Ambiguous by design.

## Character consistency enforcement

- Each character has a LoRA trained on curated reference sheets.
- LoRA hash is part of panel cache key — changing the LoRA invalidates every cached panel containing that character.
- Reference sheets live in `characters/<name>/references/` (committed to git).
- Trained LoRA weights live in `tools/loras/` (gitignored — reproducible from references).

## Trace

`@trace spec:character-canon` — every panel prompt referencing either character.
