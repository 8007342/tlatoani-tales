# Tlatoāni — Spelling

## Purpose

The word is **Tlatoāni**. With the macron. Everywhere UTF-8 reaches.

Author's rule, verbatim: *"If you don't support multibyte encodings by now it's kinda your fault."*

This is a **convergence spec**. Correct spelling is an invariant the project climbs toward; the current state of the tree is not-yet-convergent (earlier commits contain plain `Tlatoani`, some will linger for a while); each edit is a monotonic tick in that direction. Like `licensing/spec.md` and `meta-examples/spec.md`, the shape of this spec is itself the lesson: a CRDT-flavoured ledger whose IDs never reuse, and whose history is readable as a Lamport clock.

## Invariants

Every surface below MUST carry the macron form **Tlatoāni** unless it appears in the teachable-breaks ledger as a catalogued `TB##`.

| Surface | Rule | Example |
|---|---|---|
| Spec body prose (`openspec/specs/**/*.md`) | Always macron | *"Tlatoāni looks at Covi…"* |
| Spec directory names | Always macron | `openspec/specs/tlatoāni-spelling/` |
| Spec filenames | Always macron when the name carries the word | this file's path |
| Code comments | Always macron | `# Tlatoāni's crown is gold` |
| `@trace spec:<name>` values | Always macron when the spec slug carries the word | `@trace spec:tlatoāni-spelling` |
| `@Lesson <slug>` values | Always macron when the lesson slug carries the word | future slugs TBD |
| Commit messages | Always macron | `docs: Tlatoāni plate placement in TT 04/15` |
| Strip speech bubbles / captions / titles | Always macron | *"I am Tlatoāni."* |
| Rendered plates (episode + trace) | Always macron | `Tlatoāni Tales 07/15` |
| Character reference sheet labels | Always macron | `characters/tlatoāni/README.md` |
| ASCII-art banners (hand-rolled) | Always macron | the README banner at repo root |
| Local directory names on author's machine | Always macron | `~/src/tlatoāni-tales/` |
| Shell scripts — echoed strings, heredocs | Always macron | `echo "Tlatoāni Tales build starting…"` |
| `output/` filenames | Always macron | `output/tt-04-tlatoāni-covi.png` |

Rule of thumb: if the surface will accept a UTF-8 byte sequence without eating it, the macron form is mandatory. No exceptions inside the invariant set.

## Known teachable breaks (canonical ledger)

A *teachable break* is a spot where an external system's identifier grammar forbids the macron. These are not ASCII licenses — they are **catalogued failures of the surrounding ecosystem** and will become Season-2 curriculum material on encoding discipline, identifier systems, and immutable-OS boundaries. IDs never reuse (same CRDT mechanic as `licensing/spec.md` R## and `meta-examples/spec.md` ME##).

| ID | Surface | ASCII form (forced) | Why it breaks | Lesson it seeds |
|---|---|---|---|---|
| TB01 | GitHub repository | `github.com/8007342/tlatoani-tales` | GitHub repo slug grammar is ASCII-LDH only (letters, digits, hyphen — no Unicode) | Identifier systems: who owns the namespace decides its encoding. The repo URL is a foreign passport — bilateral, not bilingual. |
| TB02 | Domain names | `tlatoani-tales.com`, `www.tlatoani-tales.com`, `calmecac.tlatoani-tales.com` | DNS labels are ASCII-LDH; IDN/punycode (`xn--…`) is technically allowed but visually hostile and mail-fragile | International identifiers and trade-offs: Unicode all the way down is a lie at the network layer. Punycode exists and is ugly for a reason. |
| TB03 | Podman / toolbox container name | `tlatoani-tales` | Podman's container-name regex is ASCII-only (`[a-zA-Z0-9][a-zA-Z0-9_.-]*`) | Season-2 lesson on OS-level encoding boundaries: the kernel, cgroup, and container runtime speak older alphabets than the editor. |
| TB04 | Figlet ASCII-art banner (certain fonts) | banner without `ā` | Many figlet fonts (`standard`, `slant`, `small`) have no glyph for `ā` and silently drop or substitute | Prefer a **handwritten ASCII banner that includes `ā`** over a figlet banner that drops it. Tool capability ≠ licence to degrade. (The README banner at repo root is an example of this preference.) |

Adding a break:

1. Claim the next `TB##`.
2. Name the surface; show the forced ASCII form; cite the grammar that forbids the macron; write the one-line lesson hook.
3. Commit with `@trace spec:tlatoāni-spelling` and the `TB##`.
4. Never use a `TB##` entry as a carte-blanche to spell ASCII elsewhere — the break is scoped to exactly the surface listed.

Retiring a break:

1. Rename the row to `TB## (tombstone)`.
2. Strike through the ASCII form; keep the row legible for archival readers.
3. If a replacement exists (e.g. a platform later accepts Unicode identifiers), add a new rule with a fresh ID. Never reuse.

## Enforcement

Three stages, converging:

| Stage | Mechanism | Status |
|---|---|---|
| **Today** | Author diligence + PR review. When encountering `Tlatoani` (no macron) in material being edited, silently correct it. | Active. |
| **Near-future** | Pre-commit lint: grep `\bTlatoani\b` (no macron) in added diff lines. Fail the commit unless the hit path matches a catalogued `TB##` allow-pattern (`README.md`'s quoted URLs, `compose.yaml` container name, etc.). | Planned. |
| **Far-future** | The `visual-qa-loop` VLM checks rendered panels: titles, speech bubbles, episode plate, trace plate. Any panel whose text reads `Tlatoani` (no macron) fails QA and is re-rolled. Observability of the spelling invariant itself. | Planned — hooks into `visual-qa-loop/spec.md`. |

Enforcement is deliberately coarse today. The lint and the VLM check are convergence steps — each step strictly refines the previous, no rule is invalidated.

## Why this matters for the project

Tlatoāni Tales teaches *monotonic convergence*: specs are the contract, code catches up, each commit is a Lamport tick. The spelling of the protagonist's name is the smallest possible instance of that thesis enacted on the substrate.

The git history contains plain `Tlatoani` in older commits. This is not embarrassment — it is **evidence**. A reader running:

```
git log -S "Tlatoani" -- ':!openspec/specs/tlatoāni-spelling/spec.md'
```

can watch the correction propagate across the tree commit by commit, the same way `licensing/spec.md` will one day propagate SPDX headers and `meta-examples/spec.md` will keep accreting load-bearing `ME##` rows. The project's own spelling history is Lamport-ordered evidence of its own thesis. If we ever achieve a fully-converged tree (macron everywhere except catalogued TBs), that commit will be citable in TT 15/15 as the closing demonstration: *this repo converged to its own spec, in front of you, while you were reading it.*

Catalogued as a meta-example candidate for the next free `ME##` slot (author's discretion): "Repo-wide spelling convergence" — demonstrates C05, C06, C07.

## Trace

`@trace spec:tlatoāni-spelling`
`@Lesson S1-500` — kinship: an edit that propagates through history without conflict, the same CRDT-flavoured correction mechanism this spec formalises for one specific word.
