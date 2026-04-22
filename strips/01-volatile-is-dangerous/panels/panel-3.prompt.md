---
panel: 3
seed: 161803
base_model: flux1-schnell-fp8
character_loras:
  - tlatoani-v1
  - covi-v1
qwen_for_text: false
---

# Panel 3 prompt

## Positive prompt

Warm-paper newspaper comic panel, soft warm tones, restrained palette.
Background: warm paper tone (#F4E9D3), subtle aged grain. Quiet aftermath —
palette slightly calmer than panel 2, same base tones.
Simple, adorable, modern cartoon linework; soft shadows; clear silhouettes;
newspaper-strip nostalgia with modern polish.

Ground across the full width: pastel sticky-notes strewn at rest, a
gentle settle rather than a burst. Note tags visible at a glance:
"Prompt v17", "Rules", "Memory", "Example". The ruined cardboard box
lies tipped on its side to stage-left, lid nearby, partially empty. The
storm is over.

Centre-left: CoviFigure, standing, hands at their sides, shoulders
slightly inward. A soft pink blush on Covi's cheeks — the *realization*
moment. Eyes half-closed in sheepish acknowledgement, small sheepish
curve of mouth. Body language reads *slightly wiser* and *humbled in good
humour*, never defeated. White ambiguous rounded figure, minimal facial
features, simple shirt; no named face, no hair, no gender markers.

Centre-right: TlhAxolotlSage, small axolotl sage, single tail only, wise
half-lidded eyes, gold crown with subtle Aztec geometry, blue-green tunic
with Aztec motif. Tlatoāni is **closing the umbrella** (motion frozen at
near-closed, canopy almost fully collapsed) with the same serene
expression carried from panels 1 and 2. Posture relaxed, unhurried.

Above Tlatoāni, a soft speech bubble with the line "that was committed,
right?" rendered cleanly — short, calm, unhurried typography. The bubble
is small and the text fits comfortably; no jargon.

Composition: Covi left, Tlatoāni right, facing each other at three-quarter
angles. Bottom-right corner reserved for the episode plate (composited
later) — keep strewn notes sparser there. Top-left corner deliberately
kept clear for the title plate (composited later — although the title
runs over panel 1, not panel 3).

Focal action: Tlatoāni's wisdom line — the single calm observation that
retroactively names the whole strip's teaching. Covi's blush is the
reader's cue that the lesson landed.

## Negative prompt

double tail, second tail, two tails, forked tail, twin tails, gills-as-tails,
external gill tufts rendered as tails, named face on Covi, Covi with hair,
Covi with gender markers, Covi crying, Covi tearful, Covi despairing,
Covi defeated, Covi grinning smugly, Tlatoāni smug, Tlatoāni lecturing,
Tlatoāni wagging a finger, Tlatoāni pointing aggressively, Tlatoāni
laughing, Tlatoāni smiling broadly, umbrella open in panel 3, umbrella
missing, new burst of notes, active motion lines, spark marks, active
BOING marks, harsh rim lighting, photo-realism, 3D render, octane render,
blender render, hyperreal, sheet-crowded background, busy wallpaper,
garish saturation, neon colours, dark mode, black background, night
scene, corporate logos, watermark, signature, artist tag, misspellings
in speech bubble text, speech bubble for Covi (Covi does not speak in
panel 3), multiple speech bubbles (only Tlatoāni's one bubble),
bottom-right scroll plate rendered inside the raw panel (composited
separately), bottom-left scroll plate rendered inside the raw panel
(composited separately), title composited inside the raw panel (title is
composited separately over panel 1's top-left).

## Expected checks (from visual-qa-loop)

- `tlatoāni.single-tail` — pass
- `tlatoāni.crown-present` — pass
- `tlatoāni.serene-expression` — pass (no smugness, no lecturing)
- `tlatoāni.wisdom-line-in-bubble` — pass (single bubble, short text,
  reads "that was committed, right?")
- `covi.ambiguous-white` — pass
- `covi.no-named-face` — pass
- `covi.blush-present` — pass (the realization cue)
- `covi.good-mood` — pass (sheepish, never defeated — the arc ends
  *slightly wiser, not sadder*)
- `palette.paper-bg` — pass
- `symbol.notes-strewn-at-rest` — pass (settled, not bursting)
- `symbol.umbrella-closing` — pass (motion frozen at near-closed)
- `composition.episode-plate-region-clear` — pass (bottom-right kept
  sparse for the episode plate composite)
- `plate.*` — not checked at the panel stage; composited separately.
