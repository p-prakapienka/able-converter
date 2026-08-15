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
- Future `src/web/` and `src/desktop/` adapters depend inward on the library.

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
- Use lowercase folder names.
- Use UpperCamelCase implementation filenames such as `Live.rs`,
  `LiveParser.rs`, and `NoteExporter.rs`.
- Use Java package-style lowercase module identifiers, concatenating words without
  underscores. Use lowerCamelCase for functions, methods, fields, variables,
  arguments, and test functions. Keep types and traits UpperCamelCase.
- Connect CamelCase paths to lowercase Rust module names with explicit `#[path]`
  declarations in the nearest `mod.rs`.
- Add `#![allow(non_snake_case)]` at each crate root because this project deliberately
  uses Java naming. Do not suppress any broader warning category.
- Put tests in separate sibling files such as `LiveParserTest.rs` and register
  them with `#[cfg(test)]` from the nearest `mod.rs`.
- Keep a public parser facade such as `LiveParser.rs` focused on its parser
  struct/impl pair. Put its private collaborator objects in a lowercase package
  folder such as `src/parser/live/`, with one UpperCamelCase file per struct/impl.
- Keep application code under `src/`; use `src/web/` for the browser adapter and
  `src/desktop/` for the Tauri adapter.
- Create future format, mapper, and exporter files only when implementation begins.

## Object design

- Implement parsers, mappers, exporters, and other architectural components as
  cohesive structs with constructors and use-case methods.
- Let these objects own meaningful input, configuration, dependencies, or operation
  state; do not create empty structs as static utility namespaces.
- Keep models data-oriented unless they enforce domain invariants or behavior.
- Put behavior belonging to an architectural component in private methods on that
  component, even if a method does not yet access fields. Module-level privacy is not
  object encapsulation. Keep a private module function only when it is genuinely
  independent of every object in that module.
- Use traits only for genuine substitution or dependency boundaries, and prefer
  composition over inheritance-shaped designs.
- Keep public application workflows object-oriented instead of accumulating public
  module functions.

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
