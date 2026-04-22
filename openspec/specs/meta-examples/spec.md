# Meta-Examples

## Purpose

A running catalogue of places in this project where curriculum concepts (C01–C14) are instantiated **in the substrate** — not as comic content, but as structural decisions. Each meta-example is a convergence-in-weird-places moment we can cite in strip dialogue, in TT #14's proof-by-self-reference, and in the running commentary of the README.

When a reader objects *"but that only applies to code, not my real work"*, we point at this table. The project is a Rosetta stone: every piece of it is a different domain expressing the same core ideas.

## Rules

- A meta-example is only counted once it's **load-bearing** — real code/commits/specs depend on it. Toy examples don't count.
- Each entry cites the governing OpenSpec spec and the curriculum concept(s) it demonstrates.
- Entries receive stable IDs (`ME##`), tombstone semantics like `licensing/spec.md`.

## Ledger

| ID | Example | Demonstrates | Governing spec | Notes |
|---|---|---|---|---|
| ME01 | **Git history = Lamport clock** | C03, C04 | — (git itself) | Every commit is a monotonic tick. Timestamps + parent-pointers give total order without coordination. |
| ME02 | **OpenSpec delta-merge** | C05, C06 | `openspec/config.yaml` | Delta specs sync into main specs on archive. Parallel changes converge — spec is the contract, specs are the CRDT. |
| ME03 | **Dual-license rule table as CRDT** | C05 | `licensing/spec.md` | Pattern-scoped rules with stable IDs + tombstones. Merges are set-union; no rule conflicts because patterns partition the namespace. |
| ME04 | **License-coverage convergence (future REUSE)** | C06, C07 | `licensing/spec.md` (planned extension) | Start with coarse patterns (current state). Converge toward fine-grained SPDX headers per file (REUSE compliance). Same spec, finer grain over time — monotonic. |
| ME05 | **Content-addressed panel cache = G-Set** | C05 | (future: `panel-cache/spec.md`) | Each render's hash is a cell; re-running the orchestrator unions cells; identical hashes are idempotent. |
| ME06 | **Concept-curriculum DAG** | C04, C07 | `concept-curriculum/spec.md` | Strip proposals declare `Depends on: [Cxx]`. Topological order enforces prerequisite landing. Circular deps impossible by construction. |
| ME07 | **Lessons ledger stale-marking** | C06, C08 | `lessons-ledger/spec.md` | Author notes `Known-wobble` on a reader-assumption. Every strip that depends on that assumption is marked stale. Observability of the *reader*, not the code. |
| ME08 | **Visual QA feedback loop** | C08, C09, C10, C11, C12 | `visual-qa-loop/spec.md` | VLM critiques panel → drift score → reroll with derived addendum. Telemetry is the output of the previous iteration *and* the input of the next. The loop closes on itself. |
| ME09 | **LoRA-hash in panel cache key** | C05, C06 | `character-canon/spec.md` | Character identity is part of the content hash. Retraining the LoRA = new hash = automatic invalidation of every affected panel. Spec-to-code causality, measurable. |

## Adding a new example

1. Claim the next `ME##`.
2. Name the example with a hook that's tweetable (first column is a micro-title).
3. Cite the concept(s), cite the spec, write the one-line description.
4. Commit with `@trace spec:meta-examples`.
5. If the example suggests a strip punchline, note it on the relevant entry in `concept-curriculum/spec.md` strip mapping.

## Why this spec exists

The thesis of Tlatoāni Tales is that *monotonic convergence* is not a math incantation — it's a shape you can see in many places. This ledger is the empirical evidence. Without it, the finale (TT #13–#14) is a claim. With it, the finale is a citation.

## Trace

`@trace spec:meta-examples`
