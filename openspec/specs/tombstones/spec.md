# Tombstones

## Purpose

A **tombstone** is a deliberate, persistent mark of *something that was real and is retired*. Retired is not the same as deleted. A deleted identifier is forgotten; a tombstoned identifier is *still there, still readable, still reachable* — visibly inert.

Tombstones exist so this repo's history remains legible forever. A reader encountering an old lesson slug in a 2025 commit message, a `@trace` in an archival fork, or a strip caption cached on a mirror, must be able to follow that identifier to *something* — even if the something is just: *"that used to mean X; it no longer does, because Y; here is where it lives now."*

Every ledger in this project uses tombstones with identical semantics. This spec is the canonical definition. `licensing/spec.md` (R##), `meta-examples/spec.md` (ME##), `tlatoāni-spelling/spec.md` (TB##), `lessons/spec.md` (lesson IDs), `concept-curriculum/spec.md` (Cxx) all cite this contract.

## Why tombstones teach

The word "tombstone" is a gate. A curious reader sees it in a spec, wonders what the project is doing with such a grave word, and by reading this spec discovers that **identifiers in software are not free**. Reusing names causes ambiguity. Renaming without tombstoning causes dangling references. Deleting without tombstoning causes silent forgetting — the worst of the three, because nobody notices.

Pulling the thread:

> tombstone → never-reuse → why never-reuse? → because someone still uses the old ID → because `@trace` annotations point at old IDs → because `@trace` is how past work stays attached to evolving specs → **that is why `@trace` exists**.

The tombstone is a reader's on-ramp to the whole traceability thesis. Pull the thread of "grave marker in a code repo" and you end up holding the project's central mechanism.

This is the universal CRDT operation for "retire without conflict" — see `@Lesson S1-500` (edits-that-reconcile). Two branches can independently tombstone an identifier and converge; two branches can independently reference a tombstoned identifier and converge; a tombstone can accrete later context without ever being rewritten. Tombstoning is the graceful-retirement primitive the whole comic teaches about, enacted on the substrate of the project's own identifier ledgers.

## Core invariants

Every ledger in this project MUST honor these.

| # | Invariant | Why |
|---|---|---|
| T01 | **Never-reuse.** A tombstoned identifier is forever. New entries MUST use fresh IDs. | Reusing an ID silently repoints every historical reference to a new meaning. |
| T02 | **Archival legibility.** A tombstone MUST carry enough information that a reader encountering the old ID in a past artefact can understand what happened. | The point of retiring-without-deleting is preserving future readability. |
| T03 | **Monotonic.** Once tombstoned, the tombstone is never removed. It MAY be expanded with later context. | Removing a tombstone is a worse forgetting than the original delete would have been. |
| T04 | **Observable in Calmecac.** The viewer renders tombstones as grey, reachable entries — not hidden, not active. | Hiding tombstones defeats T02. See `calmecac/spec.md` (Wave 3). |

These are not advisory. An unhonored invariant is a spec bug.

## Anatomy of a tombstone

Every tombstone SHOULD carry these fields. Ledgers render them differently (row, inline note, struck-through cell) but the content is the same.

| Field | Meaning | Example |
|---|---|---|
| **ID** | The retired identifier. Usually struck through: `~~ID~~`. | `~~ME10~~`, `~~lesson_loop_closes~~`, `~~S1-1000-shape-has-meaning~~` |
| **Retirement date** | ISO 8601 date. Lamport-ticked by the commit that retired the entry. | `2026-04-22` |
| **Reason** | Why retired. Author's decision, renumbering event, duplicate, deprecated mechanism. | *"renumbered for Season 1 sequential convergence"* |
| **Successor** | The new ID if replaced; `null` for genuine declines. | `S1-1000-dashboards-must-add-observability`, or `null` |
| **Citation** | Optional short phrase the author wants future readers to carry. | *"retired as a teachable candidate; the tombstone itself is on-brand."* |

A `null` successor is valid. `ME10` was a candidate observation the author declined — nothing replaced it, and the tombstone itself teaches the CRDT mechanic by existing. That is a legitimate outcome.

## Where tombstones live in this project

Canonical cross-reference. Each ledger maintains its own tombstones; this table names them.

| Ledger | Spec | Tombstone mechanism | Currently tombstoned |
|---|---|---|---|
| Licensing rules | `licensing/spec.md` | R## row struck through; original license preserved | *(none yet)* |
| Meta-examples | `meta-examples/spec.md` | `~~ME##~~` row with tombstone note | `ME10` (exit-0-is-not-success observation; author declined) |
| Teachable breaks | `tlatoāni-spelling/spec.md` | `TB## (tombstone)` row; ASCII form struck through | *(none yet)* |
| Lesson full-slugs | `lessons/spec.md` | Migration table + tombstone marker | 14 legacy `lesson_*_*` slugs (Wave 1→Wave 2 restructure); 6 more imminent from the S1-950 renumbering sweep (S1-950, S1-1000-shape-has-meaning, S1-1100-meaning-is-operable, S1-1200-loop-closes, S1-1300-monotonic-convergence, S1-1400-proof-by-self-reference — all with successors) |
| Concepts | `concept-curriculum/spec.md` | Row retired with note; new ID on new row | `C09.5` (retirement pending the sequential-Season-1 decision) |

A reader can start at any ledger in this table and follow the tombstone mechanic back to this spec — the contract is the same everywhere.

## Renumbering as a tombstone event

Renumbering is the most visible tombstone-generator. When a set of identifiers shifts — Season 1's 9.5/10/11/12/13/14 renumber into 950/1000/1100/1200/1300/1400 — the OLD full-slugs tombstone; the NEW full-slugs are fresh.

**The load-bearing rule: identity is on the full `Sn-NNN-slug`, not the number alone.** Number-reuse across different full-slugs is legitimate. Worked example:

| Old (tombstoned) | New (fresh) | Same number? | Same identity? |
|---|---|---|---|
| `S1-1000-shape-has-meaning` | `S1-1000-dashboards-must-add-observability` | yes — `1000` | **no** — different full slug, different lesson, different spec directory |

The slug is the discriminant. `S1-1000-shape-has-meaning` is forever the tombstone of what lived at position 1000 before the renumber; `S1-1000-dashboards-must-add-observability` is a wholly new identifier whose position happens to be 1000. A `@trace @Lesson S1-1000` in a pre-renumber commit resolves through the tombstone to the old meaning; a `@Lesson S1-1000` in a post-renumber commit resolves directly to the new lesson. Both are correct. The tombstone is the pivot.

Never reuse across *identical* full-slugs. Never number-collide across *different* full-slugs unless the tombstone migration table makes the shift legible.

## Tombstones in Calmecac

The Calmecac viewer (`calmecac/spec.md`, Wave 3) is parallel work — this section describes the contract without coupling to implementation.

- Tombstones MUST render as visible-but-retired entries. Usually greyed, struck-through, or explicitly labelled "retired."
- Hovering or clicking a tombstone reveals the successor (if any), the retirement date, and the reason.
- Tombstones are **never hidden.** Hiding would defeat T02 (archival legibility).
- A reader browsing a lesson-coverage list, a rule table, or a meta-example ledger sees the tombstones alongside active entries — and can follow either.

The viewer is the reader's default path into this spec. A reader sees a grey entry, wonders why it is grey, clicks, and lands on this document.

## How to create a tombstone

Operational procedure. Same steps for every ledger.

1. **Identify the ledger** containing the entry (licensing R##, meta-examples ME##, teachable-breaks TB##, lessons slug, concept Cxx).
2. **Add a tombstone marker.** Strike through the ID (`~~ID~~`), or add an explicit tombstone row, whichever convention the ledger already uses.
3. **Fill the anatomy fields** — ID, retirement date, reason, successor (or `null`), optional citation.
4. **Add the entry to the ledger's migration or tombstone table** if one exists. Otherwise inline.
5. **Cross-reference in the commit message** with `@trace spec:tombstones` plus the ledger's own trace (e.g. `@trace spec:tombstones, spec:lessons`).
6. **Do not delete or rename the old line elsewhere in the repo.** Tombstones stay visible. Convergence happens through accretion, not erasure.

Per `@Lesson S1-400` (discrete-time), the commit is the Lamport tick that makes the retirement real. Before the commit, the ID is merely stale; after the commit, it is tombstoned.

## How to retire an orphan

A trickier case. If an identifier was introduced by mistake and has no referent anywhere outside its own introduction commit, the author MAY tombstone it inline in a single edit, without a migration table row.

Reserve this sparingly. The default is to keep even short-lived identifiers as full tombstones rather than hide-and-rewrite. Two edits to the same record converge through accretion (`@Lesson S1-500`); an edit that erases a prior record is not a CRDT-shaped move.

When in doubt: tombstone, don't hide.

## Relationship to `@trace`

This is the thread the curious reader is pulling.

- Tombstones exist *because* `@trace` annotations in code, commits, strip captions, caches, and archival forks point at identifiers.
- Without tombstones, renumbering would break every `@trace` silently. A reader grepping `@Lesson S1-1000` in a 2026 archive would find nothing, or worse, the *wrong* lesson.
- With tombstones, old `@trace` references are still resolvable. They land on a tombstone, which says: *"that ID is retired; here is where it lives now."*
- This is the exact same reason `@trace` itself exists: attach past work to evolving specs so neither loses the other. See `trace-plate/spec.md`. Tombstones are `@trace`'s graceful-retirement mechanism.

Curious reader clicks "tombstone" → reads this spec → understands why `@trace` matters → returns to the comic richer. Teach by example, click by click. That is the pedagogy of this project in a single interaction.

Catalogued as a candidate meta-example (author discretion): *"Tombstone-as-teaching-gate"* — demonstrates C05, C08, C12 in the identifier layer. Flagged, not canonized.

## Trace

`@trace spec:tombstones, spec:lessons, spec:meta-examples, spec:licensing, spec:tlatoāni-spelling, spec:concept-curriculum`
`@Lesson S1-500` *(edits-that-reconcile — tombstoning is the universal CRDT operation for retirement)*
