# Visual QA Loop

## Purpose

Every rendered panel is critiqued by a local vision-language model against the canon specs, scored for drift, and re-rendered if drift exceeds a threshold. The VLM's output is **telemetry** — structured measurement of whether the panel converges on the spec.

This spec is also a live demonstration of curriculum concepts C08–C13:

| Concept | How this spec demonstrates it |
|---|---|
| C08 — observability > unit tests | Unit tests can't see a double-tail. Observability can. |
| C09 — observability > just logs | We emit a structured drift report, not a raw VLM transcript. |
| C10 — dashboards must add observability | The drift report surfaces relationships (check → note → reroll addendum), not just numbers. |
| C11 — telemetry = meaning of logs | The drift *score* is the meaning; the raw VLM output is the log. |
| C12 — meaning is operable | Drift scores aggregate across panels, trend over iterations, gate cache promotion. |
| C13 — the loop closes | The drift report becomes the prompt addendum for the next render. |

When TT 08/15–13/15 ship, the strips can point readers at this spec and say: *the comic you are reading was produced this way.*

## Runtime

- **Ollama** inside the `tlatoani-tales` toolbox (installed lazily, not yet).
- **Vision model** (candidates, pick per cost/quality):
  - `moondream:2b` — ~1.7GB, fast, good for attribute-presence checks (tails, crown, plate position).
  - `qwen2.5-vl:7b` — ~5GB, stronger reasoning; use for composition + palette + character-canon adherence.
  - `llava:13b` — ~8GB, fallback.

Start with `moondream:2b` as the first-pass cheap filter; escalate to `qwen2.5-vl:7b` for borderline cases.

## Drift score schema

For each rendered panel, emit a JSON drift report:

```jsonc
{
  "panel_hash": "sha256:...",
  "strip": "TT 03/15",
  "panel": 2,
  "iteration": 1,
  "model": "moondream:2b",
  "checks": [
    { "id": "tlatoāni.single-tail",     "spec": "character-canon", "pass": true,  "confidence": 0.92 },
    { "id": "tlatoāni.crown-present",   "spec": "character-canon", "pass": true,  "confidence": 0.87 },
    { "id": "covi.ambiguous-white",     "spec": "character-canon", "pass": true,  "confidence": 0.95 },
    { "id": "covi.good-mood",           "spec": "character-canon", "pass": false, "confidence": 0.71, "note": "expression reads dejected" },
    { "id": "palette.paper-bg",         "spec": "style-bible",     "pass": true,  "confidence": 0.88 },
    { "id": "plate.episode.position",   "spec": "style-bible",     "pass": true,  "confidence": 0.93 },
    { "id": "plate.episode-total-format", "spec": "trace-plate",   "pass": true,  "confidence": 0.94, "note": "episode plate reads 'Tlatoāni Tales NN/15' — the /TOTAL denominator is present, not a bare #NN" },
    { "id": "plate.trace-present",      "spec": "trace-plate",     "pass": true,  "confidence": 0.91 },
    { "id": "plate.trace-legible",      "spec": "trace-plate",     "pass": true,  "confidence": 0.89 },
    { "id": "plate.trace-content",      "spec": "trace-plate",     "pass": true,  "confidence": 0.94, "note": "matches proposal.md trace_spec" },
    { "id": "plate.lesson-present",     "spec": "trace-plate",     "pass": true,  "confidence": 0.93 },
    { "id": "plate.lesson-legible",     "spec": "trace-plate",     "pass": true,  "confidence": 0.90 },
    { "id": "plate.lesson-slug-valid",  "spec": "lessons",         "pass": true,  "confidence": 0.96, "note": "slug is in registry" },
    { "id": "plate.lesson-spec-aligned","spec": "lessons",         "pass": true,  "confidence": 0.88, "note": "declared spec is in lesson's coverage list" },
    { "id": "plate.title-present",            "spec": "trace-plate", "pass": true,  "confidence": 0.92, "note": "top-left title plate present" },
    { "id": "plate.title-legible",            "spec": "trace-plate", "pass": true,  "confidence": 0.90, "note": "title text parses cleanly" },
    { "id": "plate.title-matches-declared",   "spec": "trace-plate", "pass": true,  "confidence": 0.89, "note": "matches proposal.md title field" },
    { "id": "plate.title-position-valid",     "spec": "trace-plate", "pass": true,  "confidence": 0.91, "note": "top-left by default, or right-floated if declared in proposal" },
    { "id": "plate.symmetry",           "spec": "trace-plate",     "pass": true,  "confidence": 0.90 }
  ],
  "drift_score": 0.14,
  "verdict": "reroll"
}
```

**drift_score** = weighted sum of failed checks × (1 - confidence). Range 0.0–1.0.

Alongside the trace, lesson, and episode plate checks, the `plate.title-*` checks enforce *title-plate observability*: a top-left title plate must be present, legible, match the `title` field declared in the strip's `proposal.md`, and sit in the top-left region by default (or intentionally right-floated if — and only if — the proposal declares so). The title plate is the strip's first line of in-frame observability; it earns the same VLM scrutiny as the lesson and trace plates.

## Thresholds

- `drift_score < 0.05` → **stable**, panel accepted, promoted to cache.
- `drift_score 0.05–0.20` → **reroll** with prompt addendum derived from failed checks. Max 5 rerolls.
- `drift_score > 0.20` → **escalate** to heavier VLM. If still failing after heavy VLM + 5 rerolls, panel is marked `needs-human` and the orchestrator logs it for the author's review.

## The loop (C12 live)

```
   spec ──► render (ComfyUI) ──► panel.png
                                   │
                                   ▼
                            VLM critique (ollama)
                                   │
                                   ▼
                            drift report (telemetry)
                                   │
                                   ▼
           ┌───────────────────────┼───────────────────────┐
           ▼                       ▼                       ▼
        stable                   reroll                 escalate
       (accept)              (with addendum)         (heavier VLM)
                                   │
                                   └──► next iteration's prompt
```

The addendum is constructed from the failed checks — their `id`, their `note`, and a terse instruction. This is *meaning of logs becoming input* (C11 + C12 in one breath).

## Per-strip review artefact

Each strip's dir contains `qa-log.jsonl` — one line per iteration per panel. Over a strip's life this is the accountability trail. The author reads it to understand not just *the output* but *the path of convergence* it took.

## Escape hatches

- `TT_QA=off` env var — skip VLM entirely, trust the render (fast iteration mode).
- `TT_QA=single` — one pass, no reroll (calibration mode).
- `TT_QA=strict` (default) — full loop with thresholds.

## Out of scope for this spec

- The VLM itself is not trained or fine-tuned. We use pretrained weights via ollama.
- Author-in-the-loop review — humans can override any VLM verdict by editing the panel's drift report and setting `verdict: "accepted"`.

## Trace

`@trace spec:visual-qa-loop, spec:concept-curriculum`
`@Lesson S1-800`
