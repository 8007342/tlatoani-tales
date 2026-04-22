# Licensing

## Purpose

Map every file in the repository to the license that covers it. This spec is itself a **CRDT**: each rule is a pattern-scoped cell, rules don't conflict because file patterns partition the namespace, and the rule set grows monotonically as new file types are introduced.

We license the project this way on purpose. Readers who think "monotonic convergence is fine for specs, but where else does it apply?" can look at how this very file evolved — it's a demonstration of the thesis of the comic. See TT #14.

## Invariant

Every committed file in the repo MUST match exactly one rule below. Unmatched files mean the spec is incomplete — add the rule, don't patch the file.

## Rules

| # | Pattern | License | Rationale |
|---|---|---|---|
| R01 | `scripts/**` | GPL-3.0-or-later | Functional tooling — copyleft |
| R02 | `**/*.sh` | GPL-3.0-or-later | Shell scripts anywhere |
| R03 | `**/*.py` | GPL-3.0-or-later | Python scripts (future orchestrator) |
| R04 | `**/*.md` | CC BY-SA 4.0 | Prose — specs, strip scripts, documentation |
| R05 | `**/*.png` | CC BY-SA 4.0 | Comic art and reference images |
| R06 | `**/*.jpg`, `**/*.jpeg`, `**/*.webp` | CC BY-SA 4.0 | Other raster art |
| R07 | `**/*.svg` | CC BY-SA 4.0 | Vector art (symbols, props) |
| R08 | `**/*.yaml`, `**/*.yml`, `**/*.toml`, `**/*.json` | GPL-3.0-or-later | Config files — considered functional |
| R09 | `LICENSE`, `LICENSE-ART` | *(verbatim upstream — see inside each file)* | License texts themselves are not relicensable |
| R10 | `.gitignore`, `.gitattributes` | CC0 | Git metadata — public domain-ish |
| R11 | `README.md` | CC BY-SA 4.0 | Matches R04; explicitly listed for readers skimming this table |
| R12 | `calmecac/**` | GPL-3.0-or-later | Calmecac PWA (HTML, JS, CSS, webmanifest, service worker) at any depth — functional UI, copyleft |
| R13 | `images/**/Containerfile` | GPL-3.0-or-later | Hardened container definitions — functional infrastructure |
| R14 | `**/*.lock`, `Cargo.lock` | GPL-3.0-or-later | Reproducibility artefacts for the Rust workspace |
| R15 | `**/.gitkeep` | CC0 | Git directory-marker convention |
| R16 | `crates/**/src/**/*.html` | GPL-3.0-or-later | Embedded HTML templates included via `include_str!` into Rust binaries |
| R17 | `assets/fonts/**/*.ttf`, `**.otf`, `**.woff`, `**.woff2` | SIL OFL 1.1 (see `LICENSES/OFL-1.1.txt`) | Font binaries — **not committed** (`.gitignore` excludes) but referenced from tt-compose. License text travels with the repo. |
| R18 | `LICENSES/**` | *(verbatim upstream — see file)* | Third-party license texts distributed alongside the repo. R09 still covers root-level `LICENSE` + `LICENSE-ART`; `LICENSES/` is the REUSE-style bucket for vendored licenses. |

## How the CRDT property plays out

- **Commutative**: rule order in the table is cosmetic. Earlier rules don't win over later rules; patterns must be disjoint.
- **Associative**: merging this spec from two parallel branches concatenates rule sets. Duplicates collapse because each rule has a stable ID (`R##`).
- **Idempotent**: re-adding an existing rule is a no-op.
- **Monotonic**: rule IDs never reuse. A removed rule becomes a tombstone entry instead of disappearing — readers of old strip archives can still find out which license their copy was under.

## How to add a rule

1. Assign the next unused `R##`.
2. Add the pattern + license + one-line rationale.
3. Commit with message referencing `@trace spec:licensing` and the rule ID.
4. If the rule covers files that existed before — they're retroactively covered by the added rule. No file moves, no content changes. Pure spec evolution.

## How to retire a rule

1. Rename the row to `R## (tombstone)`.
2. Strike through the pattern; keep the license column so archival forks remain legible.
3. Add a new rule with a fresh ID covering the intended replacement.
4. Never reuse a tombstoned ID.

## Conflict detection

If two rules match the same file, the spec is inconsistent and MUST be fixed before the next commit. Orchestrator-side lint (future): walk the tree, match each file against the table, flag 0-match or ≥2-match cases.

## Planned convergence — REUSE compliance

GitHub currently auto-detects only the file literally named `LICENSE` (GPL-3.0-or-later) and not `LICENSE-ART`. Machine-readable detection of the dual structure requires the **REUSE specification** (reuse.software): SPDX headers per source file + a top-level `LICENSES/` directory. Migration is monotonic:

1. *Current state (coarse)*: pattern-scoped rules in this table. (Today.)
2. *Intermediate*: top-level `LICENSES/` directory with canonical texts, preserved in parallel with `LICENSE` / `LICENSE-ART` for backwards compatibility.
3. *Final (fine-grained)*: SPDX short-form headers on every file (`SPDX-License-Identifier: GPL-3.0-or-later` etc.), `.reuse/dep5` for files that can't carry headers (binaries, generated art). GitHub and third-party tools read this automatically.

Each step strictly refines the previous — no rule is invalidated, only made more precise. This is C06 (specs are the contract) plus C07 (iteration with aim) expressed on the licensing layer itself. Catalogued in `meta-examples/spec.md` as ME04.

## Trace

`@trace spec:licensing, spec:meta-examples`
