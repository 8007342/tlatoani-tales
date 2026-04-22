---
panel: 1
seed: 314159
base_model: flux1-schnell-fp8
character_loras:
  - tlatoani-v1
  - covi-v1
qwen_for_text: false
---

# Panel 1 prompt

## Positive prompt

Warm-paper newspaper comic panel, soft warm tones, restrained palette.
Background: warm paper tone (#F4E9D3), subtle aged grain, no clutter.
Simple, adorable, modern cartoon linework; soft shadows; clear silhouettes;
slight newspaper-strip nostalgia with modern polish.

In the foreground, three-quarter view, CoviFigure — a small white
ambiguous rounded figure with minimal facial features, simple shirt and
casual clothes. Covi crouches over a cardboard box labelled "CONTEXT (105%)"
in hand-lettered stencil. Covi presses a pile of colourful pastel
sticky-notes downward with both hands, lid refusing to close, corners of
notes sticking out every side. Visible tags on the notes read: "Prompt v17",
"Rules", "Example", "Memory", "Notes", "Few-shot", "System msg", "Tool info".
One single coral-accent note (#D96C5B) peeks from the rim of the box — the
only alarm colour in the frame. A small sweat-drop on Covi's temple.
Expression determined, mouth set, eyes focused — concentrated and playful,
not despairing. Body language reads humble-but-trying.

In the background-left, standing calmly, TlhAxolotlSage — a small axolotl
sage, single tail only, wise half-lidded eyes, sleek crown with subtle
Aztec geometric references in gold (#C9A04B), tunic with Aztec geometric
motif in blue-green (#5E8C7E). A closed umbrella tucked under one arm.
Small sandals optional. Posture relaxed and observant, not yet involved in
Covi's struggle. Small in frame; Covi's chaos dominates the composition;
Tlatoāni is the still point.

Composition: Covi lower-right foreground, box centre, Tlatoāni upper-left
midground. Top-left corner deliberately kept clear for the title plate
(composited later).

Focal action: the fitting-attempt — Covi's concentrated, doomed effort to
compress too much context into a container that cannot hold it.

## Negative prompt

double tail, second tail, two tails, forked tail, twin tails, tails (plural),
gills-as-tails, external gill tufts rendered as tails, named face on Covi,
eyes-with-pupils-and-irises-and-named-expression on Covi, Covi with hair,
Covi with gender markers, Covi with a name badge, harsh rim lighting,
photo-realism, 3D render, octane render, blender render, hyperreal,
sheet-crowded background, busy wallpaper, garish saturation, neon colours,
dark mode, black background, night scene, corporate logos, watermark,
signature, artist tag, text-in-the-title-corner (title is composited
separately), misspellings in sticky-note text, speech bubble in panel 1,
dialogue text in panel 1 (panel 1 has no dialogue rendered — Covi's line is
added by compose layer), Tlatoāni smiling broadly, Tlatoāni laughing,
Tlatoāni panicked, Covi crying, Covi sad, Covi tearful, Covi defeated,
umbrella open in panel 1, notes flying in panel 1, burst in panel 1,
explosion marks, cracked box, broken box (box is merely overfull here),
bottom-left scroll plate (composited separately), bottom-right scroll plate
(composited separately).

## Expected checks (from visual-qa-loop)

- `tlatoāni.single-tail` — pass (critical; base-model drift mode)
- `tlatoāni.crown-present` — pass
- `tlatoāni.tunic-aztec-motif` — pass
- `covi.ambiguous-white` — pass
- `covi.no-named-face` — pass
- `covi.good-mood` — pass (strained but playful; never despairing)
- `palette.paper-bg` — pass
- `palette.coral-accent-present` — pass (single coral note on the box rim)
- `symbol.overstuffed-box-present` — pass
- `symbol.box-label-reads-context-105` — pass
- `composition.title-region-clear` — pass (top-left reserved for title
  composite)
- `plate.*` — not checked at the panel stage; rendered separately by
  `tt-compose` in the composited strip.
