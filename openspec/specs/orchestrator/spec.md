# Orchestrator

## Purpose

`tt-render` is the **Rust CLI** that drives Tlatoāni Tales end-to-end. It turns a repo full of spec markdown into a directory full of rendered PNGs, critiqued by a local VLM, composited with three plates, and emitted with full metadata — and does so through a **typed event bus** whose every state transition is observable.

The orchestrator is the point where convergence **literally closes**. Every other spec states a contract; this one describes the loop that enforces those contracts against pixels. A spec edit is a Lamport tick; the orchestrator is the G-Set union operator that re-derives the world from the new spec state (ME05). When TT 13/15 ships and a reader asks *"where does the loop close?"* — the answer is this file, the crates it governs, and the event stream those crates emit as they run.

Framed in Rust/event-bus terms: **the orchestrator is a typed event bus where every state transition is observable.** Nothing mutates silently. Every `CacheHit`, every `RerollScheduled`, every `MetadataWritten` is a typed value on a stream, citing the spec that governs it. `@Lesson S1-1300` (*loop-closes*) is not a metaphor here — it is the architecture.

`@trace spec:orchestrator`
`@Lesson S1-1300`

## Language & runtime choice

The orchestrator is a **Rust workspace**. No Python ships in our code. Python exists only in the untrusted containers that host ComfyUI and ai-toolkit; we cross the boundary over HTTP and bind-mounted filesystem, never in-process.

Author directive, verbatim: *"Python is for hand-crafting by sloppy human hands that are too lazy to implement a few extra lines for architectural correctness. We prefer Rust wherever possible."* And: *"We should use tiny rust crates, in a micro services fashion, event driven, using observable streams."*

| Layer | Choice | Why |
|---|---|---|
| Language | **Rust (stable toolchain)** | Correctness, type-safe events, cheap rebuilds inside the trusted toolbox |
| Async runtime | **tokio** | Industry-standard; broadcast channels; `tokio::select!`; `tokio::fs` for non-blocking IO |
| Logging | **`tracing` crate** | Structured fields, spans, subscribers; `spec = "<name>"` field on every accountability-tagged event |
| Observability | **`futures::Stream`** | Subscribers consume events as async streams — CLI UI, JSONL sink, Calmecac live-watch all plug into the same source |
| Errors | `thiserror` in library crates, `anyhow` in binary crates | Explicit error taxonomy; infra (exit 30) vs canon (10/20) |

Reproducibility of the *comic itself* stays unchanged — content addressing, caching, VLM loop. What changes is the substrate. See `isolation/spec.md` for the hard trust-boundary contract this spec honors.

## Invariants

- **Reproducible.** Every pixel derives from specs + cached content. No manual edits to rendered PNGs. Ever.
- **Content-addressed cache.** A spec mutation invalidates every affected panel monotonically. Mutation never requires manual cache busting (ME05).
- **Trust boundary.** The orchestrator runs in the trusted `tlatoani-tales` toolbox. Image-gen (ComfyUI) and VLM (ollama) and trainer (ai-toolkit) runtimes run in **hardened untrusted containers**. The only communication is HTTP over `127.0.0.1:<port>` and bind-mounted filesystem. No shared process space. No network egress from the untrusted zone at render time.
- **Event-driven.** Every state transition emits a typed event on the bus. There are no hidden state mutations. If it happened, it's on the stream.
- **Never poll in tight loops.** Use streams, `tokio::sync::broadcast`, `notify` crate for filesystem, HTTP long-poll for ComfyUI history. A tight-loop `sleep` poll is a bug.
- **Output is ephemeral.** Delete `output/`, rerun `tt-render`, pixel-identical result (modulo spec mutations since the last run).
- **No metadata, no ship.** Every `output/Tlatoāni_Tales_NN.png` has a sibling `output/Tlatoāni_Tales_NN.json`. Every cached panel has a sibling drift report.
- **Infra ≠ canon.** Container unreachable, model missing, bind-mount denied — infra failures. Drift score past `needs-human`, tombstoned lesson referenced — canon failures. Different exit codes, different events.

## Workspace layout

`Cargo.toml` workspace at repo root. One crate per bounded context (tillandsias idiom). Binaries under `bin/`.

```
Cargo.toml                       # workspace
crates/
  tt-core/                       # shared types: StripId, PanelId, LessonId, SpecName, Hash, error taxonomy, cache paths, the @trace + @Lesson domain types
  tt-specs/                      # OpenSpec loader — YAML frontmatter + markdown body via pulldown-cmark, validation, typed graph of lessons/specs/strips
  tt-hashing/                    # canonicalization (NFC, LF, UTF-8, trim) + SHA-256; global_style_hash, character_lora_hash, panel_hash
  tt-events/                     # typed event bus (tokio::sync::broadcast) + observable streams (futures::Stream); one enum-per-domain (RenderEvent, QaEvent, ComfyEvent, ComposeEvent, CacheEvent, LintEvent); subscriber helpers
  tt-comfy/                      # async HTTP client to the untrusted ComfyUI container via reqwest; POST /prompt, stream /history/<id>; strictly typed workflow JSON structs
  tt-qa/                         # VLM client — ollama over HTTP, drift scorer, reroll-addendum composer; reads check definitions from visual-qa-loop/spec.md at load time, does NOT hard-code checks
  tt-compose/                    # image composition via image + imageproc; font loading via ab_glyph; title plate composited over FLUX output after Qwen-Image renders the stylized text
  tt-metadata/                   # METADATA.json emitter — exactly the trace-plate/spec.md schema (title, lesson, trace_spec, plate_regions, calmecac_*_url, concepts_*, reinforces_lessons, alt_text, caption)
  tt-telemetry/                  # tracing subscribers + jsonl sink + metrics; every accountability event carries spec = "<name>" and lesson = "<Sn-NNN>" fields; feeds Calmecac's convergence dashboards
  tt-lint/                       # verify subcommand — licensing R## coverage, trace/lesson-presence, spelling (Tlatoāni-with-macron), plate-declaration presence, slug-in-registry, spec-in-lesson-coverage alignment
  tt-lora/                       # Rust wrapper over the ai-toolkit Python subprocess (which lives in the untrusted trainer container); manifest IO per character-loras/spec.md; LoRA hash computation for cache-key participation
  tt-calmecac-indexer/           # builds calmecac-index.json — walks specs + strips + metadata + git log, emits the concept-graph JSON with paths/hashes erased per calmecac/spec.md's abstraction rule
bin/
  tt-render/                     # the tt-render CLI (clap); subcommand dispatch; composes the crates above; sets up tracing + event bus; exits with the documented codes
  tt-calmecac/                   # small CLI that runs the indexer, writes output/calmecac-bundle/, optionally invokes scripts/tlatoāni_tales.sh to start the viewer container
```

**We deliberately do NOT ship our own HTTP server.** Calmecac is served from a tiny Alpine or Fedora-minimal httpd container (see `calmecac/spec.md`). Author directive, verbatim: *"we do not always know better."* Wisdom is knowing where to stop. Industrial-grade httpd is a correctness primitive; rewriting it in Rust would be the opposite of this project's thesis.

## Event model

Every crate emits events on a shared broadcast bus (`tt-events`). Variants below are canonical — subscribers filter by domain, by spec tag, or by strip. Fields are illustrative; `tt-core` types are the source of truth.

### `RenderEvent`

| Variant | Fields |
|---|---|
| `RunStarted` | run_id, started_at, spec_scope |
| `StripDiscovered` | strip, proposal_path |
| `SpecLoaded` | spec_name, hash |
| `PanelHashComputed` | strip, panel, panel_hash |
| `CacheHit` | strip, panel, panel_hash |
| `CacheMiss` | strip, panel, panel_hash, reason |
| `RunComplete` | run_id, strips_rendered, strips_cached, duration |
| `RunFailed` | run_id, class (Infra\|Canon), detail |

### `ComfyEvent`

| Variant | Fields |
|---|---|
| `Submitted` | strip, panel, panel_hash, prompt_id |
| `Progress` | prompt_id, step, total |
| `Rendered` | prompt_id, output_path |
| `Failed` | prompt_id, error_kind |
| `Timeout` | prompt_id, elapsed |

### `QaEvent`

| Variant | Fields |
|---|---|
| `Submitted` | strip, panel, iteration, model |
| `CheckResult` | strip, panel, iteration, check_id, pass, confidence |
| `Verdict` | strip, panel, iteration, drift_score, verdict (`Stable`\|`Reroll`\|`Escalate`\|`NeedsHuman`) |
| `RerollScheduled` | strip, panel, iteration_next, addendum |

### `ComposeEvent`

| Variant | Fields |
|---|---|
| `PanelsLoaded` | strip, hashes |
| `PlatesRendered` | strip, plate_kinds |
| `TitleComposited` | strip, title_display, source (Qwen-Image) |
| `ComposeDone` | strip, output_path |
| `MetadataWritten` | strip, metadata_path |

### `CacheEvent`

| Variant | Fields |
|---|---|
| `HashComputed` | panel_hash, inputs |
| `Hit` | panel_hash |
| `Miss` | panel_hash |
| `Promoted` | panel_hash, png_path, report_path |
| `Evicted` | panel_hash, reason (bad PNG, manifest mismatch) |

### `LintEvent`

| Variant | Fields |
|---|---|
| `Started` | rules_scope |
| `RuleViolated` | rule_id, path, detail |
| `Passed` | rules_checked |
| `Failed` | violations |

### Spec tag on every event

Every event carries an optional `spec_tag: Option<SpecName>` field — the Rust equivalent of `@trace spec:X`. Subscribers filter by tag (e.g. telemetry for the `visual-qa-loop` spec is every event tagged `spec:visual-qa-loop`). Events MAY carry a `lesson_tag: Option<LessonId>` for `@Lesson` annotations. `tt-telemetry` renders both into the JSONL sink as `spec` and `lesson` fields — matching the cross-project trace-annotation convention.

## Observable streams / subscribers

The same event source fans out to multiple consumers. Subscribers use `tokio::sync::broadcast::Receiver` or a `futures::Stream` adapter.

| Subscriber | Sink | Role |
|---|---|---|
| **CLI UI** | stderr in `tt-render` | Per-strip progress via `indicatif` bars driven by the stream. Live status visible to the author. |
| **Telemetry JSONL** | `output/telemetry/<strip>.jsonl` | One event per line, full fidelity. Grep-first debugging. Feeds Calmecac's convergence dashboards. |
| **Cache manager** | `cache/panels/` + internal metrics | Tracks hit/miss ratios; emits `CacheEvent::Hit` / `Miss` of its own; participates in the same bus it observes. |
| **Calmecac live-watch** *(optional, future)* | SSE endpoint | Forwards events to a running Calmecac instance. Spec'd as optional. Respects the hard boundary: Calmecac runs in its own container; if SSE is used, the socket is a bind-mounted Unix domain socket or a localhost port declared in the launcher. |
| **Structured log** | `tracing` subscriber | Developer-facing. Human-readable when a terminal is attached; JSON when piped. |

All subscribers read the **same stream**. Adding a subscriber is purely additive — no crate mutates shared state to accommodate observers.

## Content addressing

Unchanged from the prior spec's invariants — restated in the Rust vocabulary.

### Global style hash

```
global_style_hash = sha256(
  canonical(style-bible/spec.md)       ||
  canonical(character-canon/spec.md)   ||
  canonical(symbol-dictionary/spec.md) ||
  canonical(trace-plate/spec.md)
)
```

Mutating any of the four style specs invalidates every cached panel project-wide. The style bible is binding on every pixel already drawn.

### Character LoRA hash

```
character_lora_hash = sha256(
  canonical(characters/<name>/lora-manifest.json)
)
```

Per `character-loras/spec.md`. The manifest is committed; the `output.sha256` inside it is the weights hash. Retraining = new manifest hash = new panel hash.

### Panel hash

```
panel_hash = sha256(
  canonical(panel_prompt)              ||  // from strips/NN-slug/proposal.md
  global_style_hash                    ||
  character_lora_hashes[present]       ||  // sorted by character name
  seed                                 ||  // declared in strip proposal
  base_model_hash                      ||  // flux1-schnell-fp8.safetensors sha256, pinned in manifest
  qwen_image_hash_opt                  ||  // only when panel has a title plate (Qwen-Image render)
  schema_version                           // bumping invalidates the world deliberately
)
```

### Canonicalization

NFC Unicode normalization, LF line endings, UTF-8, trimmed leading/trailing whitespace, runs of blank lines collapsed to one. Stable bytes in, stable hash out.

### Cache layout

```
cache/
├── panels/
│   ├── <hash>.png               — accepted panel art
│   └── <hash>.json              — drift report for the accepting iteration
└── global_style_hash.txt        — debug breadcrumb, not authoritative
```

Cache entries are **never overwritten**. A panel that exhausts rerolls is not cached; the strip fails, exit 10, no corruption. Cache = ME05 materialized (G-Set CRDT: each hash is a cell; union of cells is idempotent; mutation produces a new cell).

## Trust boundary integration

The orchestrator sits in the **trusted zone** and never runs untrusted Python code in-process or as a child of its own process. All calls to ComfyUI, ollama, and ai-toolkit cross the boundary explicitly.

See `isolation/spec.md` for the full contract. This spec honors these rules:

| Rule | Mechanism |
|---|---|
| HTTP-only for inference | `tt-comfy` hits `POST http://127.0.0.1:<comfy-port>/prompt` and `GET /history/<id>`. `tt-qa` hits `POST http://127.0.0.1:<ollama-port>/api/chat`. Endpoints are the untrusted containers' forwarded ports on localhost. |
| Filesystem-only for artefacts | The untrusted container mounts `tools/ComfyUI/models/` read-only, `cache/panels/` read-write, `output/` read-write, `strips/` read-only. Orchestrator writes prompts to one directory and reads rendered PNGs from another. No shared memory. |
| Hardening flags on every untrusted `podman run` | `--cap-drop=ALL --security-opt=no-new-privileges --userns=keep-id --read-only --network=<mode>`, where `<mode>` defaults to `none` (fully offline inference — the comic is rendered offline) and is only `bridge` during an explicit model-download phase. |
| No egress at render time | With `--network=none`, the untrusted container cannot reach the public internet. Every image and every critique happens against local weights only. |
| Trainer in its own sibling container | `tt-lora` spawns ai-toolkit in a separate untrusted container for LoRA training. Same flags, same boundary. The orchestrator's trusted process parses the trainer's stdout/stderr; it never imports Python. |

This boundary is the project's load-bearing **Season 2 meta-example candidate**: *"this comic was rendered using offline models."* Flagged for author curation per the curation rule — not canonized here, not added to `meta-examples/spec.md` by this spec.

## Commands

```
tt-render                                      # default: render all stale strips
tt-render --only NN                            # single strip
tt-render --force                              # blow cache; re-render everything in scope
tt-render mutate --prop <spec.path.field> --to <value>
                                               # spec-mutation primitive
tt-render verify                               # lint pass; exits non-zero on any violation
tt-render trace <spec-name>|<lesson-id>        # coverage graph for a trace or lesson
tt-render watch                                # long-running; re-renders on spec/strip/character change
```

| Command | Behavior |
|---|---|
| *default* | Load specs, scan `strips/`, render all strips with stale panels, composite, emit metadata, write `output/`, append telemetry. |
| `--only NN` | Scope to a single strip (e.g. `--only 03`). |
| `--force` | Bypass cache; re-render every panel of every in-scope strip. |
| `mutate` | Edits the named spec property, recomputes hashes, emits the invalidation set as events, re-renders stale panels, stages the spec edit + new cache entries as a commit. Author-facing propagation primitive. |
| `verify` | Runs `tt-lint`. Exits non-zero on any violation. Cheap; pre-commit-friendly. |
| `trace` | Prints the coverage graph for a trace name or lesson ID — an offline version of Calmecac's drill-down view. |
| `watch` | Long-running daemon mode. Uses `notify` crate on `openspec/specs/`, `strips/`, `characters/`. Re-renders affected strips on change. |

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 10 | Canon failure (QA unrecoverable; strip marked `needs-human`) |
| 20 | Spec invariant violation (`verify` failed) |
| 30 | Infra failure (container unreachable, model missing, bind-mount denied) |
| 31 | Infra failure — permission denied on bind mount (sub-code of 30) |
| 40 | Usage error (bad args, strip not found) |

`10` vs `30` matters. A canon failure means the comic is wrong. An infra failure means the tool is wrong. CI and the author treat them differently.

## Render flow

Step-by-step in plain English, referencing the crates. Not Rust code — contracts only.

1. `bin/tt-render` parses args (`clap`). Sets up the tracing subscriber, telemetry JSONL sink, and event bus. Emits `RenderEvent::RunStarted`.
2. `tt-specs::load_all()` walks `openspec/specs/` and `strips/`, parses YAML frontmatter + markdown bodies via `pulldown-cmark`, returns a typed graph. Validates against the registries (lessons, trace_spec, depends_on). Emits `SpecLoaded` per spec.
3. `tt-hashing::compute_all(specs)` computes `global_style_hash`, every `character_lora_hash`, and every `panel_hash`. Emits `PanelHashComputed` per panel.
4. Diff panel hashes against `cache/panels/`. Emits `CacheHit` or `CacheMiss` per panel.
5. For each stale panel:
   a. `tt-comfy::submit(workflow)` — POST to the untrusted ComfyUI container. Emits `ComfyEvent::Submitted`.
   b. Await rendered PNG via streamed history. `ComfyEvent::Progress` flows as the node graph advances. `ComfyEvent::Rendered` on completion.
   c. `tt-qa::critique(panel)` — hits ollama, loads check definitions from `visual-qa-loop/spec.md`. Emits `QaEvent::Submitted`, `CheckResult` per check, and `Verdict`.
   d. If `Verdict::Reroll` — `tt-qa::addendum(failed_checks)` composes a terse negative-direction prompt (*"avoid: <check.id> (<check.note>); …"*). Emits `RerollScheduled`. Resubmit. Max 5 rerolls per model, then escalate to the heavier VLM. 5 more. Then `NeedsHuman` → exit 10.
   e. If `Verdict::Stable` — `tt-compose::write_cache(hash, png, report)` promotes the panel. Emits `CacheEvent::Promoted`.
6. Once all three panels of a strip are fresh:
   a. `tt-comfy::render_title(lesson.display_name, style_hints)` calls Qwen-Image in the same untrusted ComfyUI container (or a sibling), receives a transparent stylized title PNG. Emits `TitleComposited`.
   b. `tt-compose::composite(panels + plates)` — uses `image` + `imageproc` for stitching, `ab_glyph` for the chrome-plate text (trace+lesson plate bottom-left, episode plate bottom-right). Pastes the Qwen-Image title over the FLUX composite. Writes `output/Tlatoāni_Tales_NN.png`. Emits `ComposeDone`.
   c. `tt-metadata::emit(strip)` writes `output/Tlatoāni_Tales_NN.json` per the trace-plate schema. Emits `MetadataWritten`.
7. After all strips: `tt-calmecac-indexer::build()` regenerates `output/calmecac-bundle/calmecac-index.json` so the viewer is consistent with this render (per `calmecac/spec.md`). Emits `RunComplete`.

The loop at step 5c–5d is the literal materialization of `@Lesson S1-1300` (*the loop closes*). Telemetry from iteration `i` becomes prompt input for iteration `i+1`. The strip teaching that lesson is produced by that lesson.

## Failure modes

The canon-vs-infra split is load-bearing. Container-specific modes are new.

| Kind | Class | Detection | Response |
|---|---|---|---|
| Container unreachable (podman not running; `--network=none` misconfigured) | infra | TCP refused on localhost port | exit 30; log diagnostics; hint to run bootstrap |
| Container `ImageNotFound` | infra | `podman run` nonzero with known error string | exit 30; hint to run `scripts/bootstrap-comfyui.sh` |
| Bind-mount permission denied | infra | stat failure inside untrusted container | exit 31 (sub-code of 30) |
| ComfyUI crash mid-render | infra | HTTP error or lost `prompt_id` | exit 30; partial panels not cached |
| VLM timeout | infra | ollama client timeout | exit 30 (first occurrence) or retry once then exit 30 |
| VLM install fail | infra | `ollama pull` nonzero | exit 30; `TT_QA=off` still works |
| Drift > `needs-human` for 5 rerolls then 5 more on heavy VLM | canon | QA verdict escalation exhausted | exit 10; strip logged and skipped; other strips continue |
| Missing LoRA referenced by proposal | infra | manifest path not in `tools/loras/` | exit 30; instruct author to retrain |
| Licensing / trace / lesson-presence lint fail | canon | `tt-render verify` rule mismatch | exit 20; no render |
| Strip references tombstoned lesson | canon | `verify` resolves lesson registry | exit 20 |
| Cache file corrupted (bad PNG) | canon | decode failure on hit | delete entry, re-render, emit `CacheEvent::Evicted` |
| Spec parse error / missing `@trace` / missing `@Lesson` | canon | `tt-specs` validation | exit 10 |

## Escape hatches

| Flag / env | Effect |
|---|---|
| `TT_QA=off` | Skip VLM. Render once, accept, cache. Fast iteration. |
| `TT_QA=single` | One VLM pass, no reroll. Calibration mode. |
| `TT_QA=strict` | Default. Full loop with thresholds from `visual-qa-loop/spec.md`. |
| `TT_OFFLINE=1` | Force `--network=none` on all untrusted containers. Default posture. Setting this explicitly asserts the offline claim for captioning. |
| `TT_TRACE=1` | Verbose telemetry: every event logs, not just accountability-tagged. |
| `--dry-run` | Hash + invalidation set only; render nothing. |
| `--only NN` | Single strip. |
| `--force` | Bypass cache. |
| `--resume-from-cache-only` | Composite from cache; no ComfyUI, no VLM; offline. |

`TT_QA` values mirror `visual-qa-loop/spec.md` verbatim. `TT_OFFLINE=1` mirrors `isolation/spec.md`.

## Observability of the orchestrator itself

Meta-recursion. `tt-telemetry` emits events that are themselves `@trace`-tagged and `@Lesson`-tagged. Running `tt-render trace spec:orchestrator` surfaces every commit, every source file, every strip caption, and every past telemetry record citing this spec — including the telemetry from past runs of `tt-render` itself.

The orchestrator observes itself. Its telemetry feeds Calmecac's convergence dashboards the same way every other spec's telemetry does. That is this spec's contribution to `@Lesson S1-1000` (*dashboards-must-add-observability*) and `@Lesson S1-1500` (*proof-by-self-reference*): the tool that renders the comic about observability is, itself, observable.

## Future convergence

Each step strictly refines; none invalidate the current contracts.

| Direction | Note |
|---|---|
| gRPC-style event stream between `tt-render` and Calmecac | Currently SSE proposed; gRPC would give typed schemas across the socket |
| Hot-reload of lesson specs | The `watch` subcommand grows incremental re-render as specs change |
| Multi-GPU batching | Panels currently go 1-by-1; batch them as one ComfyUI workflow (latent-batched); cache keys unchanged |
| Rust-native LoRA trainer | Long-term: drop the ai-toolkit subprocess dependency; `tt-lora` absorbs the training path directly. Blocked on FLUX trainer availability in Rust; not on our critical path. |
| REUSE compliance | `verify` learns SPDX headers once `LICENSES/` lands (ME04) |
| Multi-agent render waves | One `tt-render --only NN` agent per strip during batch re-renders (e.g. after a style-bible mutation) |

## Trace

`@trace spec:orchestrator, spec:isolation, spec:visual-qa-loop, spec:trace-plate, spec:lessons, spec:calmecac, spec:character-loras, spec:tombstones, spec:licensing, spec:meta-examples`
`@Lesson S1-1300` *(loop-closes — this spec is the loop)*
`@Lesson S1-1500` *(proof-by-self-reference — the orchestrator is self-observing)*
