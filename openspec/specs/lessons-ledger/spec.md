# Lessons Ledger

## Purpose

Tracks what the reader is *assumed to have internalized* by strip N. This is the running state of the reader's mental model — separate from what was *published*.

When writing strip 12 we consult this ledger: what can we assume the reader already gets? What metaphors can we call back to? What symbols are now fluent vocabulary?

When we realize a strip didn't land (reader retention test, confusion signal, our own re-read), we amend the ledger — and the dependent strips become stale, triggering re-renders.

## Ledger schema

Each entry:

```
## L-NNN — <short name>
- Learned in: TT #NN (primary), reinforced in: TT #MM, TT #OO
- Takeaway: <one sentence the reader would say aloud>
- Assumed-by-strip: TT #PP onward
- Evidence: <what in the strip made it stick>
- Known-wobble: <if readers report confusion, note here>
```

## Rules

- A ledger entry is only valid **after** the primary strip has shipped (rendered AND reviewed by the author).
- If a ledger entry gets a `Known-wobble`, every strip that lists this entry in its `Assumed-prior-knowledge` becomes stale. The orchestrator marks them for re-review.
- Adding a new callback to an old metaphor? Reinforce the ledger entry, don't duplicate.

## Convergence note

This ledger is the slow-moving truth about *the reader*, not about the comic. It's where "strip 12 shows that strip 6 and 7 missed a step" gets recorded — and from here the dependency graph updates the strip set.

## Initial state

Empty. Populated as strips ship.

## Trace

`@trace spec:lessons-ledger`
