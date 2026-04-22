# Lessons

## Purpose

**Registry and index** of all published lessons. The humane layer on top of the technical trace namespace — each lesson is a warm, memorable phrase the reader carries out of a strip, readable without clicking anything.

This spec is **not** the per-lesson body. It is the *index*. Each published lesson has its own spec at:

```
openspec/specs/lessons/<Sn-NNN-slug>/spec.md
```

Per-lesson specs follow the seven-field contract defined in `lesson-driven-development/spec.md` (Abstract, Position, References, Script, Joke, Punchline, Aha moment, plus Trace). This registry points at them.

Two layers of citation continue to run through the project:

| Layer | Format | For |
|---|---|---|
| Technical trace | `@trace spec:<name>` | "Which contract governs this code?" — grep-friendly, precise |
| Lesson trace | `@Lesson <Sn-NNN>` | "Which teaching is this in service of?" — human-readable, reader-friendly |

Both appear on every strip's left plate. Non-clickers read the lesson and get the point. Clickers follow the URL into the CRDT of all wisdom gathered under that lesson.

## Canonical naming

Lesson IDs use the `Sn-NNN-slug` form defined in `seasons/spec.md`:

- `Sn` — season integer (`S1`, `S2`, …).
- `NNN` — lesson index, stepped by **100** starting at **100** (gaps absorb insertions without renumbering).
- `slug` — the lesson's kebab-case phrase.

The directory under `openspec/specs/lessons/` uses the full `Sn-NNN-slug` form; the `@Lesson` citation form uses only the short `Sn-NNN` prefix in code (full form on plates and in captions, per `trace-plate/spec.md`).

**ID stability.** Once a lesson ID is published (merged to main), **it never reuses**. A retired ID becomes a tombstone.

## Tombstoned old-slug form

The old `lesson_<snake_case>` slug form is **tombstoned** as of 2026-04-22. It is not a valid citation anywhere in the repo from that date forward. The commit history will show every `@Lesson lesson_*` reference getting replaced with its `@Lesson Sn-NNN` equivalent — the migration is itself a monotonic convergence event this project teaches about (see `meta-examples/spec.md` ME12 and the S1-500 / S1-1300 lessons).

### Migration table (old → new)

| Old slug (tombstoned 2026-04-22) | New ID |
|---|---|
| `lesson_volatile_is_dangerous` | `S1-100-volatile-is-dangerous` |
| `lesson_save_means_findable` | `S1-200-save-means-findable` |
| `lesson_memory_lives_in_history` | `S1-300-memory-lives-in-history` |
| `lesson_discrete_time` | `S1-400-discrete-time` |
| `lesson_edits_that_reconcile` | `S1-500-edits-that-reconcile` |
| `lesson_ask_in_writing` | `S1-600-ask-in-writing` |
| `lesson_loops_need_aim` | `S1-700-loops-need-aim` |
| `lesson_see_the_now` | `S1-800-see-the-now` |
| `lesson_logs_are_ingredients` | `S1-900-logs-are-ingredients` |
| *(no legacy slug — new lesson)* | `S1-1000-dashboards-must-add-observability` |
| `lesson_shape_has_meaning` | `S1-1100-shape-has-meaning` |
| `lesson_meaning_is_operable` | `S1-1200-meaning-is-operable` |
| `lesson_loop_closes` | `S1-1300-loop-closes` |
| `lesson_monotonic_convergence` | `S1-1400-monotonic-convergence` |
| `lesson_proof_by_self_reference` | `S1-1500-proof-by-self-reference` |
| `S1-950-dashboards-must-add-observability` | `S1-1000-dashboards-must-add-observability` — tombstoned 2026-04-22 — renumbered for Season 1 sequential convergence |
| `S1-1000-shape-has-meaning` | `S1-1100-shape-has-meaning` — tombstoned 2026-04-22 — renumbered for Season 1 sequential convergence |
| `S1-1100-meaning-is-operable` | `S1-1200-meaning-is-operable` — tombstoned 2026-04-22 — renumbered for Season 1 sequential convergence |
| `S1-1200-loop-closes` | `S1-1300-loop-closes` — tombstoned 2026-04-22 — renumbered for Season 1 sequential convergence |
| `S1-1300-monotonic-convergence` | `S1-1400-monotonic-convergence` — tombstoned 2026-04-22 — renumbered for Season 1 sequential convergence |
| `S1-1400-proof-by-self-reference` | `S1-1500-proof-by-self-reference` — tombstoned 2026-04-22 — renumbered for Season 1 sequential convergence |

Every old slug is tombstoned — new code, specs, proposals, captions, and commit messages MUST use the `Sn-NNN` form. Archival forks that still read `lesson_*` can resolve to the tombstoned row for their last canonical meaning.

**Never-reuse discipline.** The old slug strings above will not be reassigned to any future lesson, ever. The tombstone is the ID's afterlife. The six `S1-*` full-slug tombstones below the legacy `lesson_*` rows record the April 2026 renumbering: Season 1 moved from stepped-hundreds-with-decimal-insertion (`S1-950`) to strict sequential hundreds (`S1-100` … `S1-1500`) because sequential thought is itself Season 1's pedagogy. The decimal-insertion discipline is reserved for a future lesson that teaches what decimal insertion *means* — it does not exist yet.

## Season 1 registry

Thesis: *"From volatile context to monotonic convergence."*

| ID | Display | Takeaway | Primary TT strip | Coverage (per-lesson spec) |
|---|---|---|---|---|
| `S1-100-volatile-is-dangerous` | Volatile is dangerous | Starting fresh loses everything that mattered. | TT 01/15 | [`openspec/specs/lessons/S1-100-volatile-is-dangerous/spec.md`](S1-100-volatile-is-dangerous/spec.md) |
| `S1-200-save-means-findable` | Save means findable | Copy-pasting isn't saving. | TT 02/15 | [`openspec/specs/lessons/S1-200-save-means-findable/spec.md`](S1-200-save-means-findable/spec.md) |
| `S1-300-memory-lives-in-history` | Memory lives in history | Git remembers what you forgot — if you let it. | TT 03/15 | [`openspec/specs/lessons/S1-300-memory-lives-in-history/spec.md`](S1-300-memory-lives-in-history/spec.md) |
| `S1-400-discrete-time` | Discrete time | Every commit answers what was true before and after. | TT 04/15 | [`openspec/specs/lessons/S1-400-discrete-time/spec.md`](S1-400-discrete-time/spec.md) |
| `S1-500-edits-that-reconcile` | Edits that reconcile | Two edits to the same rule? The spec reconciles, not you. | TT 05/15 | [`openspec/specs/lessons/S1-500-edits-that-reconcile/spec.md`](S1-500-edits-that-reconcile/spec.md) |
| `S1-600-ask-in-writing` | Ask in writing | Ask for what you want in writing. | TT 06/15 | [`openspec/specs/lessons/S1-600-ask-in-writing/spec.md`](S1-600-ask-in-writing/spec.md) |
| `S1-700-loops-need-aim` | Loops need aim | You've been iterating. You just weren't aiming. | TT 07/15 | [`openspec/specs/lessons/S1-700-loops-need-aim/spec.md`](S1-700-loops-need-aim/spec.md) |
| `S1-800-see-the-now` | See the now | Tests check yesterday. Observability shows today. | TT 08/15 | [`openspec/specs/lessons/S1-800-see-the-now/spec.md`](S1-800-see-the-now/spec.md) |
| `S1-900-logs-are-ingredients` | Logs are ingredients | Raw logs aren't truth. They're material for it. | TT 09/15 | [`openspec/specs/lessons/S1-900-logs-are-ingredients/spec.md`](S1-900-logs-are-ingredients/spec.md) |
| `S1-1000-dashboards-must-add-observability` | Dashboards must add observability | Real dashboards add observability, not just display it. | TT 10/15 | [`openspec/specs/lessons/S1-1000-dashboards-must-add-observability/spec.md`](S1-1000-dashboards-must-add-observability/spec.md) |
| `S1-1100-shape-has-meaning` | Shape has meaning | A number across time has a shape; that shape is the truth. | TT 11/15 | [`openspec/specs/lessons/S1-1100-shape-has-meaning/spec.md`](S1-1100-shape-has-meaning/spec.md) |
| `S1-1200-meaning-is-operable` | Meaning is operable | You can add and compare shapes. | TT 12/15 | [`openspec/specs/lessons/S1-1200-meaning-is-operable/spec.md`](S1-1200-meaning-is-operable/spec.md) |
| `S1-1300-loop-closes` | The loop closes | Last iteration's meaning is this iteration's input. | TT 13/15 | [`openspec/specs/lessons/S1-1300-loop-closes/spec.md`](S1-1300-loop-closes/spec.md) |
| `S1-1400-monotonic-convergence` | Monotonic convergence | Ask your AI for it. You already earned it. | TT 14/15 | [`openspec/specs/lessons/S1-1400-monotonic-convergence/spec.md`](S1-1400-monotonic-convergence/spec.md) |
| `S1-1500-proof-by-self-reference` | Proof by self-reference | This comic was made that way. | TT 15/15 | [`openspec/specs/lessons/S1-1500-proof-by-self-reference/spec.md`](S1-1500-proof-by-self-reference/spec.md) |

These IDs are author-curated. **Do not add new lessons** without explicit author approval — see the curation rule in `feedback_tlatoani_tales_curation.md`.

## `@Lesson` citation forms

| Where | Form | Example |
|---|---|---|
| Code comments, commit messages, grep-friendly references | **Short** — `@Lesson Sn-NNN` | `@Lesson S1-100` |
| Strip trace plates, captions, METADATA.json | **Full** — `[@Lesson Sn-NNN — Display Name]` | `[@Lesson S1-100 — Volatile is dangerous]` |

The short form is cheap to type, greppable, stable across display-name edits. The full form is what the reader sees — in-frame on the plate, under the image on social posts. See `trace-plate/spec.md` for plate rendering.

## URL forms

| Form | Template | Surfaces |
|---|---|---|
| Lesson search | `https://github.com/8007342/tlatoani-tales/search?q=%40Lesson+Sn-NNN&type=code` | Every file/commit/caption citing this lesson |
| Per-lesson spec | `https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/lessons/Sn-NNN-slug/spec.md` | The full seven-field spec for this lesson |
| Registry (this file) | `https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/lessons/spec.md` | The index + migration table |

Search URLs are preferred in captions — they surface the whole trace/lesson network with one click. The `Sn-NNN` prefix is specific enough that search matches don't collide across seasons.

## Coverage as CRDT

Coverage (which specs, meta-examples, and strips cite a lesson) now lives inside each per-lesson spec's `## References in this project` section, not in this registry. The CRDT discipline is unchanged:

- **Commutative**: coverage lists are sets; order-independent union.
- **Associative**: merging coverage-additions from parallel branches is well-defined.
- **Idempotent**: re-adding a spec or ME## to a lesson's references is a no-op.
- **Monotonic**: coverage grows. Removal requires tombstoning the specific entry (rare — e.g. a spec was renamed).

The lesson spec is the aggregation node — a reader who clicks the plate lands there and sees every spec, meta-example, and strip that touches this teaching.

## Per-strip declaration

In each strip's `proposal.md`:

```yaml
lesson:      S1-NNN-slug       # must exist in this registry
reinforces:  []                # optional: other lesson IDs this strip echoes
trace_spec:  <spec-name>       # the governing OpenSpec for this strip
```

The orchestrator reads these, composites the two-line left plate, and emits `METADATA.json` with both trace URLs.

## Propagation note

The tombstoning of the old `lesson_<snake>` form is a real monotonic convergence event. Across the repo, every `@Lesson lesson_*` reference — in specs, code comments, commit messages, strip proposals, caption drafts — will be replaced with its `@Lesson Sn-NNN` equivalent in the commits that accompany this restructuring. The commit history is the ledger of the migration; grep across it is the proof of convergence. Live C05, C07, C12 — the CRDT-edit-and-converge discipline applied to the lesson-naming layer itself.

See `meta-examples/spec.md` ME12 (first propagation event: adding the trace plate) for the pattern template; this registry migration is its sibling.

## Meta-observation

This spec is itself an instance of the lesson it was written to describe. A reader who asks *"which spec teaches me observability?"* finds `visual-qa-loop`. A reader who asks *"which teaching is this comic delivering?"* finds a per-lesson spec under `lessons/Sn-NNN-slug/`. The lesson layer is the observability-of-the-observability — it tells the reader not what the code is doing but what the artefact is trying to teach. See `meta-examples/spec.md` ME13.

## Trace

`@trace spec:lessons`
`@Lesson S1-1500`
