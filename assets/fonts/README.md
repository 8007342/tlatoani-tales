# Tlatoāni Tales — font assets

<!-- @trace spec:licensing, spec:orchestrator, spec:trace-plate, spec:style-bible -->
<!-- @Lesson S1-1000, @Lesson S1-1500 -->

Font binaries referenced by `tt-compose` (the plate renderer) live here on the author's / builder's local disk. They are **NOT committed** — `.gitignore` excludes `*.ttf`, `*.otf`, `*.woff`, `*.woff2`. Licensing rule **R17** covers them; the license text is checked in at `LICENSES/OFL-1.1.txt` per rule **R18** so any container image baked from this repo carries the required attribution alongside the font binary when it's copied in at build time.

## What `tt-compose` expects

| Path on disk | Used for |
|---|---|
| `assets/fonts/atkinson-hyperlegible-regular.ttf` | Chrome plates (episode bottom-right, trace + lesson bottom-left) |
| `assets/fonts/atkinson-hyperlegible-bold.ttf`    | Emphasis within chrome |
| `assets/fonts/atkinson-hyperlegible-italic.ttf`  | Occasional stylistic variant |
| `assets/fonts/atkinson-hyperlegible-bolditalic.ttf` | Occasional stylistic variant |

The **title plate** at top-left is **not** drawn by this font — it's rendered stylized by Qwen-Image in the inference container and composited over the FLUX panel output. This font is strictly chrome: high legibility, license-friendly, accessible.

## How to provision

Download **Atkinson Hyperlegible** (Braille Institute) from <https://brailleinstitute.org/freefont>, unzip, rename the four TTF files to the lowercase / hyphenated forms in the table above, drop them into this directory, then delete the download archive. The repo itself carries only this README + the license text — the binaries are a local build input.

## License

**SIL Open Font License 1.1.** Full text at `/LICENSES/OFL-1.1.txt`.

The OFL permits embedding the font in derivative works (Calmecac container, rendered PNGs with composited text) without adopting the OFL for the derivative. **The license text must travel with the font** when distributed — which is why `LICENSES/OFL-1.1.txt` IS committed even though the binaries are not. That's also the pattern future vendored-asset licenses will follow (fonts, icons, sound, model weights where redistribution is legal) — REUSE-style convergence per `meta-examples/spec.md` ME04.

## Meta

The choice of Atkinson Hyperlegible is itself a small accessibility lesson: the typeface was designed specifically to increase character distinction at speed or low-vision — each letterform is maximally unlike the others. When Calmecac is served to readers, the chrome they read *is* the thing it teaches about: observability needs legibility. If a reader opens DevTools on Calmecac and notices the font, that's another small teach-by-example moment.

Candidate future meta-example (NOT canonized — curation rule): *"Typography as an accessibility lesson."*
