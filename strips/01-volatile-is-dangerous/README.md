# strips/01-volatile-is-dangerous

First per-strip directory. Walked example of the layout every future strip
follows. Not user-facing; for the orchestrator (`tt-render` / `tt-specs`)
and for future authoring agents.

## What lives here

```
strips/01-volatile-is-dangerous/
├── README.md                    — this file
├── proposal.md                  — the strip's canonical author artefact
│                                  (YAML frontmatter + body; declares the
│                                  three plates, panels, dialogue, alt
│                                  text, caption, and @trace / @Lesson)
├── metadata-template.json       — sketch of output/Tlatoāni_Tales_01.json
│                                  (the shape tt-metadata emits; useful for
│                                  authors to calibrate against before a
│                                  real render exists)
└── panels/
    ├── panel-1.prompt.md        — per-panel generator prompts, one per
    ├── panel-2.prompt.md          panel, each with YAML frontmatter
    └── panel-3.prompt.md          (seed, base_model, character_loras,
                                    qwen_for_text) plus positive prompt,
                                    negative prompt, and expected-checks
                                    list sourced from visual-qa-loop/spec.md.
```

After a real render, these siblings appear (authored by the orchestrator,
not by humans):

```
├── qa-log.jsonl                 — one line per iteration per panel
│                                  (visual-qa-loop/spec.md)
├── (panels/panel-N.png)         — intermediate cached art, may live in
│                                  cache/panels/<hash>.png instead
└── output/Tlatoāni_Tales_01.png + .json
                                 — the composited strip + full METADATA
                                  (written to repo-root output/, not here)
```

## Convention for future strips

1. **Directory name**: `NN-slug` where `NN` is the zero-padded strip
   number and `slug` is kebab-case of the lesson slug. Stable across the
   strip's life.
2. **`proposal.md`** is authoritative. It declares:
   - the three plates (title / trace+lesson / episode) per
     `trace-plate/spec.md`,
   - the panels and their blocking,
   - the dialogue (one line per panel, exact),
   - the accessible alt text,
   - the publishing caption,
   - `@trace` and `@Lesson` annotations at the end of the body.
3. **`panels/panel-N.prompt.md`** — one markdown file per panel, with
   distinct reproducible seeds. Never rely on panel cross-talk; each
   prompt is self-contained. Negative prompts must explicitly guard
   against this series' known drift modes: **double tail on Tlatoāni**,
   **named face / hair / gender markers on Covi**, **palette drift**,
   and **plates rendered inside the raw panel** (plates are composited
   separately — never by the per-panel model).
4. **`metadata-template.json`** is optional but recommended for the first
   strip authored against a spec shift. It gives the orchestrator a
   known-good shape to diff its emission against.
5. **Never commit** trained LoRA weights, rendered PNGs, `qa-log.jsonl`,
   or `cache/panels/`. Those are build artefacts; `proposal.md` and
   `panels/*.prompt.md` are source.

## Hard constraints (applied during retro-spec authoring)

- No OpenSpec spec was modified.
- No images or binaries were touched.
- No `git add` / `git commit` / `git push` was run.
- `Tlatoāni` uses the macron in all prose; LoRA trigger tokens
  (`TlhAxolotlSage`, `CoviFigure`) are ASCII-only, matching
  `character-loras/spec.md`.
- Seeds: panel 1 = `314159`, panel 2 = `271828`, panel 3 = `161803` —
  three distinct reproducible values.
- Image generation was NOT attempted; these are author artefacts the
  orchestrator will consume.

## Trace

`@trace spec:orchestrator, spec:trace-plate, spec:style-bible, spec:character-canon, spec:symbol-dictionary, spec:concept-curriculum, spec:narrative-arc, spec:visual-qa-loop`
`@Lesson S1-100`
