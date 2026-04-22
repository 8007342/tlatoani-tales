# Orchestrator

## Purpose

`tt-render` is the CLI that drives Tlatoāni Tales end-to-end: spec files → content-addressed panel cache → ComfyUI render → VLM drift critique → reroll → composite + plates → `output/Tlatoāni_Tales_NNN.png`.

This is where **convergence literally closes**. Every other spec describes a contract; this spec describes the loop that *enforces* those contracts against pixels. A spec edit is a Lamport tick (ME01); the orchestrator is the G-Set union operator that re-derives the world from the new spec state (ME05). When TT #12 ships and a reader asks "where does the loop close?" — the answer is this file and the code it governs.

`@trace spec:orchestrator`
`@Lesson lesson_loop_closes`

## Invariants

- The author **never** manually edits a rendered PNG. Every pixel is reproducible from specs + cache.
- `output/` is **ephemeral**. Delete the directory, rerun `tt-render`, pixel-identical result (modulo spec mutations since the last run).
- Every `output/Tlatoāni_Tales_NN.png` has a sibling `output/Tlatoāni_Tales_NN.json` metadata file. No metadata, no ship.
- Every panel in `cache/panels/` has a sibling QA report `cache/panels/<hash>.json`. No QA, no cache promotion.
- Every rendered strip declares exactly one `trace_spec` and one primary `lesson` (per trace-plate + lessons specs). `tt-render verify` rejects strips missing either.
- Cache keys are **content-addressed**; any upstream input change invalidates the downstream hash monotonically. Mutation never requires manual cache busting.
- Infra failures (ComfyUI down, VLM timeout) are **not** canon failures. They halt the run with a distinct exit code; they never corrupt cache or output.

## Package layout

Python package under `scripts/tt_render/`. Not a single file — each subsystem is its own module, testable in isolation.

| Module | Responsibility |
|---|---|
| `tt_render/cli/` | `argparse` entrypoint, subcommand dispatch, exit codes |
| `tt_render/specs/` | Loader for `openspec/specs/**/spec.md` and `strips/NN-slug/proposal.md`; YAML frontmatter + markdown body; validation against registry (lessons, trace_spec, depends_on) |
| `tt_render/hashing/` | Canonicalization + SHA-256; computes `global_style_hash`, `character_lora_hash`, `panel_hash` |
| `tt_render/comfy/` | HTTP client for the ComfyUI server (pool, submit workflow, poll history, stream progress) |
| `tt_render/qa/` | VLM loop; reads check definitions from `visual-qa-loop/spec.md`; drives ollama; produces drift reports |
| `tt_render/compose/` | PIL-based panel stitcher + plate compositor; font loading |
| `tt_render/metadata/` | Emits `METADATA.json` per strip per the trace-plate schema, plus `lesson_search_url` |
| `tt_render/telemetry/` | Structured JSON log events; writes `output/telemetry/<strip>.jsonl` |
| `tt_render/lint/` | Implements `tt-render verify` — licensing/trace/lesson/spec-registry invariants |

The entrypoint is registered as `tt-render` via a `pyproject.toml` console script. No other binary; no global state outside the CLI process.

## Content addressing

The cache is a G-Set (ME05): every rendered panel is a stable cell keyed by a SHA-256 of its inputs. Re-computing with the same inputs is a no-op (idempotent); mutating any input yields a new cell (monotonic).

### Global style hash

```
global_style_hash = sha256(
  canonicalize(style-bible/spec.md)        ||
  canonicalize(character-canon/spec.md)    ||
  canonicalize(symbol-dictionary/spec.md)  ||
  canonicalize(trace-plate/spec.md)
)
```

Mutating any of these files **invalidates every cached panel**, project-wide. That's the point: the style bible is not advisory, it is binding on every pixel already drawn.

### Panel hash

```
panel_hash = sha256(
  canonicalize(panel_prompt)               ||  // strip proposal panel N
  global_style_hash                        ||
  character_lora_hashes[present_chars]     ||  // per-character LoRA file sha256
  seed                                     ||  // declared in strip proposal
  base_model_hash                          ||  // flux1-schnell-fp8.safetensors sha256
  qwen_model_hash_if_text_panel            ||  // Qwen-Image, only when speech bubble
  orchestrator_schema_version                   // bump invalidates the world deliberately
)
```

Canonicalization is: strip trailing whitespace, collapse runs of blank lines to one, normalize Unicode (NFC), LF line endings, UTF-8 encode. Stable bytes in, stable hash out.

### Cache layout

```
cache/
├── panels/
│   ├── <hash>.png          — accepted panel art
│   └── <hash>.json         — full drift report for the accepting iteration
└── global_style_hash.txt   — debug breadcrumb, not authoritative
```

Cache entries are **never overwritten**. A panel that failed QA past the reroll budget is *not* cached — the strip fails to ship and the orchestrator marks it `needs-human` (per visual-qa-loop thresholds).

## Commands

| Command | Behavior |
|---|---|
| `tt-render` | Default: load specs, scan `strips/`, render all strips with stale panels, write `output/` |
| `tt-render --only NN` | Single strip (e.g. `--only 03`) |
| `tt-render --force` | Bypass cache; re-render every panel of every in-scope strip |
| `tt-render --dry-run` | Resolve hashes and print the invalidation set; render nothing |
| `tt-render --resume-from-cache-only` | Composite `output/` from existing cache only; fail if any panel is missing. Useful offline |
| `tt-render mutate --prop <spec-path.field> --to <value>` | Edits the named spec property, recomputes hashes, reports the invalidation set, re-renders stale panels, commits the spec change + new cache entries. Author-facing propagation primitive |
| `tt-render verify` | Lints (see below). Exits non-zero on any invariant violation |
| `tt-render trace <spec-or-lesson>` | Prints the coverage graph: files, commits, specs, strips that cite this trace. Local rendering of the public GitHub search URL |

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success (all in-scope strips shipped or cache-hit) |
| 10 | Canon failure (QA unrecoverable, strip marked `needs-human`) |
| 20 | Spec invariant violation (verify failed) |
| 30 | Infra failure (ComfyUI unreachable, VLM timeout, model missing) |
| 40 | Usage error (bad args, strip not found) |

`10` vs `30` matters — a canon failure means the comic is wrong; an infra failure means the tool is wrong. CI and the author treat them differently.

## `tt-render verify` — lint invariants

| Lint | Rule |
|---|---|
| `licensing.coverage` | Every tracked file matches exactly one rule in `licensing/spec.md`. Zero-match or ≥2-match is an error |
| `strip.trace-declared` | Every `strips/NN-slug/proposal.md` declares `trace_spec:` |
| `strip.lesson-declared` | Every strip proposal declares `lesson:` |
| `lesson.slug-valid` | Every declared `lesson` (primary + `reinforces:`) exists in the `lessons/spec.md` registry and is not tombstoned |
| `trace.spec-exists` | Every `@trace spec:X` (source, proposal, commit body) resolves to `openspec/specs/X/spec.md` |
| `depends.order` | Every strip's `Depends on: [Cxx]` refers to concepts whose primary strips ship *earlier* in the sequence (per concept-curriculum) |
| `plate.declared-matches-trace` | The strip's `proposal.md` `trace_spec:` matches the concrete plate to be composited |

Verify is cheap and pure. It runs at the top of every render, and as a pre-commit-friendly standalone check.

## Render flow

Pseudocode — single strip, single panel; the default command fans this out per-strip-per-panel.

```
1. load global specs        → canonical bytes for the four style specs
2. compute global_style_hash
3. load strip proposal      → panels[1..3], seed, trace_spec, lesson
4. verify strip             → tt-render verify, strip-scoped
5. for each panel:
     a. compute panel_hash
     b. if cache/panels/<hash>.png exists:
          emit render.cache_hit;  use cached art
          continue
     c. emit render.comfy_submit
     d. submit ComfyUI workflow (flux-schnell; +Qwen if panel has text)
     e. await result (poll GET /history/<prompt_id>)
     f. iteration = 1; addendum = ""
     g. loop:
          drift_report = qa.critique(panel_png, spec_context)
          emit qa.check events + qa.verdict
          if drift_report.verdict == "stable":
              write cache/panels/<hash>.png + <hash>.json
              break
          if iteration > 5 or drift_report.verdict == "needs-human":
              emit canon_failure; exit 10
          addendum = qa.build_addendum(drift_report.failed_checks)
          resubmit workflow with addendum; iteration++
6. compose strip            → PIL stitch 3 panels, paste plates, render fonts
7. write output/Tlatoāni_Tales_NN.png
8. emit composite.done and metadata
9. write output/Tlatoāni_Tales_NN.json (METADATA schema + lesson_search_url)
10. append output/telemetry/<strip>.jsonl
```

The loop at step 5g is the *literal* materialization of C12 and `lesson_loop_closes`. Telemetry from iteration `i` becomes prompt input for iteration `i+1`. The strip teaching that lesson is produced by that lesson.

## ComfyUI integration

ComfyUI is a long-running HTTP server; the orchestrator owns its lifecycle for the duration of a `tt-render` invocation.

| Aspect | Rule |
|---|---|
| Launch | First call to `comfy.submit()` spawns `python main.py --listen 127.0.0.1 --port 8188` from `tools/ComfyUI/` using the venv at `tools/ComfyUI/.venv`. Readiness: poll `GET /system_stats` until 200 |
| Pool | Single server per `tt-render` process. Multiple panel submissions are queued through the server's own queue; we do not spawn multiple ComfyUI processes |
| API | `POST /prompt` with workflow JSON (graph form), returns `prompt_id`. Poll `GET /history/<prompt_id>` until the node graph completes, then fetch output images via `GET /view?filename=...` |
| Shutdown | `atexit` + signal handler kills the child process. `tt-render` leaves no zombie servers |
| Toolbox | On Fedora Silverblue, the host invocation of `tt-render` shells into `toolbox run -c tlatoani-tales` if not already inside. The toolbox convention (project-dir = toolbox name) holds |

The server is an implementation detail of rendering, not a user-facing service. Never exposed beyond `127.0.0.1`.

## QA integration

`tt_render/qa/` is a thin adapter: the *check definitions* live in `visual-qa-loop/spec.md`. The orchestrator reads the check table at load time and presents it to the VLM as a structured prompt; it does not hard-code checks in Python.

- Thresholds: `< 0.05` stable, `0.05–0.20` reroll, `> 0.20` escalate (verbatim from visual-qa-loop).
- Max 5 rerolls per panel. After escalation to the heavier VLM, 5 more. Then `needs-human`.
- **Prompt addendum composition**: concatenate each failed check's `id` and `note` into a terse negative-direction instruction appended to the original panel prompt. Format: `"avoid: <check.id> (<check.note>); ..."`. This is ME08 materialized.
- Ollama is installed lazily on first QA-requiring run. If the install fails, exit 30 (infra), not 10 (canon).

## Composition

PIL. Inputs: three cached panel PNGs + strip metadata. Output: one composited PNG.

- Layout: horizontal 3-panel strip per style-bible. Gutters: cream paper tone (`#F4E9D3`) matching background.
- Plates: two, matched pair per trace-plate + style-bible.
  - **Episode plate** (bottom-right): text `Tlatoāni Tales #NN`.
  - **Trace + lesson plate** (bottom-left, two lines): line 1 `[@trace spec:<name>]`, line 2 `@Lesson <slug>`. Lessons spec governs the second line.
- Font: Atkinson Hyperlegible (bundled at `assets/fonts/AtkinsonHyperlegible-*.ttf`, CC BY 4.0 — already cream-paper-friendly, high legibility at small sizes). No system font fallback; missing font is exit 30.
- Plate overlap geometry per trace-plate spec: episode plate covers all of panel 3 + ~12% of panel 2; trace plate covers all of panel 1 + ~12% of panel 2.
- Plate auto-shrink: if a plate would obscure a face (detected by the QA check `plate.legibility`), shrink to ≥40% of the episode plate's area — never below.

## Metadata emission

Exactly the schema in `trace-plate/spec.md`, with one addition:

```jsonc
{
  "strip":              "TT #NN",
  "title":              "<short strip name>",
  "trace_spec":         "<name>",
  "trace_search_url":   "https://github.com/8007342/tlatoani-tales/search?q=%40trace+spec%3A<name>&type=code",
  "trace_spec_url":     "https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/<name>/spec.md",
  "lesson":             "<slug>",
  "lesson_search_url":  "https://github.com/8007342/tlatoani-tales/search?q=%40Lesson+<slug>&type=code",
  "concepts_taught":    ["Cxx"],
  "concepts_assumed":   ["Cxx"],
  "alt_text":           "<accessible description of all three panels>",
  "caption":            "Tlatoāni Tales #NN — <strip name> — @trace spec:<name>"
}
```

`lesson_search_url` format is verbatim from `lessons/spec.md`. Publishers include both search URLs in captions.

## Observability

Every interesting event emits a single-line JSON record on stdout and appends to `output/telemetry/<strip>.jsonl`. Events carry `@trace spec:orchestrator` in a `trace` field and the governing `@Lesson` in a `lesson` field — the telemetry layer is itself observable.

| Event | Fires | Key fields |
|---|---|---|
| `render.start` | per strip, at step 1 | strip, global_style_hash, seed |
| `render.cache_hit` | per panel, at 5b | strip, panel, panel_hash |
| `render.comfy_submit` | per panel, at 5c | strip, panel, panel_hash, prompt_id |
| `qa.check` | per check, per iteration | strip, panel, iteration, check.id, pass, confidence |
| `qa.verdict` | per iteration | strip, panel, iteration, drift_score, verdict |
| `composite.done` | per strip, at 7 | strip, output_path |
| `metadata.emit` | per strip, at 9 | strip, metadata_path |
| `canon.failure` | on needs-human | strip, panel, last_drift_score |
| `infra.failure` | on ComfyUI/VLM/IO fault | subsystem, error_kind |

Aggregation is grep: `jq 'select(.event == "qa.verdict")' output/telemetry/*.jsonl` is the intended first-class consumer. Dashboards are future work.

## Failure modes

| Kind | Class | Detection | Response |
|---|---|---|---|
| ComfyUI won't start | infra | bootstrap script nonzero or `/system_stats` unreachable after 60s | exit 30; log diagnostics; no cache touched |
| ComfyUI crash mid-render | infra | HTTP error or lost `prompt_id` | exit 30; partial panels not cached |
| VLM timeout | infra | ollama client timeout | exit 30 (first occurrence) or retry once then exit 30 |
| VLM install fail | infra | `ollama pull` nonzero | exit 30; `TT_QA=off` escape hatch still works |
| Cache file corrupted (bad PNG) | canon | decode fails on hit | delete entry, re-render, log `cache.repair` |
| Missing LoRA | infra | LoRA referenced in proposal is not at `tools/loras/` | exit 30; instruct author to retrain |
| Licensing lint fail | canon | `tt-render verify` rule mismatch | exit 20; no render |
| Canon reroll exhausted | canon | 5+5 rerolls and still drift > 0.20 | exit 10; strip marked `needs-human` |
| Spec references tombstoned lesson | canon | verify | exit 20 |

The orchestrator distinguishes `infra` from `canon` by exit code (30 vs 10/20) and by which events fire. This matters for CI, and it matters for the author: an infra failure is "try again"; a canon failure is "fix the spec."

## Escape hatches

| Flag / env | Effect |
|---|---|
| `TT_QA=off` | Skip VLM. Render once, accept, cache. Fast iteration |
| `TT_QA=single` | One VLM pass, no reroll. Calibration mode |
| `TT_QA=strict` | Default. Full loop with thresholds |
| `--dry-run` | Hash + invalidation set only, no renders |
| `--only NN` | Single strip |
| `--force` | Bypass cache |
| `--resume-from-cache-only` | Composite from cache; no ComfyUI, no VLM; offline |
| `TT_TRACE=1` | Verbose telemetry: every step logs, not just events |

`TT_QA` values mirror `visual-qa-loop/spec.md` verbatim.

## Future convergence

| ID | Direction | Note |
|---|---|---|
| — | REUSE migration (ME04) | `verify` learns SPDX headers once `LICENSES/` lands |
| — | Multi-agent render waves | Agent-per-strip during batch re-renders (e.g. after a style-bible mutation). Each agent a `tt-render --only NN` |
| — | GPU-batch panels | Multiple panels of one strip submitted as a single ComfyUI workflow (latent-batched). Cache keys unchanged |
| — | `tt-render watch` | Daemon mode: inotify specs, incrementally invalidate + re-render. Lives on top of existing content addressing |
| — | Headless QA | Move VLM critique to a dedicated service; current ollama adapter becomes one backend |

Each step strictly refines; none invalidate the current spec's contracts. That's C07 on the orchestrator layer itself.

## Trace

`@trace spec:orchestrator, spec:style-bible, spec:character-canon, spec:symbol-dictionary, spec:trace-plate, spec:visual-qa-loop, spec:lessons, spec:licensing, spec:meta-examples, spec:concept-curriculum`
`@Lesson lesson_loop_closes`
`@Lesson lesson_edits_that_reconcile`
`@Lesson lesson_see_the_now`
