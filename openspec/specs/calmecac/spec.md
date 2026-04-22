# Calmecac

## Purpose

Calmecac is Tlatoāni Tales' reader-facing **observability viewer**. It is where a reader who has finished a strip goes to see *how the strip was made* — the teachings it serves, the rules that govern its pixels, the neighbouring strips that reinforce it, and the path of convergence the repo took to get here.

The viewer is the literal, tangible expression of **@Lesson S1-1000** — *dashboards must add observability*. The comic teaches that a real dashboard surfaces the reasoning scaffold between signal and meaning. Calmecac *is* that scaffold for the comic itself: every plate, every chip, every dashboard in it exists so a reader can follow a relationship they could not follow in the strip alone. The reader finishes the comic, clicks a plate, and is transported from the teaching to the mechanism — and the mechanism teaches the same lesson, one level up, by *being* the thing it described.

Two contracts govern this spec:

| Contract | Statement |
|---|---|
| **Teach-by-existing** | Calmecac must be an observable dashboard, not a pretty one. If a reader walks out of it having seen charts but not relationships, the viewer failed S1-1000 and therefore failed the project. |
| **Abstraction level** | Calmecac presents **concepts** — Lessons, Rules, Traces, Strips, Changes. It never presents mechanism — no file paths, no markup tokens, no commit hashes, no tool names. The reader's mental model is *lessons and rules as ideas*, not *files on disk*. |

The concept-only rule is load-bearing. The underlying substrate is read from the repo's specs and rendered-strip metadata, but that fact is invisible in the UI. Every label, every link, every error message stays at the level the reader learned in the comic.

## Name and symbolism

*Calmecac* was the Nāhua institution of higher learning — the school where nobles and priests trained beyond the elementary *telpochcalli*. The viewer earns the name by being where readers go **after** the comic to learn how it is made. The comic is the public square; Calmecac is the upstairs library. The name is not decorative; it declares the viewer's relationship to the reader — a second, deeper door, opened only by those who chose to open it.

The naming also honours a commitment made earlier in `trace-plate/spec.md`: the `calmecac.` subdomain is already wired into every strip's bottom-left plate. This spec makes that subdomain real.

## Deployment targets

Calmecac runs in two shapes. Both serve the same static bundle; the difference is who launches the container and where the domain name points.

| Shape | Host | Launcher | Content |
|---|---|---|---|
| **Local** | `http://localhost:<port>` | `scripts/tlatoāni_tales.sh` (Silverblue-only) | Same static bundle as production, served out of the current checkout. Intended for authors iterating and for readers who want to run the viewer offline. |
| **Production** | `calmecac.tlatoani-tales.com` | CDN / static host, container image optional | The published bundle for the current main. Deep-linkable: `…/lesson/S1-NNN` and `…/rule/<name>` resolve directly. |

Meanwhile the public comic at `www.tlatoani-tales.com` stays a **plain comic**: strips, captions, episode pages. No observability mode there. The left plate on `www` links out to the matching `calmecac.` deep-link; crossing the subdomain boundary is the reader's signal that they are leaving the story and entering the atelier.

Both shapes serve **the same build output** (`output/calmecac-bundle/`). Production and local differ only in who hosts the bytes — this is a deliberate monotonic convergence choice: a single bundle, two addresses.

### Boundary with `www.tlatoani-tales.com`

- `www.` is the comic. Strips, captions, episode permalinks, feeds. No observability UI, no dashboards. The three plates render as in the rendered PNG; the bottom-left plate is an image-map hotlink to `calmecac.`.
- `calmecac.` is the observability layer. Every strip available on `www.` is also visible here, but in observability mode (see **UI model**).
- The `[@Lesson S1-NNN]` hotspot on `www.` is a plain outbound link. The reader chooses, deliberately, to cross over.

## Silverblue-only constraint

The local launcher script (`scripts/tlatoāni_tales.sh`) runs **only on Fedora Silverblue**. Any other OS is refused with a one-line message and a non-zero exit.

This is by design. It is a **seed** for Season 2's curriculum on immutable-OS boundaries, toolbox discipline, and podman-run-drop-privileges. The viewer does not need Silverblue in theory; it needs Silverblue in practice *because the project teaches that operational substrate matters*. When a Season 2 lesson ships, the constraint will already exist in the repo for it to point at — proof-by-self-reference, rehearsed.

| Rule | Mechanism |
|---|---|
| Silverblue check | Read `/etc/os-release`; accept only `VARIANT_ID=silverblue`. |
| Podman check | `command -v podman` must return a real binary path. (Pre-installed on Silverblue; defensive anyway.) |
| Failure mode | One-line message: *"Local viewer supports Fedora Silverblue only. Season 2 teaches why."* Non-zero exit. No partial launch, no ASCII fallback, no `sudo` workaround. |
| Teachable-break candidate | **TB05 candidate** — *Silverblue-only launcher as living Season-2 seed.* Flagged for author curation; not canonized here. |

The refusal message is deliberately **non-apologetic**. It is part of the teaching. A reader on another OS who encounters the message has just received a footnote that Season 2 will later cash in.

## Launcher script contract

`scripts/tlatoāni_tales.sh` is the single local entry point. Its filename carries the macron (per `tlatoāni-spelling/spec.md`); no ASCII fallback. If a filesystem or shell refuses the UTF-8 filename (rare on modern Linux), that refusal becomes a catalogued teachable break — silent ASCII substitution is forbidden.

### Invariants

- Script is **idempotent**. Running it twice while the viewer is already up logs *"already running at http://localhost:<port>"* and exits 0. No respawn, no port-hopping.
- Script is **single-instance**. Exactly one container named `tlatoani-tales-viewer` (ASCII-only container name per teachable break **TB03**) exists at a time.
- Script **owns** the container lifecycle. It creates, starts, stops, and rebuilds. It does not assume anyone else has touched podman state.
- Script reports **exit codes** that the orchestrator and future CI can trust.

### Flags

| Flag | Effect |
|---|---|
| *(none)* | Start the viewer. Idempotent. Best-effort open the viewer in the reader's browser. |
| `--port N` | Override the default port. |
| `--stop` | Stop the running viewer and exit. |
| `--rebuild` | Rebuild the container image from source, then start. Useful while the bundle is iterating. |
| `--help` | Print usage at the concept level — no mechanism jargon in the output. |

### Behaviour

| Step | Rule |
|---|---|
| 1 — OS guard | Refuse non-Silverblue. Exit 1. |
| 2 — Podman guard | If podman absent, instruct the reader to install it via the distribution's layering tool. Exit 1. |
| 3 — Image presence | If the viewer image is not yet built locally, build it from `images/calmecac/`. If already present, skip. |
| 4 — Container state | If running → log and exit 0. If stopped-but-exists → start. If absent → run fresh. |
| 5 — Port selection | Default **8088**. If taken, refuse with a helpful message (do not silently try another). |
| 6 — Open browser | Best-effort `xdg-open` if a desktop session is present. Non-fatal if it fails. |
| 7 — `--stop` | Stop and remove the container. Exit 0 whether or not it was running. |
| 8 — `--rebuild` | Rebuild the image, then flow into steps 3–6. |

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Viewer running (or cleanly stopped via `--stop`). |
| `1` | Precondition failed (wrong OS, missing podman, port taken). |
| `2` | Container failed to start for a reason that is not a precondition (image build error, runtime fault). |

## Container image

The image is an **industry-standard tiny httpd** serving static files. No server-side logic; Calmecac's "dynamic" feel is entirely client-side after the initial bundle load.

### Wisdom is knowing where to stop

> *"We will not create our own http server, instead we will prefer the industry standard rock solid Alpine super secure httpd version, or even Fedora's tiny container with httpd. We do not always know better."* — author directive

This is load-bearing. The rest of the stack this project authors is **Rust** (per the workspace-wide preference in `orchestrator/spec.md`); but serving static files over HTTP is a solved, battle-tested, decades-old problem. Writing a Rust HTTP server for Calmecac would be **noise masquerading as rigour** — fewer CVEs in the serving layer this year means a rock-solid upstream httpd, not a clever one-author reimplementation. The choice to stop at the trust boundary and use industrial-grade plumbing is itself a teachable meta-example (candidate, not canonized).

| Aspect | Choice |
|---|---|
| Base | `registry.fedoraproject.org/fedora-minimal:latest` (primary recommendation — see **Open questions** for the Alpine alternative). |
| Server | The distribution-provided `httpd` package. No custom HTTP code, no Rust server crate, no hand-rolled listener. |
| Source | `images/calmecac/Containerfile` in the repo. |
| Mounts | `output/calmecac-bundle/` is bind-mounted **read-only** at `/usr/local/apache2/htdocs/` (or distro equivalent). The container itself is immutable at runtime. |
| Hardening | `--read-only --cap-drop=ALL --security-opt=no-new-privileges --userns=keep-id --network=<bridge-bound-to-localhost>`. Port exposed only on `127.0.0.1:<port>`. Intentional — this is Season 2 substrate already in place. |
| Name | `tlatoani-tales-viewer` — ASCII per **TB03**. |

The container is ephemeral. State lives in the build output, not in the container. `--stop` and re-run is always safe. No writable paths inside the container; no code execution inside beyond the httpd itself.

### Trust-boundary placement

Calmecac runs in an **UNTRUSTED** hardened container — same trust model as every other non-Rust runtime in the project (per `isolation/spec.md`). The static content is produced by **trusted Rust code** (`tt-calmecac-indexer` — see **Concept index generation**) in the trusted toolbox, then mounted read-only into the UNTRUSTED httpd container. The boundary is clean: trusted code writes the bundle; untrusted runtime serves it. No write paths cross back; no code executes inside; network access is bound to localhost.

This is the same shape the orchestrator uses for ComfyUI and ai-toolkit (untrusted Python runtimes in hardened tiny containers). Calmecac's risk is lower (static files, not image generation), but the *shape is identical* — and that consistency is itself the teaching.

## Build output — the reader-side bundle

Served bundle lives at `output/calmecac-bundle/` and is **gitignored** (per `licensing/spec.md`: generated artefacts rebuild from source).

| Piece | Role |
|---|---|
| Static pages | The three UI views (Comic, Lesson, Rule) plus the Convergence tab and the Graph tab. |
| Client script | Traversal and rendering of the concept graph; view transitions; breadcrumb state. |
| Offline manifest | Makes Calmecac installable as a progressive app, with offline read-through of every shipped strip. |
| Service worker | Caches the bundle and strip art for offline use. Also emits the viewer's own telemetry (see **Observability of Calmecac itself**). |
| **Concept index** | `calmecac-index.json` — the generated concept graph. Single file, loaded on startup. |

The bundle is **fully static**. No databases, no server calls after load. This is intentional: Calmecac must be possible to host as plain files on any static host, because the teaching about observability should never depend on a live service.

## UI model

Calmecac presents the comic through three primary views and two secondary tabs. Every view enforces the concept-only abstraction rule.

### Primary views

| View | Entered from | What the reader sees |
|---|---|---|
| **Comic** (default) | Direct visit, or the episode hotspot | The strip as it appears on `www.`: three panels, three plates. The plates are clickable. This view is the bridge — what you saw on `www.` is what you see here, until you touch a plate. |
| **Lesson** | Clicking the bottom-left `[@Lesson S1-NNN]` hotspot | The teaching unfolds. Title, abstract, position on the curriculum (predecessors and successors as clickable chips), aha moment. Underneath, the lesson's coverage expands into a row of `[@trace rule:<name>]` chips — one per rule that governs this teaching. The strip sits alongside, so reader holds teaching and picture in one glance. Related strips that reinforce this lesson appear as small tiles. |
| **Rule** (what the reader knows as an `@trace rule:<name>`) | Clicking a `[@trace rule:<name>]` chip from Lesson view, or clicking that plate directly | For **visual rules** (e.g. the spelling rule, the symbol catalogue, the character canon): a gallery of every rendered artefact that cites this rule — title plates, speech bubbles, frames, reference art. For **non-visual rules** (the project-process rules like licensing, meta-examples, concept-curriculum): a convergence dashboard — the boring kind, deliberately. See **Boring dashboards**. Every rule view also shows the lessons that cite it and the neighbouring rules it references. |

Breadcrumbs are always present: *Comic → Lesson → Rule → …*. Back always works, in both directions. Every view has a visible "teaching" of what the reader is looking at, written at the concept level. *"You are looking at every place in the project where the spelling rule is upheld. Thirty-seven artefacts have been checked. Three are pending."*

### Secondary tabs

| Tab | Content |
|---|---|
| **Convergence** | A tree of historical dashboards — the same per-rule dashboards rendered in the Rule view, but plotted across changes. This is where the reader sees *the project converging toward its own rules*. Every data point clicks through to the state of that rule at that change. |
| **Graph** | A navigable diagram of the whole observability scaffold — lessons on one axis, rules on another, strips as nodes pulled to the lessons they instantiate and the rules they cite. The ultimate-observability view. Clicking any node drills into the matching Lesson or Rule. |

### Abstraction vocabulary (enforced in UI strings)

| Concept | Label the reader sees | Label the reader NEVER sees |
|---|---|---|
| A teaching | Lesson | "lesson spec", "lesson file" |
| A contract | Rule | "spec", "specification document", "markdown" |
| A published comic | Strip | "render", "artefact" |
| A linkage | Trace | (unchanged — the reader learned `@trace` from the comic) |
| A citation to a teaching | `@Lesson S1-NNN` | (unchanged — reader learned it from the comic) |
| A project edit | Change | "commit", hash, author name, timestamp of commit (a date is OK; a hash is not) |
| A pending teaching | Seed | (unchanged — concept exists in the curriculum) |
| A retired entry | Tombstone | (unchanged — concept exists in the curriculum) |

Labels in the left column are **allowed** because the reader encountered them inside the comic. Labels in the right column are **forbidden** in UI copy because they belong to the substrate the project deliberately hides from the reader at this abstraction level.

Copy-review discipline: every user-visible string in the bundle is reviewed against this table at build time. `tt-calmecac-indexer` emits the strings it will render (tooltips, empty states, error messages, breadcrumbs) as a flat list; a check step greps forbidden tokens and fails the build.

### Mobile-first

Most readers arrive from a phone link on `www.`. The Comic view must be usable first-touch on a phone; the Lesson and Rule views must be readable on a phone without horizontal scroll. The Graph view MAY degrade to a list on small screens — a graph is not always a diagram.

### Accessibility

- Every image carries alt text taken from the rendered strip's metadata.
- Every plate hit-region is also reachable by keyboard.
- Every chip is a real link with a legible label.
- Contrast meets the project's accessibility floor (same as `style-bible` governs for panel palettes).
- The viewer respects reduced-motion preferences — the Graph's force-directed layout freezes to a static arrangement when reduced-motion is requested.

## Seasonal layering — progressive disclosure by curriculum

Calmecac is not a static viewer. Its dashboards layer as the reader progresses through seasons, because the reader's questions evolve.

| Reader state | Calmecac surfaces |
|---|---|
| **Has finished Season 1** (lessons S1-100 … S1-1500) | Observability of *how work progresses*: lesson graph, rule coverage, convergence history, tombstone trails, OpenSpec as CRDT. Answers *"how was this made?"* |
| **Has finished Season 2** (future) | Adds observability of *how work progresses **safely***: trust-boundary visualization, container-graph diagram, privilege matrix (`--cap-drop=ALL`, `--userns=keep-id`, `--network=none` per role), offline-inference proof chart, verify-lint pass/fail timeline. Answers *"how do I know it's safe?"* |

Season 2's views **reuse** the same indexer + bundle contract; they surface telemetry the orchestrator was already emitting (container start events, verify-lint results, network-mode flags on each render) — see `orchestrator/spec.md` and `isolation/spec.md`. No new build pipeline; just additional dashboards that *unhide* once the reader has crossed the Season-1→Season-2 threshold.

The progressive-disclosure model is itself a small teaching: dashboards are for the reader you have now, not the reader you might eventually have. A Season-1 reader staring at a privilege matrix learns nothing; a Season-2 reader staring at it sees the proof that *"this comic was rendered using offline models"* isn't marketing.

Gating: the viewer reads a simple `reader_progress` cookie (or query string for share-linking) to decide which tier of views to expose by default. Every dashboard stays **reachable** for the curious — gating is default-visible, not access-control.

## The three hotspots — from plate to view

| Plate on a rendered strip | Calmecac destination |
|---|---|
| Bottom-left, top line `[@Lesson S1-NNN]` | Lesson view for S1-NNN |
| Bottom-left, bottom line `[@trace rule:<name>]` | Rule view for `<name>` |
| Bottom-right `Tlatoāni Tales #NN` | Strip permalink on `www.` (leaves Calmecac — it is the strip's public page) |
| Top-left title (opt-in) | Lesson view for the strip's primary lesson, if the strip declares `title_linkable: true` |

These targets come verbatim from `trace-plate/spec.md`; Calmecac is the resolver for every `calmecac_*_url` that spec emits.

## Boring dashboards

Some rules have no pictures. There is nothing for the reader to admire in a rule about process (`licensing`, `meta-examples`, `concept-curriculum`, `lesson-driven-development`). These rules still need observability — *especially* these, because the feedback loop consumes their convergence metrics even when no reader ever clicks them.

Calmecac renders these as **static dashboards** — metrics at the time of the last change, with a small historical series behind them. The design intent is explicit: the dashboards are *boring*, and that is the lesson. The feedback loop is the audience; readers are welcome but not courted.

| Dashboard element | Renders |
|---|---|
| Body length over changes | A line series of the rule's contract length at each change. Shape-of-a-number (S1-1000) applied to the rule itself. |
| Citation count over changes | How many places in the project cite this rule over time. Monotonic, usually. |
| Churn | Additions and removals per change, as a paired bar series. |
| Last change summary | A one-line, concept-level description of what the most recent change was for — no hash, no author, a date at most. |
| Pending seeds | Teachings or entries flagged as *pending author decision* that touch this rule. A visible queue of intent. |

Each dashboard point is clickable. Clicking navigates to a **retrospective view** — the Rule view as it would have appeared at that change. Retrospective views are read-only; breadcrumbs clearly mark them as historical.

*"Boring by design — the feedback loop reads this, not you. You can still read it."* — rendered as the panel's introductory copy.

The convergence data is **derived at build time**. It does not live in the repo; it is materialised from the history of the specs and strips. The derivation is explicitly handled by the index generator (see **Concept index generation**).

## Dependency graph visualization

The Graph tab presents the whole observability scaffold as a navigable diagram.

| Node type | Meaning | Rendering |
|---|---|---|
| Lesson | A teaching | Large, warm-palette node; label is the lesson's display name |
| Rule | A contract | Medium, cool-palette node; label is the rule's short name |
| Strip | A published comic | Small, paper-tone node; label is the strip's episode number |
| Seed | A pending teaching candidate | Ghosted node; label italicised |
| Tombstone | A retired entry | Outlined node; label struck-through |

| Edge type | Meaning |
|---|---|
| *teaches* | Strip → Lesson |
| *governs* | Rule → Strip or Rule → Lesson (rule constrains the production of this lesson) |
| *cites* | Rule → Rule, Lesson → Lesson |
| *reinforces* | Strip → Lesson (secondary; from a strip's `reinforces:` list) |
| *tombstones* | Tombstone ← Entry (retired supersedence) |

Rendering approach: a compact, tree-leaning force-directed layout. Static after initial settle, interactive on hover/click. The implementation choice (a dedicated graph-rendering library vs. hand-rolled vector drawing) is deferred to **Open questions**; the contract is the nodes, edges, and interactions above.

The Graph is the literal demonstration of S1-1000 one level further up — the dashboard that teaches its story is not just a per-rule panel, it is the whole graph.

## Concept index generation

`calmecac-index.json` is the single artefact the bundle reads on load. The index generator walks the repo at build time and produces a **concept-level** JSON description. This step is the abstraction boundary's enforcement point — it is where substrate vocabulary is *erased*, not where the UI learns to hide it.

The generator is a **Rust crate**, `tt-calmecac-indexer`, living alongside the other `tt-*` crates in the orchestrator workspace (see `orchestrator/spec.md`). It is invoked by the `tt-calmecac` binary, which writes `output/calmecac-bundle/calmecac-index.json` and optionally invokes `scripts/tlatoāni_tales.sh` to start the viewer container. This resolves the earlier open question about index-generator language: the workspace preference is Rust wherever possible, and nothing in this generator requires a Python dependency to justify the exception.

Shape (tentative, flagged in **Open questions**): `tt-calmecac-indexer` is a *library crate* (reusable by CI, by `tt-render`'s post-composition step, by future tooling); `tt-calmecac` is a *binary crate* that wires the library to argv and the launcher script. Splitting the responsibilities keeps the library pure — no argv, no stdout formatting, no process-spawning — and the binary thin.

### Inputs

| Source | Role |
|---|---|
| All rule contracts under `openspec/specs/` | Source of rules, their mutual citations, the lessons they trace. |
| All lesson specs under `openspec/specs/lessons/Sn-NNN-slug/` | Source of lessons, their seven-field bodies, coverage lists. |
| All strip proposals under `strips/NN-slug/` | Source of strips, their declared lesson + rule + reinforcements. |
| All strip metadata under `output/Tlatoāni_Tales_NN.json` | Source of rendered-artefact plate regions, alt text, captions. |
| `@trace rule:<name>` and `@Lesson S1-NNN` citations across the repo | Source of citations outside specs — from scripts, comments, commit messages. |
| The project's change history | Source of convergence metrics — body-length series, citation-count series, churn series. |

### Output shape (conceptual)

The index is a graph of typed nodes and typed edges, plus a per-node time series of convergence metrics. At the concept level:

| Key | Contents |
|---|---|
| `lessons` | Every lesson's id, display name, abstract, position (predecessors, successors), references-in-project coverage, aha moment. |
| `rules` | Every rule's short name, plain-language role, the lessons it traces, the rules it cites, its artefact gallery (for visual rules) or its convergence series (for non-visual rules). |
| `strips` | Every published strip's episode number, title, plate regions, primary lesson, primary rule, reinforced lessons, alt text. |
| `seeds` | Every candidate entry flagged *pending author decision* (from the specs' curation markers). |
| `tombstones` | Every retired id with the date of retirement and its last canonical meaning. |
| `edges` | Every typed relation between nodes (see the Graph table). |
| `convergence` | Per-node time series, one point per historical change, keyed by change date. |

### The substrate-erasing step

This is the load-bearing part. `tt-calmecac-indexer` reads substrate (markdown, paths, dates, hashes) and *emits concepts* (rules, lessons, changes). Filenames are parsed **and thrown away**; the index contains no path strings visible to the client. Change hashes are reduced to dates plus short human descriptions (taken from the first line of the change's summary). Author identities are dropped. The client never sees them because they do not exist in the file it loads.

This placement is deliberate. The UI does not need to remember to hide substrate — the substrate never reaches it. A future UI rewrite could not accidentally leak a path, because the path is not in the bundle. The erasure happens in trusted Rust, once, at build time — not in the untrusted httpd container, which only sees the already-abstracted bytes.

### When the index is regenerated

| Trigger | Behaviour |
|---|---|
| A completed render run by `tt-render` | The orchestrator invokes `tt-calmecac-indexer` as its final composition step, so Calmecac is always consistent with the last render. |
| `scripts/tlatoāni_tales.sh --rebuild` | Rebuilds the httpd container image AND re-runs `tt-calmecac` to refresh the index. |
| Manual invocation of `tt-calmecac` | Supported as an escape hatch — primarily for CI. |

`tt-calmecac-indexer` is **not** a web service. It runs at build time, emits the file, exits. The bundle never reaches back. The httpd container, which *is* long-running, never invokes the indexer — the trust boundary is preserved.

## Integration with the orchestrator

Calmecac composes downstream of `tt-render`. The orchestrator's render flow (see `orchestrator/spec.md`) ends with metadata emission for each rendered strip; immediately after, the orchestrator invokes `tt-calmecac-indexer` as a library call (same process) or `tt-calmecac` as a subprocess — both are Rust, both live in the same workspace. On a successful end-to-end run, the following are consistent:

| Surface | State |
|---|---|
| `output/Tlatoāni_Tales_NN.png` | Latest strip pixels. |
| `output/Tlatoāni_Tales_NN.json` | Latest strip metadata. |
| `output/calmecac-bundle/calmecac-index.json` | Every strip and spec reflected, every convergence series extended. |
| `output/calmecac-bundle/` | Served by the local container; deployable to production. |

This is the project's central consistency property for the viewer: *Calmecac always shows the comic as the orchestrator last produced it.* If the orchestrator did not run, Calmecac shows the previous state; it does not lie about what exists.

## Observability of Calmecac itself

Calmecac emits its own observability signals. This is deliberate meta-recursion — the viewer for the project's observability must itself be observable, or it fails its own lesson.

| Signal | Emitted by | Visible where |
|---|---|---|
| View transitions | Client script | A developer-only panel inside Calmecac (hidden behind a key combination). Visible to authors during iteration, hidden from ordinary readers. |
| Fetches of strip art | Service worker | Same panel. |
| Chip and node clicks | Client script | Same panel. |
| Build-time index summary | Index generator | A small "last change" banner on the Convergence tab — the concept-level summary of the most recent run. |

All signals stay **local**. No external telemetry, no third-party analytics, no outbound network beyond strip art and the index itself. Reader privacy is a feature — and, again, a Season 2 seed.

## Commit-history observability — the conversation made observable

Calmecac's third deep-disclosure layer (after Lesson views and Rule views) is the **Conversation** view: a browseable projection of the repo's commit history, filtered by `@trace spec:X` / `@Lesson S1-NNN` tags. The author's discussion with the tooling becomes a commit trail; that trail becomes a spec edit; that spec edit becomes a panel regeneration. Every layer is linked. Calmecac surfaces the full chain so a reader can follow **not only what was taught, but the argument that produced the teaching**.

| Entry point | Opens |
|---|---|
| Rule view → "Convergence history" | Per-Rule timeline of commits with `@trace spec:<this-rule>`, each point click-through to a minimal commit summary. |
| Lesson view → "Argument trail" | Per-Lesson timeline of commits with `@Lesson S1-NNN` OR `@trace spec:<any-spec-in-lesson-coverage>`, presenting the threaded evolution of the teaching. |
| Convergence tab → "Changes across the repo" | A repo-wide commit log, concept-level — each commit summarized by the specs/lessons it touched, no file paths, no hashes. |
| Strip view → "How this strip arrived at this state" | Commits that affected any input to this strip's panel_hash — lesson spec, style bible, character canon, LoRA manifest, the strip's own proposal. |

The UI still respects the abstraction rule: **no raw commit hashes in display text** (summaries are rendered as concept-level change descriptions), **no file paths**, **no markdown/filename leakage**. Commits are "changes" in the reader's vocabulary; they link outward to GitHub only if the reader explicitly asks, via an "open on GitHub" action that is itself a teachable moment about traceability.

The indexer is responsible for this projection: `tt-calmecac-indexer` walks `git log --follow` across all spec files and strip artefacts, extracts `@trace` / `@Lesson` tags from commit messages, and writes a `commits` section into `calmecac-index.json` with per-commit concept-level summaries. Commit messages that carry neither tag are still listed but pushed to a secondary tier — they're part of the history, but they did not shape a teaching they are tagged against.

This closes the loop on the author's workflow model: **discussion → spec update → commit → pipeline run → panel → strip → Calmecac view of the commit that started it all**. The observability surface now covers every layer of how the comic was made, including the conversation that made it.

The author uses the in-viewer panel to see which lessons and rules readers dwell on during development. If a chip is never clicked, that is telemetry. If a graph node is hovered for seconds, that is telemetry. The reader's experience is the input to the next iteration of the viewer — the loop, closed, one more time.

## URL shape

All Calmecac URLs live under `calmecac.tlatoani-tales.com`. The local launcher serves the same URL shapes from `http://localhost:<port>`.

| Shape | Resolves to |
|---|---|
| `/` | Home: recent strips, the Convergence tab as a sidebar hint, a link into the Graph. |
| `/strip/NN` | Comic view for strip NN. |
| `/lesson/S1-NNN` | Lesson view. |
| `/rule/<name>` | Rule view for `<name>`. |
| `/rule/<name>/at/<date>` | Retrospective Rule view at a historical change. |
| `/graph` | The full observability graph. |
| `/convergence` | The top-level convergence dashboard. |

The shape is stable. Deep links from the comic's plates, from social posts, and from future citations rely on this stability.

## UX abstraction rules — enforceable checklist

The build process runs the following checks before producing the bundle. Failures block the build.

| Check | Rule |
|---|---|
| `copy.no-markup-tokens` | No UI string contains tokens specific to the substrate's format (angle brackets around tags, file extensions, path separators that look like paths). |
| `copy.no-mechanism-vocabulary` | No UI string contains the forbidden words in the Abstraction vocabulary table's right column. |
| `copy.breadcrumbs-present` | Every non-home view has a breadcrumb block in its rendered template. |
| `copy.alt-text-present` | Every strip image has alt text sourced from its metadata. |
| `copy.contrast-floor` | Every text/background pair in the rendered pages meets the project's accessibility contrast floor. |
| `index.no-paths` | The emitted `calmecac-index.json` contains no path-shaped strings (values ending in `/`-separated segments resembling filesystem paths). |
| `index.no-hashes` | The emitted index contains no change-hash strings. Dates and concept-level summaries only. |
| `launcher.single-instance` | The launcher script's state transitions are idempotent; running twice from a clean state yields the same state. |

These checks are part of `tt-calmecac-indexer`'s output validation. They are not UI polish; they are the abstraction rule's teeth — enforced in trusted Rust before a byte ever reaches the httpd container.

## Future convergence

Each step refines; none invalidate this spec's contracts.

| Direction | Note |
|---|---|
| Live-watch mode | An optional future enhancement: the orchestrator publishes an SSE stream of render events, and the open viewer refreshes the affected strip in place. Because the MVP httpd container serves plain static files only (no SSE, no dynamic endpoints), this feature requires a **separate tiny reverse-proxy container** — still industry-standard, still a thing we do not author — to bridge the SSE stream in front of the static httpd. MVP does not block on this; the contract is specified now so the proxy's shape is already constrained. |
| Historical strip diff view | Every strip acquires a per-change visual history. A reader picks two changes; the viewer shows the strip side by side. Another boring dashboard, in visual form. |
| Multi-season navigation | Once Season 2 ships, the viewer's index gains a season dimension. No change to URL shape — `/lesson/S2-NNN` already fits. |
| Lesson-authoring mode | Editing a lesson directly inside Calmecac. Blocked on Season 2's teaching of safe, sandboxed edits — the viewer will not learn to write before the comic teaches how to write safely. |
| Annotation layer | A reader can leave a local annotation on a lesson or rule; annotations are stored client-side only. Deferred until a meaningful teaching beat for it exists. |

## Open questions for author

The following load-bearing choices are judgment calls made within this spec. Each deserves the author's eye before canonization. The previous "static-file server choice" and "index generator language" questions are **resolved** (industry-standard httpd in a hardened container; Rust, specifically `tt-calmecac-indexer`).

1. **Default local port.** Pick `8088`, but any free port in the 8000s is fine. Does the author have a project-wide port convention?
2. **Bundle path.** `output/calmecac-bundle/` follows the `output/` convention for generated artefacts. Is `output/` the right bucket, or does the author prefer a sibling directory (e.g. `site/`, `dist/`) to keep the artefact categories distinct?
3. **Containerfile location.** `images/calmecac/` is proposed as a sibling of `scripts/`. The author may prefer `scripts/calmecac/` (to colocate container source with launcher), or a top-level `container/` directory.
4. **Graph rendering library.** A dependency-light SVG renderer keeps the bundle small and the "concept-only" contract easy to enforce in code review. A mainstream graph-layout library gives a richer Graph tab with less bespoke code. The author should weigh bundle weight vs. implementation effort.
5. **Base image: Alpine vs Fedora-minimal.** The spec recommends **`registry.fedoraproject.org/fedora-minimal:latest`** for the httpd container. Rationale: the project's hosts are Fedora Silverblue, the trusted toolbox is Fedora, the orchestrator's UNTRUSTED runtime targets are specced as "hardened tiny Alpine or Fedora-minimal" (per `isolation/spec.md`), and staying on one base family per trust zone reduces the attack-surface footprint the author has to track mentally. The Alpine alternative (`alpine:latest` + `apache2` package, `~5 MB` base) is strictly smaller and is a perfectly defensible choice; the author picks. Either way, we do not author the server.
6. **One crate or two.** Current lean: `tt-calmecac-indexer` is a **library crate** (pure: inputs → index, no argv, no I/O beyond spec-reading); `tt-calmecac` is a **binary crate** that wires the library to argv and may also launch the httpd container via the script. Alternative: one crate with an optional binary target. The author decides; the two-crate shape is the Rust-workspace default and keeps the library reusable.

Where the spec does not prescribe one of these choices, Calmecac is still buildable from this contract. The open questions are refinements, not gaps.

## Trace

`@trace spec:calmecac, spec:orchestrator, spec:isolation, spec:trace-plate, spec:visual-qa-loop, spec:lessons, spec:tlatoāni-spelling, spec:licensing, spec:meta-examples, spec:seasons, spec:lesson-driven-development`
`@Lesson S1-1000`
`@Lesson S1-1500` *(proof-by-self-reference — a viewer that observes the project by being observable itself)*
