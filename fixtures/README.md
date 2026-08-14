# Fixtures

Keep only synthetic, minimal format fixtures in this directory.

- `live/` is reserved for generated `.als` fixtures.
- `note/` is reserved for generated `.abl` and `.ablbundle` fixtures.
- `paired/` documents fixtures representing the same musical state in both formats.
- `expected/` contains stable inspection and conversion outputs.

Do not commit personal projects, commercial presets, or copyrighted audio. Prefer
tests that generate small GZIP or ZIP inputs in memory when a binary fixture is
not necessary.
