# Concept Curriculum

## Purpose

The ordered ladder of lessons the reader climbs over the season. Each concept has dependencies (concepts that must land first to make sense of this one). This spec is the DAG the whole series converges toward.

When strip #12 reveals that strip #6 missed a beat, we update this spec (or insert a new concept), mark dependent strips stale via the cache layer, and re-render. Convergence.

## Reader promise

After each strip, the reader leaves with a *trivial yet devastating* improvement: a habit they can ask their AI to adopt tomorrow. No math required. The reader never sees the words "Lamport clock", "CRDT", or "monotonic convergence" — until the very last strip, where those words become punchlines because the reader *already has* the intuition.

## The ladder

> C## = internal concept ID (historical). S1-NNN = canonical public lesson ID. Both stay; they cross-reference.

| # | Concept | Lesson slug | One-line reader takeaway | Depends on |
|---|---|---|---|---|
| C01 | Clean context is fragile | `S1-100` | "Starting fresh every time loses everything that mattered." | — |
| C02 | Saving context (naively) | `S1-200` | "Copy-pasting isn't saving. Saving means you can find it again." | C01 |
| C03 | Git history is time-context | `S1-300` | "Your project remembers what you forgot — if you let it." | C02 |
| C04 | Commits impose order | `S1-400` | "Every commit answers: what was true before this, and after." (Lamport, unnamed) | C03 |
| C05 | Specs merge without conflict | `S1-500` | "Two edits to the same rule? The spec reconciles, not you." (CRDT, unnamed) | C04 |
| C06 | Specs are the contract | `S1-600` | "Ask for what you want in writing. Code converges toward it, not the other way." | C05 |
| C07 | Iteration was always there | `S1-700` | "You've been iterating the whole time. You just weren't aiming." | C06 |
| C08 | Observability > unit tests | `S1-800` | "Tests tell you if yesterday's mistake reappeared. Observability tells you what today's mistake IS." | C07 |
| C09 | Observability > just logs | `S1-900` | "Logs are raw material. Observability is what you do with them." | C08 |
| C10 | Dashboards must add observability | `S1-1000` | "A dashboard that only shows raw logs is a louder log. Observability is the shape the dashboard adds." | C09 |
| C11 | Telemetry = meaning of logs | `S1-1100` | "A number across time has a shape. That shape is the truth." | C10 |
| C12 | Meaning is operable | `S1-1200` | "You can add, multiply, and compare shapes. That's engineering with eyes open." | C11 |
| C13 | The loop closes | `S1-1300` | "Last iteration's meaning is this iteration's input. Now you're not guessing — you're converging." | C12 |
| C14 | BOOM — monotonic convergence | `S1-1400` | "Ask your AI for monotonic_convergence. You already earned it." | C01–C13 |
| C15 | Proof by self-reference | `S1-1500` | "Not sure it applies to your work? This comic was made that way. Repo is right there." | C14 |

Lesson slugs are canon per `lessons/spec.md`. Each concept has exactly one primary lesson — the humane, readable phrase the reader carries out of the strip.

## Strip mapping (initial — subject to revision)

| Strip | Concept(s) | Notes |
|---|---|---|
| TT 01/15 | C01 | Already shipped (retro-spec) — context overflow demo |
| TT 02/15 | C02 | Covi "saves" context by screenshotting it |
| TT 03/15 | C03 | Tlatoāni opens the notebook; Covi realizes git log was there all along |
| TT 04/15 | C04 | Two hourglasses, one labeled "before", one "after" — Tlatoāni just points |
| TT 05/15 | C05 | Two Covis edit the same scroll, expect conflict, scroll shows reconciled |
| TT 06/15 | C06 | Covi asks AI for feature; Tlatoāni hands them a sealed scroll first |
| TT 07/15 | C07 | Covi on treadmill insisting they're working; Tlatoāni rotates the treadmill toward a staircase |
| TT 08/15 | C08 | Covi shows green tests, fire visible through window; Tlatoāni hands a telescope |
| TT 09/15 | C09 | Covi reads a log line; Tlatoāni holds up a dashboard with the *shape* of that line over time |
| TT 10/15 | C10 | Covi proudly unveils a dashboard that is just bigger log lines; Tlatoāni quietly overlays a trend curve — the dashboard gains observability |
| TT 11/15 | C11 | Covi watches a single number; Tlatoāni shows it as a curve with meaning |
| TT 12/15 | C12 | Covi tries to add two curves; Tlatoāni shows the sum is a new meaning |
| TT 13/15 | C13 | The curve from last week becomes this week's prompt |
| TT 14/15 | C14 | Covi casually says "AI, monotonic_convergence please" — it just works — Tlatoāni nods |
| TT 15/15 | C15 | Skeptic: "You can't apply this anywhere." Covi: "Of course you can! This comic was!" — GitHub URL in frame |

Concepts may split across multiple strips as we go. Mapping is part of the spec and updates in place.

## Dependency enforcement

Strip proposals MUST declare `Depends on: [Cxx, Cxx]` in their frontmatter. The orchestrator refuses to render a strip whose dependencies haven't shipped earlier in the sequence.

## Trace

`@trace spec:concept-curriculum`
`@Lesson S1-1500`
