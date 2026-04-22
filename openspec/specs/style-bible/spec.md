# Style Bible

## Purpose

Canonical visual and tonal rules for every Tlatoāni Tales strip. Any panel violating this spec is wrong — the panel is regenerated, not the spec.

## Format

- **Layout**: horizontal 3-panel newspaper strip. Mobile-first readable.
- **Aspect**: composite strip ~2:1 (wider than tall). Each panel ~square, separated by thin gutters.
- **Background**: warm paper tone (#F4E9D3 ± 5%). Lightly aged, subtle grain.
- **Margins**: consistent spacing on all four sides. No extra empty bands above or below panels.
- **Plates (matched pair)**: dark ink on cream scroll / banner motif. Symmetric. Same typeface family and weight.
  - **Episode plate** (bottom-right): overlaps all of panel 3 and ~12% of panel 2. Text: `Tlatoāni Tales NN/TOTAL` for Season 1 (TOTAL=15). Example: `Tlatoāni Tales 11/15`. The `/TOTAL` is intentional — a reader seeing `11/15` senses nearness to the complete teaching; a bare number teaches nothing. The `/TOTAL` form is only applied once the season's TOTAL is stable (convergence); until then, a bare `Tlatoāni Tales #NN` is legal. Decimal strip numbers (e.g. `TT 11.5/15`) are reserved for lessons that *teach* decimal-insertion's meaning — none exists yet.
  - **Trace plate** (bottom-left): overlaps all of panel 1 and ~12% of panel 2. Text: `[@trace spec:<name>]`. Selection rules and metadata emission in `trace-plate/spec.md`. Every strip MUST carry one — no trace, no ship.
- **Panel parse time**: < 1 second per panel. Minimal clutter.

## Visual style

- Simple, adorable, modern cartoon linework.
- Soft shadows, warm tones, restrained palette.
- Expressive faces and body language; clear silhouettes.
- Slight newspaper-strip nostalgia with modern polish.

## Palette anchors

- Paper: `#F4E9D3`
- Ink: `#2B2420`
- Tlatoāni tunic blue-green: `#5E8C7E`
- Tlatoāni crown gold: `#C9A04B`
- Covi white: `#F2EEE7`
- Accent coral (alarms, outbursts): `#D96C5B`

## Tone

- Gentle superiority; compassionate mockery.
- Ancient wisdom meets modern engineering.
- Cute enough to share, sharp enough to sting — but **Covi is never mocked cruelly**.

## Writing rules

- One concept per comic, flawlessly.
- Punchline lands in panel 3.
- First read: humor. Second read: insight.
- Minimal text. No jargon unless the joke *depends* on it.
- Teach by metaphor, never lecture.

## North star

Make readers laugh first, then realize they were taught something important.

## Trace

`@trace spec:style-bible` — every render script and every strip proposal references this.
