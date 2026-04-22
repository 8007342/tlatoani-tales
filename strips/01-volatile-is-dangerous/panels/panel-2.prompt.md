---
panel: 2
seed: 271828
base_model: flux1-schnell-fp8
character_loras:
  - tlatoani-v1
  - covi-v1
qwen_for_text: false
---

# Panel 2 prompt

## Positive prompt

Warm-paper newspaper comic panel, soft warm tones, restrained palette.
Background: warm paper tone (#F4E9D3), subtle aged grain. Palette remains
restrained — chaos is in the *motion*, not in the colours.
Simple, adorable, modern cartoon linework; soft shadows; clear silhouettes;
newspaper-strip nostalgia with modern polish.

Centre of panel: the cardboard box from panel 1 has **burst**. Lid blown off
upward; colourful pastel sticky-notes exploding outward in a radial burst —
the "explosion of notes" symbol from the symbol dictionary, volatile memory
failure made literal. Tags visible on flying notes: "Prompt v17", "Rules",
"User intent", "Few-shot", "Tool info", "Memory", "Example". Small cartoon
"BOING!" marks, spark lines, and motion streaks punctuate the burst.
A couple of coral-accent (#D96C5B) notes in the burst — the alarm colour
now at full volume.

Left-centre: CoviFigure, mid-panel, caught in the moment — hands thrown up,
mouth wide open mid-shout, eyes squeezed shut. Body posture theatrical and
animated, arms spread. White ambiguous rounded figure, minimal facial
features, simple shirt. Body language reads *playful panic*, never tragic.
Covi still has no named face, no hair, no gender markers — remains the
reader's proxy.

Right-centre: TlhAxolotlSage, small axolotl sage, single tail only, wise
half-lidded eyes, gold crown with subtle Aztec geometry, blue-green tunic
with Aztec motif. Holds a **fully open** umbrella (dark canopy with a
subtle geometric trim) over both Covi and themself. Sticky-notes bounce off
the umbrella's canopy with small impact marks. Tlatoāni's expression is
unchanged from panel 1 — same calm, same half-lidded eyes, same serene
posture. The open umbrella anchors the composition — a dome of order inside
an otherwise chaotic frame.

Ground: first accumulation of debris — a few sticky-notes settling around
their feet. Broken box tilted to stage-left.

Composition: Covi left-of-centre, Tlatoāni right-of-centre, umbrella canopy
arches over both, burst fills the upper portion of the frame. Top-left
corner kept relatively clear; burst debris stays below it to preserve the
title-plate region.

Focal action: the burst — volatile failure mode rendered symbolically;
Tlatoāni's umbrella deflecting the fallout.

## Negative prompt

double tail, second tail, two tails, forked tail, twin tails, gills-as-tails,
external gill tufts rendered as tails, named face on Covi, Covi with hair,
Covi with gender markers, Covi crying, Covi sad, Covi tearful, Covi
despairing, Covi defeated, Tlatoāni panicked, Tlatoāni running, Tlatoāni
smiling broadly, Tlatoāni laughing, umbrella closed in panel 2, umbrella
missing, umbrella inverted-by-wind, Mary-Poppins-style floating, harsh
rim lighting, photo-realism, 3D render, octane render, blender render,
hyperreal, sheet-crowded background, busy wallpaper, garish saturation,
neon colours, dark mode, black background, night scene, thunderstorm,
rain, real lightning, real fire, real explosion effect, corporate logos,
watermark, signature, artist tag, misspellings in sticky-note text,
speech bubble for Tlatoāni (Tlatoāni does not speak in panel 2), dialogue
text in panel 2 (Covi's shout is added by compose layer), bottom-left
scroll plate (composited separately), bottom-right scroll plate
(composited separately), title composited inside the raw panel (title is
composited separately).

## Expected checks (from visual-qa-loop)

- `tlatoāni.single-tail` — pass
- `tlatoāni.crown-present` — pass
- `tlatoāni.serene-expression` — pass (unchanged from panel 1)
- `covi.ambiguous-white` — pass
- `covi.no-named-face` — pass
- `covi.good-mood` — pass (posture should read *playful panic*, not
  despair — the hardest check in this strip)
- `palette.paper-bg` — pass
- `symbol.explosion-of-notes-present` — pass
- `symbol.umbrella-open-deflecting-debris` — pass
- `composition.umbrella-canopy-anchors-frame` — pass
- `composition.title-region-clear` — pass (burst debris kept below
  top-left region)
- `plate.*` — not checked at the panel stage; composited separately.
