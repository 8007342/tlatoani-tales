# Characters

Per-character assets for the LoRA training corpus. Each recurring character in
Tlatoāni Tales owns one subdirectory. The directory name is the character's
**ASCII slug** — the same token used as the `character` field in the manifest,
as the LoRA filename stem under `tools/loras/`, and as the trainer container
argument. Prose spelling (with macrons, e.g. *Tlatoāni*) lives in
author-facing markdown only; the filesystem rail is ASCII-only.

This is **TB03** made operational: the comic's `Tlatoāni` rides its macron
everywhere a human reads it; the filesystem, container names, and LoRA
filenames ride the lowest-common-denominator ASCII rail. See
`openspec/specs/character-loras/spec.md` §Trust boundary and
`crates/tt-lora/src/lib.rs` (`CharacterName::new` explicitly rejects the
macron form).

## Layout

```
characters/
├── README.md                     (this file)
├── tlatoani/
│   ├── README.md                 — character-specific notes + invariants
│   ├── lora-manifest.json        — reproducibility contract (committed)
│   └── references/               — 24–40 PNG reference sheets (committed art)
│       └── .gitkeep
└── covi/
    ├── README.md
    ├── lora-manifest.json
    └── references/
        └── .gitkeep
```

## What's committed, what's not

| Path                                          | Committed? | Why |
|-----------------------------------------------|------------|-----|
| `characters/<name>/references/*.png`          | yes        | Committed art, CC BY-SA 4.0 (licensing R05). The LoRA is reproducible from these. |
| `characters/<name>/lora-manifest.json`        | yes        | The reproducibility contract — hashes, hyperparams, trigger token. |
| `characters/<name>/README.md`                 | yes        | Character invariants + pipeline pointers. |
| `tools/loras/<name>-v<N>.safetensors`         | **no**     | Trained weights — build artefact, regenerable from references. Gitignored via root `.gitignore` (`tools/`). |
| `tools/loras/<name>-train.json`               | **no**     | Rendered ai-toolkit config — ephemeral, regenerable. |

The weights are **not** shipped. Readers who want them train their own from
the committed references. That is the reproducibility promise — see
`character-loras/spec.md` §Out of scope.

## Characters currently scaffolded

| Slug        | Trigger token       | Canon source                                  | State    |
|-------------|---------------------|-----------------------------------------------|----------|
| `tlatoani`  | `TlhAxolotlSage`    | `openspec/specs/character-canon/spec.md` §Tlatoāni | empty corpus — awaiting author curation |
| `covi`      | `CoviFigure`        | `openspec/specs/character-canon/spec.md` §Covi     | empty corpus — awaiting author curation |

## First-pass strategy

References can be populated either by author-curated art or by a first-pass
LoRA-less FLUX render of the canon description, filtered through the
reference-gate VLM check (see `openspec/specs/visual-qa-loop/spec.md` and
`character-loras/spec.md` §Reference gate). An image that itself fails canon
(a double-tailed Tlatoāni, a named Covi) is rejected before it can
contaminate the corpus.

## Specs

- `openspec/specs/character-canon/spec.md` — prose identity (invariants)
- `openspec/specs/character-loras/spec.md`  — weights identity (corpus, hyperparams, manifest schema)
- `openspec/specs/isolation/spec.md`         — trust boundary (why the trainer container is ASCII-only)
- `crates/tt-lora/src/lib.rs`                — Rust I/O for the manifest; `CharacterName` validator

@trace spec:character-canon, spec:character-loras, spec:isolation
