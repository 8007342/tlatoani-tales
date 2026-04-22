# Seasons

## Purpose

Tlatoāni Tales is planned as a multi-season arc. A **season** is a coherent thesis — a self-contained climb the reader completes — not a marketing unit. Seasons exist so that:

- Lessons cluster under a named thrust the reader can hold in one phrase.
- New lessons can be *inserted* at any point (decimal numbering) without renumbering the world.
- The project has a map: readers, contributors, and the orchestrator can all ask *"which season governs this?"* and get one answer.
- Forward planning is legible. Season 2 seeds can live in this repo as placeholder IDs without requiring finished content.

Two seasons are planned. **MVP ships Season 1 only.** Season 2 is canonical forward-planning — its structural dependencies (Silverblue-only scripts, podman, toolbox isolation, drop-privileges flags) are already honored by this repo's operational discipline, so that when Season 2 lands it can teach by pointing at its own substrate. Proof by self-reference, rehearsed.

## Season numbering format

Every published lesson receives an ID of the form:

```
S<n>-<number>-<slug>
```

- `S<n>` — season integer (`S1`, `S2`, …).
- `<number>` — lesson index, stepped by **100** starting at **100** (`100, 200, 300, …, 1400, …`).
- `<slug>` — the lesson's kebab-case phrase (the humane hook; see `lessons/spec.md`).

**Insertion rule.** A new lesson may land at *any* decimal between two existing numbers. Examples:

| Insert | Between | Why |
|---|---|---|
| `S1-150` | `S1-100` and `S1-200` | New beat belongs after "volatile is dangerous" but before "save means findable". |
| `S1-950` | `S1-900` and `S1-1000` | Dashboards-must-add-observability lesson (this turn's decision). |
| `S2-250` | `S2-200` and `S2-300` | Any future Season 2 insertion. |

Gaps of 100 exist precisely to absorb insertions without renumbering. Finer insertions (S1-910, S1-925) are legal but discouraged unless the author judges them necessary.

**ID stability.** Once a lesson ID is published (merged to main), **it never reuses**. A retired ID becomes a tombstone — see `meta-examples/spec.md` ME10 and `lessons/spec.md` tombstone rules. This is the same CRDT discipline the comic teaches (C05).

## Season 1 registry

**Thesis: "From volatile context to monotonic convergence."**

| ID | Slug | Strip | Source concept |
|---|---|---|---|
| `S1-100` | `volatile-is-dangerous` | TT #01 | C01 |
| `S1-200` | `save-means-findable` | TT #02 | C02 |
| `S1-300` | `memory-lives-in-history` | TT #03 | C03 |
| `S1-400` | `discrete-time` | TT #04 | C04 |
| `S1-500` | `edits-that-reconcile` | TT #05 | C05 |
| `S1-600` | `ask-in-writing` | TT #06 | C06 |
| `S1-700` | `loops-need-aim` | TT #07 | C07 |
| `S1-800` | `see-the-now` | TT #08 | C08 |
| `S1-900` | `logs-are-ingredients` | TT #09 | C09 |
| `S1-950` | `dashboards-must-add-observability` | *TBD — inserts between TT #09 and TT #10* | (new — observability-of-observability) |
| `S1-1000` | `shape-has-meaning` | TT #10 | C10 |
| `S1-1100` | `meaning-is-operable` | TT #11 | C11 |
| `S1-1200` | `loop-closes` | TT #12 | C12 |
| `S1-1300` | `monotonic-convergence` | TT #13 | C13 |
| `S1-1400` | `proof-by-self-reference` | TT #14 | C14 |

`S1-950` is the canonical example of the insertion rule: the author recognized mid-arc that *dashboards without observability* is a distinct teaching beat, not a sub-case of `logs-are-ingredients` (S1-900) or `shape-has-meaning` (S1-1000). It earned its own ID. Its comic strip number is TBD; it will likely slot between TT #09 and TT #10 without renumbering the existing strips.

## Season 2 sketch

**Thesis: "From dangerously-skip-permissions to podman-run-drop-privileges."**

Season 2 teaches the shift from "just let the AI do whatever" to scoped, reversible, sandboxed trust. It's the operational sibling of Season 1: S1 is *how do I converge toward intent?*, S2 is *how do I do that without burning my machine down?*

| ID | Slug (sketch) | Notes |
|---|---|---|
| `S2-100` | `dangerously-skip-permissions` | The opener — the tempting shortcut. |
| `S2-???` | *(TBD — immutable OS / ostree)* | Silverblue's contract: the host can't be mutated; that's a feature. |
| `S2-???` | *(TBD — toolbox as disposable workspace)* | One project, one toolbox, one `rm` to reset. |
| `S2-N` | `podman-run-drop-privileges` | The closer — least-privilege container execution as the mirror of S1's monotonic convergence. |

> **Season 2 lessons are placeholder sketches.** Individual lesson specs do **not** exist yet and will only be written when the author approves them. The registry above is a forward-planning seed, not a commitment. Slugs, count, and ordering will change.

## MVP scope

| Layer | MVP includes | MVP defers |
|---|---|---|
| Season 1 | All 15 lessons (`S1-100` … `S1-1400`, including `S1-950`). Strips, prompts, renders, lesson specs, coverage. | — |
| Season 2 | Seasons registry entries as **forthcoming** stubs (above). | Per-lesson specs, strips, prompts, renders. |
| Calmecac viewer | Ships with Season 1 content. | Season 2 content surfaces only when lessons land. |
| Repo operational stance | Already Season-2-compliant: Silverblue-only scripts, podman/toolbox isolation, drop-privileges flags, no host mutation. | Season 2 *content* that will eventually teach using this very repo as the example. |

The repo's operational discipline being Season-2-compliant *before* Season 2 lessons are written is intentional — it's the same move the project already makes at the Season 1 finale (TT #14, `S1-1400`, proof by self-reference). When Season 2 arrives, it teaches by pointing at the substrate. The substrate is already correct.

## Season boundaries and invariants

- **A season is a thesis.** S1 = *"from vibes to convergence"*. S2 = *"from skip-checks to scoped trust"*. If a proposed lesson doesn't serve the season's thesis, it belongs in a different season (or a new one).
- **Slugs never reuse across seasons.** Each season gets a unique phrase bank. `volatile-is-dangerous` belongs to S1; S2 may not reuse that slug even as a variant. (Tombstone rules apply per-season as well.)
- **Cross-season citation is allowed.** A Season 2 lesson may cite `S1-1300` or `@Lesson S1-500`. Lessons form a DAG across seasons; seasons are a clustering, not a wall.
- **Season-internal order is the numbering.** The `<number>` field imposes total order within a season. Across seasons, order is lexicographic on `(season, number)`.
- **New seasons require author approval.** Adding `S3` is a structural change, not a housekeeping edit.

## Relationship to existing specs

This spec introduces the `S<n>-<number>-<slug>` namespace. It does **not** rewrite existing specs. Migration is author-curated and may happen in any order:

| Spec | What will change when author migrates |
|---|---|
| `concept-curriculum/spec.md` | C01–C14 map onto `S1-100` … `S1-1400`. New row for the `S1-950` concept (dashboards-must-add-observability). |
| `narrative-arc/spec.md` | Season 1 arc absorbs the `S1-950` insertion between TT #09 and TT #10. |
| `lessons/spec.md` | Registry rows adopt `S1-NNN-<slug>` IDs. Existing `lesson_<snake>` slugs become aliases or get formally retired via tombstone (author's call). |
| `meta-examples/spec.md` | Per-lesson references update from concept IDs to season IDs where it sharpens the citation. |

**Do not edit those files from this spec's authority.** They are author-curated. This spec is the namespace; they are the content.

## Trace

`@trace spec:seasons`
`@Lesson S1-1400`
