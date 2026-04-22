---
strip: TT 01/15
strip_id: 1
title: Volatile is dangerous
title_display: "[Volatile is dangerous]"
title_float: left
title_backing: none
title_linkable: false
lesson: S1-100-volatile-is-dangerous
lesson_id: S1-100
lesson_display: Volatile is dangerous
reinforces: []
trace_spec: concept-curriculum
depends_on: []
concepts_taught: [C01]
concepts_assumed: []
seeds:
  panel_1: 314159
  panel_2: 271828
  panel_3: 161803
---

# TT 01/15 — Volatile is dangerous

## Purpose

Strip 01 is the root beat of the series. Everything downstream — saving
(S1-200), history (S1-300), order (S1-400), reconciliation (S1-500), and
eventually convergence (S1-1400) — stands on the lived discovery Covi makes
here: **a full context window is not a workspace, it is a candle.** When the
candle guts out, the light is gone; nothing is saved because nothing was ever
*kept*.

The strip delivers the reader's first *trivial yet devastating* habit:
*"transcribe what you burned, or you didn't actually have it."* Covi learns
this through embarrassment; Tlatoāni teaches it through a single, serene
question. The overstuffed box and the umbrella are the season's opening
iconography — box = pressure, umbrella = protection from chaos, debris =
volatile failure. All three re-appear through Season 1 and grow meaning with
each return.

This strip is also the project's **retro-spec walked example**: the ChatGPT
demo image at repo root is captured here as it *was*, before plates, title,
and lesson trace existed. The upcoming local render will add those; this
proposal describes that target render under current canon, and is the
template every subsequent `strips/NN-slug/` follows.

## Panels

### Panel 1

Blocking:
- Covi (CoviFigure) in the foreground, three-quarter view, crouched over a
  cardboard box labelled `CONTEXT (105%)`. The box is visibly overfull — the
  lid will not close. Covi is pressing colourful sticky-notes downward with
  both hands; a small sweat-drop sits on their temple. Expression is
  *determined* and *playful*, not despairing (per character-canon: Covi is
  always in a good mood, even mid-stumble).
- Sticky-notes already crammed into the box, tags partly visible:
  `Prompt v17`, `Rules`, `Example`, `Memory`, `Notes`, `Few-shot`,
  `System msg`, `Tool info`. Corners stick out everywhere; the silhouette
  reads *overfull*.
- A single coral-accent note (`#D96C5B`) at the rim — the alarm colour,
  telegraphing that something is about to fail.
- Tlatoāni (TlhAxolotlSage) in the background-left, standing calmly,
  single tail visible, crown and blue-green tunic present. A **closed**
  umbrella tucked under one arm. Half-lidded wise eyes. Not yet involved.
  Small axolotl, small in frame — Covi's chaos dominates; Tlatoāni is the
  still point.
- Background: warm paper tone, subtle grain, no clutter.

Focal action: the *fitting-attempt* — Covi's concentrated, doomed effort to
compress too much context into a container that cannot hold it.

### Panel 2

Blocking:
- The box has **burst**. Lid blown off; sticky-notes exploding outward in a
  radial burst — the symbol-dictionary "explosion of notes = volatile memory
  failure" made literal. Tags fly at every angle: `Prompt v17`, `Rules`,
  `User intent`, `Few-shot`, `Tool info`, `Memory`, plus small `BOING!` and
  spark marks for the chaos.
- Covi, mid-panel, hands in the air, mouth wide, eyes squeezed shut —
  yelling. Posture is theatrical, not tragic. Body language stays playful.
- Tlatoāni, to Covi's right, has **opened the umbrella fully** and holds it
  over both of them. Notes bounce off the umbrella's canopy. Tlatoāni's
  expression has not changed from panel 1 — same calm, same half-lidded
  eyes. The umbrella is the composition's anchor — a dome of order inside
  an otherwise chaotic frame.
- Debris accumulating on the ground around their feet.
- Background: same warm paper, same subtle grain, unchanged palette —
  chaos is in the *motion*, not the palette.

Focal action: the *burst* — the volatile failure mode, rendered symbolically;
Tlatoāni's umbrella deflecting the fallout.

### Panel 3

Blocking:
- Quiet aftermath. Sticky-notes strewn across the ground — a gentle
  settle, not a fresh burst. The ruined box tipped on its side to
  stage-left, partially empty. The storm is over.
- Tlatoāni, centre-right, is **closing the umbrella** (motion: near-closed,
  not yet fully) with the same serene expression. Posture relaxed.
- Covi, centre-left, stands with hands at their sides. A soft blush on
  their cheeks — the *realization* moment (per character-canon: Covi
  sometimes blushes when realizing a mistake). Eyes half-closed in
  sheepish acknowledgement. Body language reads *slightly wiser*, not
  defeated.
- Tlatoāni delivers the wisdom line in a soft speech bubble (see Dialogue).
- Background: warm paper, strewn notes as subtle foreground texture. No
  added clutter.

Focal action: Tlatoāni's wisdom line — the single calm observation that
retroactively names the whole strip's teaching.

## Dialogue

- **Panel 1 — Covi** *(strained, through gritted teeth)*: "Almost
  fitting..."
- **Panel 2 — Covi** *(full-voice, theatrical)*: "MY PRECIOUS CONTEXT!"
- **Panel 3 — Tlatoāni** *(calm, unhurried)*: "...that was committed,
  right?"

Dialogue is intentionally sparse. Covi carries the emotional arc (strained
→ panicked → sheepish) through posture; Tlatoāni speaks exactly once, and
the single question does the lesson's whole work. No jargon. No lecture.
The punchline lands in panel 3 by implication, not statement — the reader
completes the teaching.

## Plate declarations

Per `trace-plate/spec.md` — three plates, declared here, rendered by the
orchestrator (not by the panel prompts):

- **Title plate** (top-left, Qwen-Image rendered over FLUX composite):
  `[Volatile is dangerous]`. Stylised — trembling strokes, a letter
  half-dissolved, expressive of the lesson's meaning. `title_float: left`,
  `title_backing: none`, `title_linkable: false`.
- **Trace + Lesson plate** (bottom-left, cream scroll, dark ink, chrome):
  - Line 1: `[@Lesson S1-100]`
  - Line 2: `[@trace spec:concept-curriculum]`
  Overlaps all of panel 1 and ~12% of panel 2, symmetric to the episode
  plate.
- **Episode plate** (bottom-right, cream scroll, dark ink, chrome):
  `Tlatoāni Tales 01/15`. Overlaps all of panel 3 and ~12% of panel 2.
  The `/15` denominator is the pedagogical payload — the reader reads
  `01/15` and senses the arc, not a bare position.

## Accessible alt text

A three-panel warm-paper newspaper comic titled *Volatile is dangerous*.
Panel 1: a small white ambiguous figure named Covi crouches over a
cardboard box labelled `CONTEXT (105%)`, pressing colourful sticky-notes
labelled `Prompt v17`, `Rules`, `Example`, `Memory` into the box with
concentrated effort; behind them stands Tlatoāni, a small axolotl sage
with a single tail, a gold crown with subtle Aztec geometry, and a
blue-green tunic, holding a closed umbrella tucked under one arm and
watching calmly. Panel 2: the box bursts; sticky-notes and small `BOING!`
marks fly outward in every direction; Covi throws their arms up and
shouts "MY PRECIOUS CONTEXT!"; Tlatoāni has opened the umbrella fully
and holds it over both of them, deflecting the debris with the same
serene expression. Panel 3: quiet aftermath; sticky-notes lie strewn
across the ground; Tlatoāni closes the umbrella; Covi blushes
sheepishly; Tlatoāni says softly, "...that was committed, right?" The
strip carries a top-left title plate reading `[Volatile is dangerous]`,
a bottom-left plate reading `[@Lesson S1-100]` on line 1 and
`[@trace spec:concept-curriculum]` on line 2, and a bottom-right plate
reading `Tlatoāni Tales 01/15`.

## Caption for publishing

```
Tlatoāni Tales 01/15 — [Volatile is dangerous] — @Lesson S1-100 / @trace spec:concept-curriculum
```

## Trace

`@trace spec:style-bible, spec:character-canon, spec:symbol-dictionary, spec:trace-plate, spec:concept-curriculum, spec:narrative-arc, spec:visual-qa-loop`
`@Lesson S1-100`
