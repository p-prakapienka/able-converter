# Repository guidance

## Purpose

Able Converter maps selected Ableton Live Set content into Ableton Note projects.
The conversion library must remain usable from WebAssembly and a native desktop
adapter.

See `docs/implementation-plan.md` for product scope, format strategy, and delivery
milestones. Update it when an architectural or scope decision changes.

## Architecture

The project starts as one Rust crate. Use ordinary modules until a real target or
dependency boundary justifies extracting another crate.

- `src/model/` contains source, canonical internal, and destination models.
- `src/parser/` converts source bytes into a source model.
- `src/mapper/` converts source models to the canonical model and the canonical
  model to destination models.
- `src/exporter/` serializes destination models into project bytes.
- `src/tools/` contains developer targets, not product interfaces.
- Future browser and Tauri adapters depend inward on the library.

Keep platform-specific file access outside the library. Prefer byte or `Read`-based
APIs and explicit sample-resolver interfaces.

The conversion path is always:

```text
source bytes -> source parser -> source model -> source-to-internal mapper
-> internal model -> internal-to-target mapper -> target model -> exporter
```

Do not add direct format-pair mappers such as `LiveNoteMapper`; all formats use the
canonical internal model.

## Source organization

- Keep all Rust implementation and test code under `src/`.
- Use lowercase folder and module names.
- Use UpperCamelCase implementation filenames such as `Live.rs`,
  `LiveParser.rs`, and `NoteExporter.rs`.
- Connect CamelCase paths to snake_case Rust module names with explicit `#[path]`
  declarations in the nearest `mod.rs`; do not suppress naming lints globally.
- Put tests in separate sibling files such as `LiveParserTests.rs` and register
  them with `#[cfg(test)]` from the nearest `mod.rs`.
- Create future format, mapper, and exporter files only when implementation begins.

## Conversion rules

- Never discard unsupported musical data silently.
- Report every lossy conversion or unsupported feature.
- Treat Session and Arrangement content as distinct sources.
- Use synthetic, minimal fixtures. Do not commit copyrighted projects, presets,
  or audio samples.
- Preserve unknown format data when practical, but never invent semantics for
  undocumented fields.

## Validation

Before committing, run:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```
