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
- Parse Session clip placement, bounds, loop settings, notes, velocity, probability,
  release velocity, and enabled state into a Live-specific source model.
- Map Session clips into the canonical model with explicit diagnostics for invalid
  data and unsupported expression or automation.

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
  mapper/
    LiveToInternalMapper.rs
    LiveToInternalMapperTest.rs
    mod.rs
  tools/
    Inspect.rs         Developer inspection utility
docs/
  implementation-plan.md
```

As conversion is implemented, `mapper/`, `exporter/`, and destination model files
will be added without splitting the project into crates prematurely. CamelCase
filenames and lowercase Rust module names provide Java-style scanability.

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

The library API exposes `parser::live::LiveParser` for inspection and Session clip
extraction, followed by `mapper::livetointernal::LiveToInternalMapper` for canonical
conversion. Both are cohesive objects that own their operation input or state. File
access remains the responsibility of the caller so they can be reused from
WebAssembly and desktop adapters.

## Development

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Only synthetic fixtures belong in this repository. Real Live projects, Ableton
presets, and audio samples may carry personal or copyrighted material.
