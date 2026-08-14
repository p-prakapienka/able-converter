# Repository guidance

## Purpose

Able Converter maps selected Ableton Live Set content into Ableton Note projects.
The conversion core must remain usable from the CLI, WebAssembly, and a native
desktop adapter.

## Architecture

- `mapper-model` owns platform-neutral domain types.
- `mapper-live` reads `.als` data and must not depend on UI or filesystem APIs.
- `mapper-core` orchestrates inspection and conversion use cases.
- `mapper-cli` is a thin native adapter.
- Future Note, WebAssembly, and Tauri crates must depend inward on these layers.

Keep platform-specific file access outside the conversion core. Prefer byte or
`Read`-based APIs and explicit sample-resolver interfaces.

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
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
