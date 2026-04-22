```
  ________    ___  __________  ___    _   ______   _________    __    ___________
 /_  __/ /   /   |/_  __/ __ \/   |  / | / /  _/  /_  __/   |  / /   / ____/ ___/
  / / / /   / /| | / / / / / / /| | /  |/ // /     / / / /| | / /   / __/  \__ \ 
 / / / /___/ ___ |/ / / /_/ / ___ |/ /|  // /     / / / ___ |/ /___/ /___ ___/ / 
/_/ /_____/_/  |_/_/  \____/_/  |_/_/ |_/___/    /_/ /_/  |_/_____/_____//____/ 

                          ·  Tlatoāni  Tales  ·
```

*A 3-panel webcomic that teaches you to build software with AI — made by the very thing it teaches.*

---

The comic ships in strips. Each strip is one lesson. The ladder climbs from *clean context windows are fragile* to *ask your AI for `monotonic_convergence`*. No math required.

## Structure

- `openspec/specs/` — the canon: style, characters, symbols, curriculum, arc, ledger, licensing
- `strips/NN-slug/` — per-strip scripts, prompts, reference sheets (as they ship)
- `output/` — composited strips, regenerated from specs (gitignored)
- `scripts/` — the image-gen driver
- `characters/` — reference art for Tlatoāni and Covi (LoRA training corpus)

Every rendered panel is content-addressed. Every spec change is a git commit — a Lamport tick. If a property mutates, every panel that referenced it is invalidated and re-rendered. The comic is produced by the discipline it teaches. That's the point.

## Two licenses (yes, on purpose — see TT 15/15)

- **Code** — [GPL-3.0-or-later](./LICENSE). Shell, Python, any functional tooling.
- **Everything else** — [CC BY-SA 4.0](./LICENSE-ART). Markdown, strip scripts, comic art, reference sheets, specs.
- Machine-readable mapping: [`openspec/specs/licensing/spec.md`](openspec/specs/licensing/spec.md).

The mapping is a tiny CRDT. New file types get new rules; rules don't conflict because each pattern is its own cell. The license set converges as the project grows. That's the joke. (Also: that's a free lesson.)

## Status

Work in progress. Strip 01/15 shipped as a demo from ChatGPT's image mode; strips 02/15–15/15 are being authored and rendered from the specs in this repo, on a local ComfyUI + FLUX.1-schnell + Qwen-Image stack with per-character LoRAs.

Watch the commit history. It's the same Lamport clock the comic will eventually name in panel 3 of TT 04/15.
