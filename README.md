# Able Converter

Able Converter is a local-first tool for inspecting Ableton Live Sets and mapping
selected clips into Ableton Note projects.

The project is at the format-discovery and parser-foundation stage. The current
CLI can inspect the high-level structure of a gzip-compressed `.als` file without
extracting it to disk.

## Current capabilities

- Read `.als` GZIP streams.
- Detect basic Live format metadata.
- Count MIDI, audio, group, return, and main tracks.
- Distinguish Session, Arrangement, and unclassified MIDI/audio clips.
- Read the global tempo when represented by Live's `Tempo/Manual` structure.
- Emit a human-readable or JSON inspection report.

## Workspace

```text
crates/
  mapper-model/   Platform-neutral project and inspection models
  mapper-live/    Ableton Live archive and XML inspection
  mapper-core/    Application use cases
  mapper-cli/     Command-line adapter
```

The planned browser build will compile the Rust conversion core to WebAssembly.
The planned desktop build will call the same core natively from Tauri.

## Usage

```bash
cargo run -p mapper-cli -- inspect path/to/project.als
cargo run -p mapper-cli -- inspect path/to/project.als --json
```

## Development

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Only synthetic fixtures belong in this repository. Real Live projects, Ableton
presets, and audio samples may carry personal or copyrighted material.
