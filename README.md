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
- Parse track identifiers, types, effective/user names, colors, and ordered Session
  scene names, colors, tempo flags, and time-signature identifiers.
- Map Session clips into the canonical model with explicit diagnostics for invalid
  data and unsupported expression or automation.
- Resolve canonical track names and scene tempo overrides while reporting enabled,
  not-yet-decoded scene time signatures.
- Map a selected canonical Session grid into a typed Ableton Note Set subset with
  explicit track, scene, clip-length, and lossy-feature diagnostics.
- Serialize `Song.abl` and package it as a stored modern `.ablbundle` with a known
  Analog Drift Core Library preset reference.

## Structure

```text
src/
  lib.rs
  model/
    Live.rs            Ableton Live source models
    Internal.rs        Canonical format-neutral model
    Note.rs            Verified Ableton Note Set JSON subset
    InternalTest.rs
    mod.rs
  parser/
    LiveParser.rs      Public Ableton Live parser facade
    LiveParserTest.rs
    live/              Private parser state objects and builders
    mod.rs
  mapper/
    LiveToInternalMapper.rs
    InternalToNoteMapper.rs
    livetointernal/        Private per-call mapping context
    internaltonote/        Private per-call mapping context
    LiveToInternalMapperTest.rs
    mod.rs
  exporter/
    NoteExporter.rs    Song.abl and .ablbundle serialization
    NoteExporterTest.rs
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

To exercise the complete conversion path while the browser interface is incomplete:

```bash
cargo run --example convert -- path/to/project.als path/to/output.ablbundle
```

The converter prints all diagnostics and refuses to write an output bundle when any
error diagnostic is present.

The library API exposes `parser::live::LiveParser` for inspection and Session clip
extraction, followed by `mapper::livetointernal::LiveToInternalMapper` for canonical
conversion. `mapper::internaltonote::InternalToNoteMapper` creates destination Set
data, and `exporter::note::NoteExporter` writes Set JSON or a modern bundle. The
mappers and exporter are reusable stateless services; inputs are method arguments and
diagnostics live in private per-call contexts. File access remains the responsibility
of the caller so the pipeline can be reused from WebAssembly and desktop adapters.

The Note schema is undocumented. See `docs/note-format.md` for the implemented
evidence, provisional behavior, and physical acceptance check.

## Development

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Only synthetic fixtures belong in this repository. Real Live projects, Ableton
presets, and audio samples may carry personal or copyrighted material.
