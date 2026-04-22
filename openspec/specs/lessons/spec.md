# Lessons

## Purpose

The **humane layer** on top of the technical trace namespace. Each lesson is a warm, memorable phrase the reader carries out of a strip — readable without clicking anything. Slugs are stable identifiers; coverage sets (specs + meta-examples + strips) are a CRDT that only grows except through tombstones.

Two layers of citation run through the project:

| Layer | Format | For |
|---|---|---|
| Technical trace | `@trace spec:<name>` | "Which contract governs this code?" — grep-friendly, precise |
| Lesson trace | `@Lesson <slug>` | "Which teaching is this in service of?" — human-readable, reader-friendly |

Both appear on every strip's left plate. Non-clickers read the lesson and get the point. Clickers follow the URL into the CRDT of all wisdom gathered under that lesson.

## Invariants

- **Slug format**: `lesson_<snake_case_phrase>`. Lowercase. No leading/trailing underscore. Phrase is meaningful standalone.
- **Slugs never reuse.** Retired slugs become tombstones; archival forks stay legible.
- **Each strip declares ONE primary lesson** in its `proposal.md`. Strips may reinforce additional lessons (listed in `reinforces:`).
- **Each lesson declares its coverage**: which specs implement or describe it, which meta-examples exemplify it, which strips teach or reinforce it.

## Registry

| Slug | Display | Reader takeaway | Primary | Reinforces |
|---|---|---|---|---|
| `lesson_volatile_is_dangerous` | Volatile is dangerous | "Starting fresh loses everything that mattered." | TT #01 | TT #02, TT #14 |
| `lesson_save_means_findable` | Save means findable | "Copy-pasting isn't saving. Saving means you can find it again." | TT #02 | — |
| `lesson_memory_lives_in_history` | Memory lives in history | "Git remembers what you forgot — if you let it." | TT #03 | TT #04 |
| `lesson_discrete_time` | Discrete time | "Every commit answers what was true before and after." | TT #04 | TT #05 |
| `lesson_edits_that_reconcile` | Edits that reconcile | "Two edits to the same rule? The spec reconciles, not you." | TT #05 | — |
| `lesson_ask_in_writing` | Ask in writing | "Ask for what you want in writing. Code converges toward it." | TT #06 | — |
| `lesson_loops_need_aim` | Loops need aim | "You've been iterating. You just weren't aiming." | TT #07 | — |
| `lesson_see_the_now` | See the now | "Tests check yesterday. Observability shows today." | TT #08 | TT #09, TT #10 |
| `lesson_logs_are_ingredients` | Logs are ingredients | "Raw logs aren't truth. They're material for it." | TT #09 | — |
| `lesson_shape_has_meaning` | Shape has meaning | "A number across time has a shape. That shape is the truth." | TT #10 | TT #11 |
| `lesson_meaning_is_operable` | Meaning is operable | "You can add, compare, and transform shapes. That's engineering with eyes open." | TT #11 | — |
| `lesson_loop_closes` | The loop closes | "Last iteration's meaning is this iteration's input." | TT #12 | — |
| `lesson_monotonic_convergence` | Monotonic convergence | "Ask your AI for it. You already earned it." | TT #13 | — |
| `lesson_proof_by_self_reference` | Proof by self-reference | "Not sure it applies? This comic was made that way." | TT #14 | — |

These slugs are author-curated. **Do not add new lessons** without explicit author approval — see the curation rule.

## Coverage (what each lesson points to)

Each lesson's entry is the aggregation node: the reader who clicks through lands here and sees every spec, meta-example, and strip that touches this teaching. This is the CRDT payoff — one link, the whole wisdom graph.

### `lesson_volatile_is_dangerous`
- **Concepts**: C01
- **Specs**: `concept-curriculum`
- **Meta-examples**: ME02 (OpenSpec delta-merge as the durable counterpart)
- **Strips**: TT #01 (primary), TT #02 (reinforce), TT #14 (callback)

### `lesson_discrete_time`
- **Concepts**: C04
- **Specs**: `concept-curriculum`, `meta-examples`
- **Meta-examples**: ME01 (git history as Lamport clock)
- **Strips**: TT #04 (primary), TT #05 (reinforce)

### `lesson_edits_that_reconcile`
- **Concepts**: C05
- **Specs**: `licensing` (the live CRDT demo of this lesson), `concept-curriculum`, `meta-examples`
- **Meta-examples**: ME02 (OpenSpec delta-merge), ME03 (licensing rule table), ME05 (panel cache G-Set), ME09 (LoRA-hash in cache key)
- **Strips**: TT #05 (primary)

### `lesson_see_the_now`
- **Concepts**: C08
- **Specs**: `visual-qa-loop`, `concept-curriculum`
- **Meta-examples**: ME08 (VLM drift loop), ME11 (trace plate as in-frame observability)
- **Strips**: TT #08 (primary), TT #09 (reinforce), TT #10 (reinforce)

### `lesson_loop_closes`
- **Concepts**: C12
- **Specs**: `visual-qa-loop`, `concept-curriculum`
- **Meta-examples**: ME08 (the loop itself)
- **Strips**: TT #12 (primary)

### `lesson_proof_by_self_reference`
- **Concepts**: C14
- **Specs**: `meta-examples`, `concept-curriculum`, `narrative-arc`, **all of them**
- **Meta-examples**: ME01–ME12 (the entire ledger is the proof)
- **Strips**: TT #14 (primary)

*Remaining lessons: coverage sections fill in as each strip's proposal lands. Empty is legal; the CRDT grows monotonically.*

## CRDT properties

- **Commutative**: coverage lists are sets; order-independent union.
- **Associative**: merging coverage-additions from parallel branches is well-defined.
- **Idempotent**: re-adding a spec or ME## to a lesson's coverage is a no-op.
- **Monotonic**: coverage grows. Removal is not a normal operation; it requires tombstoning the specific entry (rare — e.g. a spec was renamed).

## Tombstones

A retired lesson slug is struck through, dated, and kept:
`~~lesson_obsolete~~` *(tombstoned 2026-MM-DD — reason)*

New slugs may not reuse the old name. Archival forks that read strips referencing the tombstoned slug can still resolve it to its last canonical meaning.

## URL forms

- **Lesson search URL** (the "favorite query"):
  `https://github.com/8007342/tlatoani-tales/search?q=%40Lesson+<slug>&type=code`
- **Direct registry section**:
  `https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/lessons/spec.md`

The search URL surfaces every code comment, strip proposal, commit message, and caption that carries `@Lesson <slug>`. That's the reader's door into the CRDT.

## Per-strip declaration

In each strip's `proposal.md`:

```yaml
lesson: lesson_volatile_is_dangerous
reinforces: []
trace_spec: concept-curriculum
```

The orchestrator reads these, composites the two-line left plate, and emits `METADATA.json` with both trace URLs.

## Meta-observation

This spec is itself an instance of the lesson it was written to describe. A reader who asks *"which spec teaches me observability?"* finds `visual-qa-loop`. A reader who asks *"which teaching is this comic delivering?"* finds `lessons`. The lesson layer is the observability-of-the-observability — it tells the reader not what the code is doing but what the artefact is trying to teach. See `meta-examples/spec.md` ME13.

## Trace

`@trace spec:lessons`
`@Lesson lesson_proof_by_self_reference`
