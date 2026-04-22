# Tlatoāni Tales — font assets

<!-- @trace spec:orchestrator, spec:trace-plate, spec:style-bible -->
<!-- @Lesson S1-1000, @Lesson S1-1500 -->

The `tt-compose` crate reads fonts from this directory at runtime (NOT via
`include_bytes!` — the crate must build cleanly without the font files
present). Missing fonts produce a clear `TtError::Infra` pointing the author
at this README.

## Expected files

| Path (relative to `assets/fonts/`)     | Purpose                                  | License                                |
|---------------------------------------|------------------------------------------|----------------------------------------|
| `atkinson-hyperlegible-regular.ttf`   | Chrome plates (trace+lesson, episode)   | SIL Open Font License 1.1 / CC BY 4.0 |
| `title-stylized-regular.ttf`          | Title plate (top-left). Any expressive display face. | varies — document per font           |

Per `calmecac/spec.md` and `orchestrator/spec.md`, Atkinson Hyperlegible is the
canonical chrome typeface (maximal character-shape distinction → best
legibility for readers using screen magnifiers or with low vision). Download
from <https://brailleinstitute.org/freefont> and commit the TTF alongside its
license text.

## Title font

The title plate is expressive per `trace-plate/spec.md` — each strip's title
`[Volatile is dangerous]`, `[The loop closes]`, etc. benefits from a stylized
face. For the composition test path, a single fallback file
`title-stylized-regular.ttf` is sufficient; long-term, each strip's
`proposal.md` may specify a title font override.

If the title font is missing, `load_embedded()` reuses the chrome font with an
emitted warning — the strip still renders, just without stylized title.

## TODO(pin)

- Commit `atkinson-hyperlegible-regular.ttf` with checksum pinned in this
  README. Bundle license text as `LICENSE-ATKINSON.txt`.
- Pick a title display face with a compatible license; commit with
  provenance note.
