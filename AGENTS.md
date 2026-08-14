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

- `src/model.rs` owns platform-neutral domain types.
- `src/live/` reads `.als` data and must not depend on UI or filesystem APIs.
- Future `src/note/`, `src/mapping/`, and `src/diagnostics/` modules remain
  platform-neutral.
- `examples/` contains developer tools, not product interfaces.
- Future browser and Tauri adapters depend inward on the library.

Keep platform-specific file access outside the library. Prefer byte or `Read`-based
APIs and explicit sample-resolver interfaces.

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
