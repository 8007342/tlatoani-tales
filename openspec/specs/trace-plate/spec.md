# Trace Plate

## Purpose

Every strip carries **three plates** that turn the comic into a self-describing, self-observable artefact. Each plate is a different door:

- **Top-left TITLE plate** — the reader's first touch. Full human-readable teaching name (e.g. `[Volatile is dangerous]`). Stylized, expressive, the lesson's meaning made visible before a single panel is read.
- **Bottom-left TRACE+LESSON plate** — the door inward. `[@Lesson S1-NNN]` on line 1, `[@trace spec:<name>]` on line 2. The grep-friendly, clickable handles to the CRDT of wisdom.
- **Bottom-right EPISODE plate** — the series pointer. `Tlatoāni Tales #NN`.

Together the three plates make every strip self-describing (the title reads as content), self-citing (the trace+lesson is a canonical link), and self-locating (the episode is a coordinate in the series). The plates are not chrome; they are the strip's observability surface.

Two layers of citation run through each plate:

| Layer | Marker | What it observes |
|---|---|---|
| Title | `[<Lesson display>]` | Reader ↔ teaching — what the strip is about, readable before any click |
| Technical trace | `@trace spec:<name>` | Code ↔ spec — which contract governs which implementation |
| Lesson trace | `@Lesson S1-NNN` | Spec ↔ teaching — which teaching a spec is in service of |

`@Lesson` is the observability *of the* `@trace` layer. The title is the observability *of the* `@Lesson` layer. Teach by example, literally in-panel.

## Motivation

- **Non-clickers** read the title alone and walk away wiser. Zero clicks, full payload.
- **Casual clickers** follow the lesson link at `calmecac.tlatoani-tales.com/lesson/S1-NNN` and see the teaching unfolded.
- **Deep clickers** follow the trace link into the spec view and see every artefact that cites it.
- Forces authors to pick exactly **one primary lesson + one primary spec + one title phrasing** per strip — itself a convergent discipline. No plates = strip doesn't ship.

## Layout — three plates

| Plate | Position | Content | Render engine |
|---|---|---|---|
| **Title** | Top-left (MAY float right) | `[<Lesson display name>]` | Qwen-Image (stylized text) composited over FLUX panel |
| **Trace + Lesson** | Bottom-left | Line 1 `[@Lesson S1-NNN]`, line 2 `[@trace spec:<name>]` | PIL (chrome) |
| **Episode** | Bottom-right | `Tlatoāni Tales #NN` | PIL (chrome) |

### Title plate (top-left, new)

- **Content**: the lesson's full display name, bracketed: `[Volatile is dangerous]`, `[The loop closes]`, `[Proof by self-reference]`.
- **Position**: top-left by default. MAY float to the right of the top band if panel composition demands (e.g. a character's face occupies the top-left of panel 1). Author declares `title_float: right` in `proposal.md` when the shift is needed.
- **Font**: stylized, expressive — **not** the chrome typeface used by the other two plates. Each title is free-styled for expressive tie to the lesson's meaning. Examples:
  - `[Volatile is dangerous]` — wobble, trembling strokes, a letter half-dissolved.
  - `[The loop closes]` — subtle circular motif, letters that bend along an arc.
  - `[Proof by self-reference]` — the `[` and `]` drawn as if the title quotes itself.
- **Chrome**: no plate border required by default. The title is an artistic element, not an info-box. An author MAY request a light scroll backing if legibility over a busy panel requires it (declared `title_backing: scroll` in the proposal).
- **Render engine**: Qwen-Image. FLUX renders pixels of faces, poses, and scenes beautifully, but its in-image text fidelity is unreliable; Qwen-Image is used specifically for the stylized title text, composited over the FLUX panel by the orchestrator. Requires the Qwen-Image model at `tools/ComfyUI/models/checkpoints/Qwen-Image/` (currently downloading; strip ships are blocked on its availability for titles).
- **Free-styled latitude**: authors are encouraged to make each title *feel* like the lesson. The QA check `plate.title-legible` enforces readability as a floor; beyond that, expressive choices are welcome.

### Trace + Lesson plate (bottom-left, retained)

- **Position**: bottom-left, symmetric to the episode plate at bottom-right.
- **Overlap**: all of panel 1 and ~12% of panel 2 (mirror of the episode plate).
- **Shape**: two-line stack on one scroll/banner motif. Height adjusts to fit both lines.
- **Styling**: cream scroll, dark ink, same typeface family as the episode plate. Line 1 (`@Lesson`) is primary — slightly bolder or larger than line 2 (`@trace`).
- **Text**:
  ```
  [@Lesson S1-100]
  [@trace spec:concept-curriculum]
  ```
- **Text wrapping**: both lines may wrap for unusually long names. Prefer wrapping the `@trace` line first (secondary).
- **Legibility constraint**: MUST NOT obscure character faces or key action. Shrink to min 40% of episode-plate area if forced.

### Episode plate (bottom-right, unchanged)

- **Position**: bottom-right.
- **Overlap**: all of panel 3 and ~12% of panel 2.
- **Text**: `Tlatoāni Tales #NN`.
- **Styling**: matches the trace+lesson plate (chrome pair).

## Selection rule (per strip)

Each strip declares its three plates in `proposal.md`:

```yaml
lesson:       S1-NNN                # must exist in seasons/spec.md registry
title:        <Display name>        # the bracketed string rendered top-left
reinforces:   []                    # optional: other lesson IDs this strip echoes
trace_spec:   <spec-name>           # the governing OpenSpec for this strip
title_float:  left                  # left (default) | right
title_backing: none                 # none (default) | scroll
```

Selection priority:

1. The lesson whose **primary strip** is this one in `seasons/spec.md` (obvious case).
2. For the spec: the one governing the **visual prop** introduced/highlighted, OR the **concept** being taught (usually `concept-curriculum`), OR `meta-examples` if the joke is meta about the project itself.
3. The declared lesson and spec MUST be consistent — the spec SHOULD appear in the lesson's coverage list. QA check `plate.lesson-spec-aligned` enforces this.
4. The declared title MUST match the lesson's `Display` column in `lessons/spec.md`. QA check `plate.title-matches-declared` enforces this.

## Initial strip-to-plate mapping

| Strip | Title (displayed top-left) | Lesson ID (bottom-left line 1) | Trace spec (bottom-left line 2) |
|---|---|---|---|
| TT #01 | `[Volatile is dangerous]` | `S1-100` | `concept-curriculum` |
| TT #02 | `[Save means findable]` | `S1-200` | `concept-curriculum` |
| TT #03 | `[Memory lives in history]` | `S1-300` | `meta-examples` |
| TT #04 | `[Discrete time]` | `S1-400` | `meta-examples` |
| TT #05 | `[Edits that reconcile]` | `S1-500` | `licensing` |
| TT #06 | `[Ask in writing]` | `S1-600` | `concept-curriculum` |
| TT #07 | `[Loops need aim]` | `S1-700` | `concept-curriculum` |
| TT #08 | `[See the now]` | `S1-800` | `visual-qa-loop` |
| TT #09 | `[Logs are ingredients]` | `S1-900` | `visual-qa-loop` |
| TT #TBD | `[Dashboards must add observability]` | `S1-950` | `visual-qa-loop` |
| TT #10 | `[Shape has meaning]` | `S1-1000` | `visual-qa-loop` |
| TT #11 | `[Meaning is operable]` | `S1-1100` | `visual-qa-loop` |
| TT #12 | `[The loop closes]` | `S1-1200` | `visual-qa-loop` |
| TT #13 | `[Monotonic convergence]` | `S1-1300` | `meta-examples` |
| TT #14 | `[Proof by self-reference]` | `S1-1400` | `meta-examples` |

Mapping confirmed in each strip's `proposal.md`. Lessons registry is authoritative — see `lessons/spec.md` and `seasons/spec.md`.

## Clickable semantics (published comic)

The rendered PNG is static pixels. The **published comic** at `www.tlatoani-tales.com` wraps each PNG in HTML with an image-map that makes the three plate regions clickable. The orchestrator emits exact pixel coordinates in `METADATA.json` so the publishing site builds the image-map without re-parsing pixels.

| Plate region | Click target | Behavior |
|---|---|---|
| Top-left title | *(default: none)* | Artistic element; no click target unless opted in per-strip via `title_linkable: true`. When opted in, links to `calmecac.tlatoani-tales.com/lesson/S1-NNN`. |
| Bottom-left `[@Lesson S1-NNN]` sub-region | `calmecac.tlatoani-tales.com/lesson/S1-NNN` | Opens the lesson's observability view (see Calmecac contract below). |
| Bottom-left `[@trace spec:<name>]` sub-region | `calmecac.tlatoani-tales.com/spec/<name>` | Opens the spec view. |
| Bottom-right episode | `/tt/NN` (on the public site) | Opens the per-strip permalink page. |

The bottom-left plate's two lines are **two independent click regions**. The image-map's `trace_lesson` region is split into `trace_lesson.lesson_line` and `trace_lesson.trace_line` — see the METADATA schema.

## Calmecac view contract

The `calmecac.` subdomain is the observability mirror of the public comic. `calmecac` means *house of learning*; the naming honors the Nāhua pedagogical institution. Two view modes:

### Lesson view (`/lesson/S1-NNN`)

- Renders the **same comic** as on the public site, but in **observability mode**: plates become expanded interactive chips, not static text.
- The `[@Lesson S1-NNN]` plate unfolds into a horizontal row of `[@trace spec:A] [@trace spec:B] [@trace spec:C] …` chips — one per spec in the lesson's coverage list (see `lessons/spec.md`).
- Clicking any `[@trace spec:X]` chip navigates to the Spec view.
- The title plate and episode plate retain their visual position but display coverage-count badges (e.g. title chip shows `7 specs` under it).

### Spec view (`/spec/<name>`)

- Replaces the three-frame comic with a **gallery of every rendered element that cites the spec**: frames, titles, speech bubbles, file listings, commit messages, whatever surfaces the trace.
- Example: clicking `[@trace spec:tlatoāni-spelling]` shows every place `Tlatoāni` is correctly rendered with a macron — title plates that contain the name, speech bubbles that carry it, source files where it appears, commits where it was corrected.
- The gallery is preceded by **metadata of related clickable specs**: the spec's own `@trace` declarations, its `@Lesson` citations, and links back to every lesson whose coverage list includes it.
- Some specs have no visual artefacts to gather (e.g. `concept-curriculum`, `meta-examples` in the abstract). Those render as **convergence-data views**: histograms of delta statistics from previous commits, spec-mutation timelines, "boring dashboards." Explicitly by design — the boring dashboards feed the feedback loop, not reader appreciation. Calmecac shows them because observability isn't only for humans; it's also for the orchestrator's next iteration.
- Calmecac stays at the **concept abstraction level**. It does NOT expose markdown filenames, directory paths, or tooling mechanism to the reader. Readers see `@trace spec:X` as a concept chip; the backend reads the markdown file to populate it, but the UI never surfaces that detail. The reader's mental model is *lessons and specs as ideas*, not *files on disk*.

## METADATA schema (emitted per strip)

```jsonc
{
  "strip":              "TT #NN",
  "title":              "Volatile is dangerous",        // display name, no brackets
  "title_display":      "[Volatile is dangerous]",      // exactly as rendered in-panel
  "title_render_model": "Qwen-Image",                   // the stylized text engine
  "title_float":        "left",                         // left | right
  "title_linkable":     false,                          // default; per-strip opt-in

  "lesson":             "S1-NNN",
  "lesson_display":     "Volatile is dangerous",
  "lesson_search_url":  "https://github.com/8007342/tlatoani-tales/search?q=%40Lesson+S1-NNN&type=code",
  "lesson_spec_url":    "https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/lessons/spec.md",
  "calmecac_lesson_url":"https://calmecac.tlatoani-tales.com/lesson/S1-NNN",

  "trace_spec":         "<name>",
  "trace_search_url":   "https://github.com/8007342/tlatoani-tales/search?q=%40trace+spec%3A<name>&type=code",
  "trace_spec_url":     "https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/<name>/spec.md",
  "calmecac_spec_url":  "https://calmecac.tlatoani-tales.com/spec/<name>",

  "plate_regions": {
    "title":         { "x": 0,    "y": 0,    "w": 620,  "h": 140 },
    "trace_lesson": {
      "x": 0, "y": 860, "w": 780, "h": 180,
      "lesson_line": { "x": 12, "y": 872, "w": 760, "h": 78 },
      "trace_line":  { "x": 12, "y": 958, "w": 760, "h": 72 }
    },
    "episode":       { "x": 1180, "y": 860, "w": 720,  "h": 180 }
  },

  "concepts_taught":    ["Cxx"],
  "concepts_assumed":   ["Cxx"],
  "reinforces_lessons": [],

  "alt_text":           "<accessible description of all three panels, naming all three plates>",
  "caption":            "Tlatoāni Tales #NN — [Title] — @Lesson S1-NNN / @trace spec:<name>"
}
```

All pixel coordinates are in the composited PNG's coordinate space (origin top-left). The publishing site reads `plate_regions` verbatim to build the HTML image-map; no pixel re-parsing.

## URL forms

| Form | Template | Where used |
|---|---|---|
| Lesson GitHub search | `https://github.com/8007342/tlatoani-tales/search?q=%40Lesson+S1-NNN&type=code` | Caption, metadata |
| Lesson registry | `https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/lessons/spec.md` | Metadata |
| Calmecac lesson view | `https://calmecac.tlatoani-tales.com/lesson/S1-NNN` | Plate click target, metadata |
| Trace GitHub search | `https://github.com/8007342/tlatoani-tales/search?q=%40trace+spec%3A<name>&type=code` | Caption, metadata |
| Spec file direct | `https://github.com/8007342/tlatoani-tales/blob/main/openspec/specs/<name>/spec.md` | Metadata |
| Calmecac spec view | `https://calmecac.tlatoani-tales.com/spec/<name>` | Plate click target, metadata |

Search URLs are preferred in captions; calmecac URLs are the click targets on the published comic.

## Propagation cost

Adding the title plate retroactively invalidates every strip's layout again. Per `meta-examples/spec.md` ME12 already catalogued, this is a continuing propagation event. Strip #01's demo will need yet another retro render (its third composition generation: pre-plate → two-plate → three-plate). The commit history is Lamport evidence (ME01) of the convergence, and the repo is again in a known-not-yet-convergent state. Live C07 + C12.

## QA integration

VLM checks (defined in `visual-qa-loop/spec.md`):

| Check | Enforces |
|---|---|
| `plate.title-present` | A top-left title plate exists with the declared display name |
| `plate.title-legible` | Title text parses under Qwen-Image rendering |
| `plate.title-matches-declared` | Rendered title string matches `title_display` in proposal.md |
| `plate.title-position-valid` | Title is top-left by default; if `title_float: right`, it is top-right; never top-center, never bottom |
| `plate.trace-present` | Bottom-left plate exists with the `@trace` line |
| `plate.trace-legible` | `@trace` text parses |
| `plate.trace-content` | `@trace` text matches declared `trace_spec` in proposal.md |
| `plate.lesson-present` | The plate has the `@Lesson` line |
| `plate.lesson-legible` | `@Lesson` text parses |
| `plate.lesson-id-valid` | ID is in the `seasons/spec.md` registry and `lessons/spec.md` registry |
| `plate.lesson-spec-aligned` | Declared spec appears in the lesson's coverage list |
| `plate.symmetry` | Bottom-left and bottom-right plates visually matched as a pair |
| `plate.title-regions-emitted` | `plate_regions.title` coordinates present and non-empty in METADATA |

## Trace

`@trace spec:trace-plate, spec:style-bible, spec:visual-qa-loop, spec:lessons, spec:seasons, spec:meta-examples`
`@Lesson S1-1400` *(self-reference — trace-plate is literally observability-of-the-observability)*
