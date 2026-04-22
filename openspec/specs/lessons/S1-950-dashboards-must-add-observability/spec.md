# S1-950 — Dashboards must add observability

## Abstract

A dashboard that only mirrors statistics isn't a dashboard — it's a passive display. Real dashboards *add* observability: they surface relationships, provenance, and convergence state that don't exist in the raw numbers.

## Position

- Season: S1
- Number: 950 (decimal-style insertion between S1-900 and S1-1000, per the stepped-hundreds namespace)
- Predecessors: S1-900-logs-are-ingredients, S1-800-see-the-now
- Successors: S1-1000-shape-has-meaning

## References in this project

- `openspec/specs/calmecac/spec.md` *(future — Wave 3; the storyboard viewer that is this lesson made tangible)*
- `openspec/specs/visual-qa-loop/spec.md` — drift scores are dashboarded
- `openspec/specs/meta-examples/spec.md` ME08 — VLM drift loop as telemetry
- `openspec/specs/concept-curriculum/spec.md` (C09.5)

## Script

**Panel 1.** Covi stands proudly in front of a big screen covered in beautiful charts, gauges, and sparklines. Every color is on-brand. A caption floats near Covi: *"Look, a dashboard!"* Tlatoāni stands calmly to the side, holding their notebook, eyes half-lidded.

**Panel 2.** Tlatoāni turns slightly toward Covi. One short line: *"What do you see?"* Covi, beaming, points at the charts: *"Numbers going up and to the right!"* A small coral accent glows on one chart — foreshadowing.

**Panel 3.** Tlatoāni walks around behind the dashboard and turns it to face the reader. On the *back* of the dashboard: a hand-drawn storyboard. Each chart from panel 1 is re-rendered as a small cell connected by arrows labelled *because*, *then*, *unless*. A scroll hovers above: *"A dashboard that doesn't teach its story is just wallpaper that moves."* Covi's eyes go wide. The trace plate reads `@Lesson S1-950` / `@trace spec:visual-qa-loop`.

## Joke

Confusing *charts* for *understanding*. A wall of green graphs feels like insight; it is often only motion in a flattering palette.

## Punchline

The back of the dashboard is the real dashboard. The charts were the *ingredients* (S1-900 rhymes here); the story — the *because/then/unless* graph — is what observability actually adds.

## Aha moment

Observability isn't a graph — it's a *relationship surfaced between signal and meaning*. A chart is data; an observable dashboard is data **plus** the reasoning scaffold that makes the data answer a question you couldn't answer before. Calmecac, the repo's storyboard viewer, is this lesson made tangible: each panel cell links back to its governing spec, its lesson, and the iterations that produced it — the dashboard *is* the story of convergence.

## Candidate meta-example (pending author decision)

> Calmecac is the live demonstration of this lesson — the storyboard viewer surfaces provenance, lesson linkage, and convergence state that raw render stats cannot. Flag for author review as a candidate ME## (not canonized here per curation rules in `feedback_tlatoani_tales_curation.md`).

## Trace

`@trace spec:lessons, spec:S1-950-dashboards-must-add-observability, spec:visual-qa-loop`
`@Lesson S1-950`
