# Able Converter

Able Converter is a local-first tool for inspecting Ableton Live Sets and mapping
selected clips into Ableton Note projects.

The project is at the format-discovery and parser-foundation stage. The current
library can inspect the high-level structure of a gzip-compressed `.als` stream
without extracting it to disk.

## Current capabilities

- Read `.als` GZIP streams.
- Detect basic Live format metadata.
- Count MIDI, audio, group, return, and main tracks.
- Distinguish Session, Arrangement, and unclassified MIDI/audio clips.
- Read the global tempo when represented by Live's `Tempo/Manual` structure.

## Structure

```text
src/
  lib.rs
  model/
    Live.rs            Ableton Live source models
    Internal.rs        Canonical format-neutral model
    InternalTest.rs
    mod.rs
  parser/
    LiveParser.rs      Ableton Live GZIP/XML inspection
    LiveParserTest.rs
    mod.rs
  tools/
    Inspect.rs         Developer inspection utility
docs/
  implementation-plan.md
```

As conversion is implemented, `mapper/`, `exporter/`, and destination model files
will be added without splitting the project into crates prematurely. CamelCase
filenames provide Java-style scanability; their Rust module names stay idiomatic
snake_case.

The planned browser build will compile the Rust conversion library to WebAssembly.
The planned desktop build will call the same library natively from Tauri. Their
adapters will live in `src/web/` and `src/desktop/`, keeping application code under
`src/`.

## Inspecting a Set during development

```bash
cargo run --example inspect -- path/to/project.als
cargo run --example inspect -- path/to/project.als --json
```

The inspector is a development utility rather than a supported product CLI.

## Development

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Only synthetic fixtures belong in this repository. Real Live projects, Ableton
presets, and audio samples may carry personal or copyrighted material.
