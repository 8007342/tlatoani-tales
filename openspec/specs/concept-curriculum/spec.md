# Concept Curriculum

## Purpose

The ordered ladder of lessons the reader climbs over the season. Each concept has dependencies (concepts that must land first to make sense of this one). This spec is the DAG the whole series converges toward.

When strip #12 reveals that strip #6 missed a beat, we update this spec (or insert a new concept), mark dependent strips stale via the cache layer, and re-render. Convergence.

## Reader promise

After each strip, the reader leaves with a *trivial yet devastating* improvement: a habit they can ask their AI to adopt tomorrow. No math required. The reader never sees the words "Lamport clock", "CRDT", or "monotonic convergence" — until the very last strip, where those words become punchlines because the reader *already has* the intuition.

## The ladder

| # | Concept | One-line reader takeaway | Depends on |
|---|---|---|---|
| C01 | Clean context is fragile | "Starting fresh every time loses everything that mattered." | — |
| C02 | Saving context (naively) | "Copy-pasting isn't saving. Saving means you can find it again." | C01 |
| C03 | Git history is time-context | "Your project remembers what you forgot — if you let it." | C02 |
| C04 | Commits impose order | "Every commit answers: what was true before this, and after." (Lamport, unnamed) | C03 |
| C05 | Specs merge without conflict | "Two edits to the same rule? The spec reconciles, not you." (CRDT, unnamed) | C04 |
| C06 | Specs are the contract | "Ask for what you want in writing. Code converges toward it, not the other way." | C05 |
| C07 | Iteration was always there | "You've been iterating the whole time. You just weren't aiming." | C06 |
| C08 | Observability > unit tests | "Tests tell you if yesterday's mistake reappeared. Observability tells you what today's mistake IS." | C07 |
| C09 | Observability > just logs | "Logs are raw material. Observability is what you do with them." | C08 |
| C10 | Telemetry = meaning of logs | "A number across time has a shape. That shape is the truth." | C09 |
| C11 | Meaning is operable | "You can add, multiply, and compare shapes. That's engineering with eyes open." | C10 |
| C12 | The loop closes | "Last iteration's meaning is this iteration's input. Now you're not guessing — you're converging." | C11 |
| C13 | BOOM — monotonic convergence | "Ask your AI for monotonic_convergence. You already earned it." | C01–C12 |
| C14 | Proof by self-reference | "Not sure it applies to your work? This comic was made that way. Repo is right there." | C13 |

## Strip mapping (initial — subject to revision)

| Strip | Concept(s) | Notes |
|---|---|---|
| TT #01 | C01 | Already shipped (retro-spec) — context overflow demo |
| TT #02 | C02 | Covi "saves" context by screenshotting it |
| TT #03 | C03 | Tlatoāni opens the notebook; Covi realizes git log was there all along |
| TT #04 | C04 | Two hourglasses, one labeled "before", one "after" — Tlatoāni just points |
| TT #05 | C05 | Two Covis edit the same scroll, expect conflict, scroll shows reconciled |
| TT #06 | C06 | Covi asks AI for feature; Tlatoāni hands them a sealed scroll first |
| TT #07 | C07 | Covi on treadmill insisting they're working; Tlatoāni rotates the treadmill toward a staircase |
| TT #08 | C08 | Covi shows green tests, fire visible through window; Tlatoāni hands a telescope |
| TT #09 | C09 | Covi reads a log line; Tlatoāni holds up a dashboard with the *shape* of that line over time |
| TT #10 | C10 | Covi watches a single number; Tlatoāni shows it as a curve with meaning |
| TT #11 | C11 | Covi tries to add two curves; Tlatoāni shows the sum is a new meaning |
| TT #12 | C12 | The curve from last week becomes this week's prompt |
| TT #13 | C13 | Covi casually says "AI, monotonic_convergence please" — it just works — Tlatoāni nods |
| TT #14 | C14 | Skeptic: "You can't apply this anywhere." Covi: "Of course you can! This comic was!" — GitHub URL in frame |

Concepts may split across multiple strips as we go. Mapping is part of the spec and updates in place.

## Dependency enforcement

Strip proposals MUST declare `Depends on: [Cxx, Cxx]` in their frontmatter. The orchestrator refuses to render a strip whose dependencies haven't shipped earlier in the sequence.

## Trace

`@trace spec:concept-curriculum`
