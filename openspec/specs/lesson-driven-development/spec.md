# Lesson-Driven Development

## Purpose

> The LESSON is the most meaningful unit in this project. Each COMIC is addressed AS a LESSON. LESSON-DRIVEN DEVELOPMENT treats the lesson as the primary artefact — specs govern the lesson; comics instantiate the lesson; code @traces point at the lesson. No comic ships without a teaching lesson.

Tlatoāni Tales is a teaching project. Everything else — scripts, renders, caches, orchestrators, CRDTs — exists to deliver a *lesson* into a reader's head. **Lesson-Driven Development (LDD)** names that ordering explicitly so the discipline is enforceable, not aspirational.

LDD asserts a contract higher than behaviour, outcome, or vocabulary:

- **TDD** asserts code behaves correctly. It cannot answer *"right for what?"*
- **BDD** asserts users experience the intended outcome. It cannot answer *"which outcome matters?"*
- **DDD** structures code around the terms of a domain. It cannot answer *"teach me the domain first."*
- **LDD** asserts *the reader learns the intended insight*. Behaviour, outcome, and domain terms all descend from the lesson.

The lesson is the highest-level contract in the repo. Tests, behaviours, and domain terms exist *below* the lesson, in service of it.

## The lesson as primary artefact

A lesson in this project is a seven-field object. Those seven fields are the lesson-spec contract — a lesson spec that omits any of them is malformed:

| # | Field | What it is |
|---|---|---|
| a | **Bare abstract thought** | One sentence. The teaching, stripped of joke, character, or citation. If it's empty, the lesson doesn't exist yet. |
| b | **Position in hierarchy** | Season + number + dependencies. `S1-NNN-<slug>` and the list of lessons this one requires to have already landed. |
| c | **Code references** | Places in the project's source that cite the lesson via `@Lesson S1-NNN` annotations. Every reference is a vote that the lesson is load-bearing. |
| d | **Script** | The three-panel beat sheet that carries the thought. |
| e | **Joke** | The compressed surprise the reader laughs at. |
| f | **Punchline** | The specific line or image that lands the joke. |
| g | **"Aha!" moment** | Prose description of the internal click the reader is meant to feel — what changes in them. |

Fields (a) and (g) are the teaching. Fields (d)–(f) are the delivery. Fields (b)–(c) are the wiring. A comic without (a) and (g) has nothing to say; a lesson without (d)–(f) has nothing to ship.

## Lifecycle

```
idea → spec → referenced → instantiated → observed
        │        │              │             │
        │        │              │             └── Calmecac aggregates coverage
        │        │              └── strip proposal + rendered panels
        │        └── @trace spec:… and @Lesson S1-NNN in code, scripts, docs
        └── openspec/specs/lessons/Sn-NNN-slug/spec.md
```

Each step leaves a `@trace spec:lesson-driven-development` and a `@Lesson S1-NNN` the reader can follow:

1. **Idea.** Author notes a candidate teaching beat. Per the curation rule, candidates stay "pending author decision" until explicitly canonized.
2. **Spec.** Author writes `openspec/specs/lessons/Sn-NNN-slug/spec.md` with all seven fields.
3. **Referenced.** Other specs cite the lesson; code annotates with `@Lesson S1-NNN`; commits carry the annotation.
4. **Instantiated.** A strip proposal declares `lesson: Sn-NNN-slug`; the orchestrator renders panels.
5. **Observed.** Calmecac (Wave 3 viewer) aggregates coverage across specs, meta-examples, strips, and code citations; a reader sees what the lesson touches.

## Invariants

- **No comic without a lesson.** `tt-render verify` rejects a strip whose `proposal.md` declares no `lesson:` field. A strip with `lesson: S1-NNN-slug` pointing at a nonexistent lesson spec is also rejected.
- **No lesson without a teaching.** A lesson spec whose *bare abstract thought* (field a) is empty cannot be referenced by a strip. The orchestrator resolves `@Lesson S1-NNN` at render time and refuses to composite a strip whose lesson has an empty teaching.
- **Append-only.** Lessons follow the CRDT discipline declared in `lessons/spec.md`. Edits refine; removals tombstone. A tombstoned lesson ID never reuses — an archival fork reading a strip that cites the tombstone can still resolve it to its last canonical meaning.
- **Trace companionship.** Every `@trace spec:X` annotation in code SHOULD carry a companion `@Lesson S1-NNN` when the governance traces back to a lesson. Pure-infrastructure specs (`trace-plate`, `orchestrator`) may trace without a lesson companion; teaching-adjacent specs should not.

## Relationship to sibling specs

| Spec | Role relative to LDD |
|---|---|
| `concept-curriculum` | The ladder of concepts each lesson teaches. LDD treats this as the ordered DAG of teachings. |
| `lessons` | The registry + per-lesson specs. Canonical source of lesson identity, slugs, coverage, tombstones. |
| `seasons` | The clustering. Defines the `S<n>-<number>-<slug>` namespace LDD cites. |
| `narrative-arc` | The dramatic shape across lessons. LDD's fields (d)–(f) sit inside the arc. |
| `trace-plate` | The in-frame citation mechanism. How a rendered strip exposes its lesson to the reader. |
| `visual-qa-loop` | The drift-control loop that gates lesson instantiation. A rendered strip that fails VLM critique does not count as instantiating its lesson. |
| `calmecac` *(Wave 3, not yet written)* | The viewer that observes lessons. Aggregates coverage — the reader-facing face of LDD. |

## LDD vs TDD / BDD / DDD

| Methodology | Primary assertion | Failure mode | Where it sits in this repo |
|---|---|---|---|
| **TDD** | Code behaves correctly. | *Right for what?* — passes tests, misses intent. | Below the lesson; tests validate instantiation details. |
| **BDD** | Users experience the intended outcome. | *Which outcome matters?* — scenarios without a reason. | Below the lesson; outcomes descend from the teaching. |
| **DDD** | Code is structured around domain terms. | *Teach me the domain first.* — vocabulary without motivation. | Below the lesson; the domain is what the lesson teaches. |
| **LDD** | The reader learns the intended insight. | *None within this frame.* — if the reader doesn't learn, the project failed at its highest contract. | The top. Everything else descends from it. |

## Process: adding a new lesson

1. **Reserve an ID.** Pick a slug in the season's namespace: `S<n>-<number>-<slug>`. Season 1 uses strict sequential hundreds (`S1-100`, `S1-200`, … `S1-1500`); decimal insertion is reserved for a future lesson that *teaches* decimal-insertion's meaning. New beats added during authoring trigger a renumbering sweep (see the April 2026 tombstone rows in `lessons/spec.md`).
2. **Write the per-lesson spec** at `openspec/specs/lessons/Sn-NNN-slug/spec.md` with all seven fields (a)–(g).
3. **Register** the lesson in `lessons/spec.md` (registry table + coverage section).
4. **Reference** the lesson from `concept-curriculum` if it teaches a canonical concept on the ladder.
5. **Start a strip proposal** in `strips/NN-slug/proposal.md` declaring `lesson: Sn-NNN-slug`.
6. **Render.** The orchestrator composites; Calmecac picks up the new lesson automatically through its coverage aggregation.

Per the curation rule, steps 2–4 require explicit author approval before canonization. Reserved-but-unapproved IDs become tombstones, which is itself an on-brand demonstration of the CRDT mechanic.

## Teach-by-example note

This project's own development follows LDD. Each commit in the repo's Lamport chain can be traced back to a lesson — or at minimum to a spec that governs a lesson. The methodology is self-describing: LDD is the discipline the comic teaches about governing code with specs, applied one level up to governing comics with lessons.

*Candidate meta-example, pending author curation:* LDD as the methodology meta-example — the project's own process is an instance of the thing the project teaches. Flagged for author decision, not canonized here.

## Trace

`@trace spec:lesson-driven-development, spec:lessons, spec:concept-curriculum, spec:seasons`
`@Lesson S1-1500` *(proof-by-self-reference — the methodology is self-describing)*
