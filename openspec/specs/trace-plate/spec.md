# Trace Plate

## Purpose

Every strip carries a small `[@trace spec:<name>]` plate at the **bottom-left**, mirroring the episode plate at the bottom-right. The trace plate is a visible, public citation — the governing OpenSpec behind the strip's joke. Readers who want more can click. The plate makes observability a *drawn artefact*, not just a tooling convention.

This is the canonical "teach by example" move: the strip that teaches *observability beats tests* itself ships observable evidence, in-frame, before the reader has even finished the last panel.

## Motivation

- Converts casual readers into deep readers: those who follow the plate find the repo, the spec, the commit history, and ultimately the thesis.
- Closes the loop between the comic and its own source — a reader can trace any joke to the governing contract in one click.
- Forces authors to choose a primary spec per strip, which is itself a convergent discipline. No trace plate = strip doesn't ship.

## Layout

- **Position**: bottom-**left**, symmetric to the episode plate at bottom-right.
- **Overlap**: all of panel 1 and ~12% of panel 2 (mirror of the episode plate's overlap on panels 3 + 2).
- **Styling**: cream scroll / banner motif, dark ink. Same typeface family and weight as the episode plate. This is deliberately identical-looking chrome — it reads as "the other nameplate" on first glance.
- **Text**: `[@trace spec:<name>]`. Brackets included to echo markdown/code syntax — readers with tech literacy recognize the gesture instantly.
- **Text wrapping**: if `<name>` is long (e.g. `visual-qa-loop`), split onto two lines:
  ```
  [@trace
   spec:visual-qa-loop]
  ```
- **Legibility constraint**: MUST NOT obscure character faces or key action. If a natural plate-sized region covers a key element, shrink plate to min 40% of episode-plate area (never below that). QA loop's `plate.legibility` check enforces this.

## Spec-selection rule (per strip)

Each strip picks **one** primary `@trace spec:<name>`. One is enough — the whole repo is reachable via a single click. Selection priority, highest first:

1. The spec that governs the **visual prop** introduced or highlighted in that strip (e.g. `symbol-dictionary` if the strip shows off a new symbol for the first time).
2. The spec that governs the **concept** being taught (usually `concept-curriculum`).
3. `meta-examples` if the joke is meta about the project itself.

Declared in each strip's `proposal.md` under `trace_plate:`. The orchestrator reads it, composites the plate, writes the URL into the metadata file.

## Initial strip-to-trace mapping

| Strip | Trace plate | Why |
|---|---|---|
| TT #01 — context overflow | `concept-curriculum` | C01 is the root lesson; curriculum is the entry point |
| TT #02 — naive save | `concept-curriculum` | C02 |
| TT #03 — git = memory | `meta-examples` (ME01) | git-as-Lamport is the substrate example |
| TT #04 — hourglasses | `meta-examples` (ME01) | same substrate, now named by symbol |
| TT #05 — scroll reconciles | `licensing` | licensing IS a live CRDT demo — reinforces by citation |
| TT #06 — sealed scroll | `concept-curriculum` | C06 is the pivot; keep curriculum in view |
| TT #07 — treadmill → staircase | `concept-curriculum` | C07 |
| TT #08 — telescope | `visual-qa-loop` | first observability strip; cite the live loop |
| TT #09 — dashboard shape | `visual-qa-loop` | ditto |
| TT #10 — curves | `visual-qa-loop` | ditto |
| TT #11 — operable meaning | `visual-qa-loop` | ditto |
| TT #12 — loop closes | `visual-qa-loop` | the payoff strip for the observability arc |
| TT #13 — BOOM | `meta-examples` | the whole ledger is the evidence |
| TT #14 — proof by self-reference | `meta-examples` | literally ME-everything |

Mapping is authoritative here, confirmed in each strip's proposal, and editable as strips author and critique land.

## Metadata emission

Each strip's directory emits `METADATA.json` alongside its PNG:

```jsonc
{
  "strip":              "TT #NN",
  "title":              "<short strip name>",
  "trace_spec":         "<name>",
  "trace_search_url":   "https://github.com/8007342/tlatoani-tales/search?q=%40trace+spec%3A<name>&type=code",
  "trace_spec_url":     "https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/<name>/spec.md",
  "concepts_taught":    ["Cxx"],
  "concepts_assumed":   ["Cxx"],
  "alt_text":           "<accessible description of all three panels>",
  "caption":            "Tlatoāni Tales #NN — <strip name> — @trace spec:<name>"
}
```

Publishers (any social channel the strip appears on) include `trace_search_url` in the post body. The plate is the hook; the URL is the door.

## URL format

- Search URL (the canonical "favorite query"):
  `https://github.com/8007342/tlatoani-tales/search?q=%40trace+spec%3A<name>&type=code`
- Direct spec URL:
  `https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/<name>/spec.md`

The search URL is preferred in captions because it surfaces the whole trace network around that spec — every file, script, and commit that cites it. That's the observability promise: one click, full context.

## Propagation cost

Adding this to the style bible retroactively invalidates every strip's layout. Strip #01 (the ChatGPT demo) predates the trace plate and will be re-rendered locally once LoRAs + models are in place. This is the **first real propagation event** in the project — see `meta-examples/spec.md` ME11.

The project's own state right now is *not-yet-convergent-with-its-spec*. That's fine — it's a live demonstration of C07 (iteration with aim) and C12 (loop closes). The commit history will show the convergence.

## QA integration

New VLM checks (added to `visual-qa-loop/spec.md` as this spec lands):

- `plate.trace-present` — is there a bottom-left plate?
- `plate.trace-legible` — does the plate text parse?
- `plate.trace-content` — does the text match the declared `trace_spec` in proposal.md?
- `plate.symmetry` — are the two plates visually matched as a pair?

## Trace

`@trace spec:trace-plate, spec:style-bible, spec:visual-qa-loop, spec:meta-examples`
