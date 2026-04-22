# Trace Plate

## Purpose

Every strip carries a **two-line left plate** mirroring the episode plate at the bottom-right. The left plate is a dual citation:

- Line 1: `[@Lesson <slug>]` — the humane, readable teaching this strip delivers.
- Line 2: `[@trace spec:<name>]` — the technical contract governing that teaching.

Non-clickers read the lesson phrase and get the point. Clickers follow the URL into the CRDT of all wisdom aggregated under that lesson (see `lessons/spec.md`). The plate makes observability a *drawn artefact*, not a tooling convention.

Two layers of observability run through the project:

| Layer | Marker | What it observes |
|---|---|---|
| Technical trace | `@trace spec:<name>` | Code ↔ spec linkage — which contract governs which implementation |
| Lesson trace | `@Lesson <slug>` | Spec ↔ teaching linkage — which teaching a spec is in service of |

The plate makes both visible in-frame. `@Lesson` is the observability *of the* `@trace` layer. Teach by example, literally in-panel.

## Motivation

- Converts casual readers into deep readers. The lesson phrase lands alone; the URL opens the whole wisdom graph.
- **Non-technical readers** read the lesson line ("volatile is dangerous") and walk away wiser with zero clicks.
- **Technical readers** follow the trace line to the governing spec, then to every file that cites it.
- Forces authors to pick exactly **one primary lesson + one primary spec** per strip — itself a convergent discipline. No plate = strip doesn't ship.

## Layout

- **Position**: bottom-**left**, symmetric to the episode plate at bottom-right.
- **Overlap**: all of panel 1 and ~12% of panel 2 (mirror of the episode plate's overlap).
- **Shape**: two-line stack on one scroll/banner motif. Height adjusts to fit both lines; width accommodates the longest slug/spec name.
- **Styling**: cream scroll, dark ink, same typeface family as the episode plate. Line 1 (`@Lesson`) is primary — slightly bolder or larger than line 2 (`@trace`), reflecting that readers should see the lesson first.
- **Text**:
  ```
  [@Lesson lesson_volatile_is_dangerous]
  [@trace spec:concept-curriculum]
  ```
- **Text wrapping**: both lines may wrap for unusually long names. Prefer wrapping the `@trace` line first (secondary).
- **Legibility constraint**: MUST NOT obscure character faces or key action. If a natural plate-sized region covers a key element, shrink to min 40% of episode-plate area (never below). QA checks `plate.*-legible` enforce this.

## Selection rule (per strip)

Each strip declares one primary lesson and one primary spec in its `proposal.md`:

```yaml
lesson:      lesson_<slug>       # must exist in lessons/spec.md registry
reinforces:  []                  # optional: other lesson slugs this strip echoes
trace_spec:  <spec-name>         # the governing OpenSpec for this strip
```

Selection priority:

1. The lesson whose **primary strip** is this one in `lessons/spec.md` (obvious case).
2. For the spec: the one governing the **visual prop** introduced/highlighted, OR the **concept** being taught (usually `concept-curriculum`), OR `meta-examples` if the joke is meta about the project itself.
3. The declared lesson and spec MUST be consistent — the spec SHOULD appear in the lesson's coverage list. QA check `plate.lesson-spec-aligned` enforces this.

## Initial strip-to-trace mapping

| Strip | Lesson | Trace spec | Why |
|---|---|---|---|
| TT #01 — context overflow | `lesson_volatile_is_dangerous` | `concept-curriculum` | C01 is the root; curriculum is the entry point |
| TT #02 — naive save | `lesson_save_means_findable` | `concept-curriculum` | C02 |
| TT #03 — git = memory | `lesson_memory_lives_in_history` | `meta-examples` | git-as-Lamport (ME01) is the substrate example |
| TT #04 — hourglasses | `lesson_discrete_time` | `meta-examples` | same substrate, named by symbol (ME01) |
| TT #05 — scroll reconciles | `lesson_edits_that_reconcile` | `licensing` | licensing IS a live CRDT demo — reinforce by citation |
| TT #06 — sealed scroll | `lesson_ask_in_writing` | `concept-curriculum` | C06 is the pivot |
| TT #07 — treadmill → staircase | `lesson_loops_need_aim` | `concept-curriculum` | C07 |
| TT #08 — telescope | `lesson_see_the_now` | `visual-qa-loop` | first observability strip; cite the live loop |
| TT #09 — dashboard shape | `lesson_logs_are_ingredients` | `visual-qa-loop` | ditto |
| TT #10 — curves | `lesson_shape_has_meaning` | `visual-qa-loop` | ditto |
| TT #11 — operable meaning | `lesson_meaning_is_operable` | `visual-qa-loop` | ditto |
| TT #12 — loop closes | `lesson_loop_closes` | `visual-qa-loop` | the payoff strip for the observability arc |
| TT #13 — BOOM | `lesson_monotonic_convergence` | `meta-examples` | the whole ledger is the evidence |
| TT #14 — proof by self-reference | `lesson_proof_by_self_reference` | `meta-examples` | literally ME-everything |

Mapping confirmed in each strip's `proposal.md`. Lessons registry is authoritative — see `lessons/spec.md`.

## Metadata emission

Each strip emits `METADATA.json` alongside its PNG:

```jsonc
{
  "strip":              "TT #NN",
  "title":              "<short strip name>",

  "lesson":             "<slug>",
  "lesson_display":     "<display name from lessons/spec.md>",
  "lesson_search_url":  "https://github.com/8007342/tlatoani-tales/search?q=%40Lesson+<slug>&type=code",
  "lesson_spec_url":    "https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/lessons/spec.md",

  "trace_spec":         "<name>",
  "trace_search_url":   "https://github.com/8007342/tlatoani-tales/search?q=%40trace+spec%3A<name>&type=code",
  "trace_spec_url":     "https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/<name>/spec.md",

  "concepts_taught":    ["Cxx"],
  "concepts_assumed":   ["Cxx"],
  "reinforces_lessons": [],

  "alt_text":           "<accessible description of all three panels>",
  "caption":            "Tlatoāni Tales #NN — <title> — @Lesson <slug> / @trace spec:<name>"
}
```

Publishers include both `lesson_search_url` and `trace_search_url` in post bodies. Two doors, same wisdom graph — casual readers take the lesson door, technical readers take the trace door.

## URL forms

| Form | Template | Surfaces |
|---|---|---|
| Lesson search | `https://github.com/8007342/tlatoani-tales/search?q=%40Lesson+<slug>&type=code` | Every file/commit/caption citing this lesson |
| Lesson registry | `https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/lessons/spec.md` | The full coverage graph (specs, MEs, strips) for the lesson |
| Trace search | `https://github.com/8007342/tlatoani-tales/search?q=%40trace+spec%3A<name>&type=code` | Every implementer/citer of the spec |
| Spec direct | `https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/<name>/spec.md` | The spec file itself |

Search URLs are preferred in captions — they surface the whole trace/lesson network with one click.

## Propagation cost

Adding this spec retroactively invalidates every strip's layout. Strip #01 (ChatGPT demo) predates the plate and will be re-rendered once LoRAs + models are ready. First real propagation event — see `meta-examples/spec.md` ME11 (trace plate as in-frame observability), ME12 (this propagation), ME13 (lesson-trace CRDT).

Repo is presently in a known-not-yet-convergent state; commit history will show the convergence. Live C07 + C12.

## QA integration

VLM checks (defined in `visual-qa-loop/spec.md`):

| Check | Enforces |
|---|---|
| `plate.trace-present` | A bottom-left plate exists with the `@trace` line |
| `plate.trace-legible` | `@trace` text parses |
| `plate.trace-content` | `@trace` text matches declared `trace_spec` in proposal.md |
| `plate.lesson-present` | The plate has the `@Lesson` line |
| `plate.lesson-legible` | `@Lesson` text parses |
| `plate.lesson-slug-valid` | Slug is in the `lessons/spec.md` registry |
| `plate.lesson-spec-aligned` | Declared spec appears in the lesson's coverage list |
| `plate.symmetry` | Both plates visually matched as a pair |

## Trace

`@trace spec:trace-plate, spec:style-bible, spec:visual-qa-loop, spec:lessons, spec:meta-examples`
`@Lesson lesson_see_the_now`
