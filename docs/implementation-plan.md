# Able Converter implementation plan

## Goal

Build a local-first application that reads Ableton Live Sets, previews MIDI clips
in a piano roll, maps selected Session or Arrangement content into Ableton Note's
track and scene structure, and exports a valid `.ablbundle`.

The converter must run entirely on the user's device:

- as Rust compiled to WebAssembly in the browser;
- as native Rust behind a Tauri desktop application;
- through small developer examples while the user interfaces are incomplete.

## Product constraints

Ableton Note currently limits a Set to eight tracks, eight scenes, and clips of up
to sixteen bars. The mapper must never silently truncate content that exceeds these
limits.

Note projects use `.abl` for Set data and `.ablbundle` for a bundle containing Set
data, samples, and metadata. Their JSON schema is undocumented and must be inferred
from controlled fixtures.

The first supported source is Ableton Live 11 and 12 `.als`: GZIP-compressed XML.

## Architecture

Start with one `able-converter` crate. Use Rust modules for internal boundaries and
extract crates only when a real build-target or dependency constraint appears.

```text
src/
  lib.rs
  model/
    Live.rs
    Internal.rs
    Note.rs                 added with the Note writer
    *Tests.rs
    mod.rs
  parser/
    LiveParser.rs
    *Tests.rs
    mod.rs
  mapper/
    LiveToInternalMapper.rs
    InternalToNoteMapper.rs
    *Tests.rs
    mod.rs
  exporter/
    NoteExporter.rs
    *Tests.rs
    mod.rs
  tools/
    Inspect.rs
web/          future browser frontend and WebAssembly adapter
src-tauri/    future desktop adapter
```

The library accepts bytes or `Read` implementations and returns typed data. It must
not depend on browser, Tauri, or native filesystem APIs. Platform adapters provide
files and implement sample resolution.

Implementation and test filenames use UpperCamelCase for Java-style discoverability.
Folders and Rust module identifiers remain lowercase snake_case. `mod.rs` files use
explicit `#[path]` declarations to connect the two conventions without disabling
Rust naming lints.

Add files only when they contain real implementation. For example, `Caustic.rs` and
its parser belong under these same boundaries when Caustic work begins, not as empty
placeholders today.

## Canonical model

Live XML and Note JSON must not map directly to each other. Both formats map through
a canonical project model containing:

- project metadata, tempo, key, and scale;
- source tracks and mixer state;
- Session scenes and clip slots;
- Arrangement clips and timeline ranges;
- normalized MIDI notes, loops, and supported automation;
- instruments, effects, and sample references;
- diagnostics for unsupported or lossy data.

Musical positions are represented in quarter-note beats. Format-specific numeric
representations are converted only at input and output boundaries.

The concrete conversion pipeline is:

```text
.als bytes
  -> parser/live
  -> model/live
  -> mapper/live_to_internal
  -> model/internal
  -> mapper/internal_to_note
  -> model/note
  -> exporter/note
  -> .ablbundle bytes
```

This avoids pairwise mappers such as `LiveNoteMapper`. A future Caustic importer only
needs `CausticParser` and `CausticToInternalMapper`; the Note half is reused.

## Format discovery

Create paired synthetic fixtures for every mapped property:

1. Create the smallest possible Set in Note.
2. Export `.abl` and `.ablbundle`.
3. Open the same Set in Live and save it as `.als`.
4. Change exactly one property and repeat.
5. Diff the Note JSON and Live XML.
6. Record verified semantics in `docs/` and encode them in golden tests.

Fixtures must cover notes, loops, track state, Session placement, samples, supported
devices, automation, and format limits. Do not commit personal projects, commercial
presets, or copyrighted audio.

Generated Note projects should initially use a known-valid minimal template and
replace verified fields while retaining required opaque metadata.

## Live extraction

The Live reader will:

1. stream-decompress the `.als` archive;
2. detect the Live format version;
3. parse tracks, scenes, clips, locators, devices, and sample references;
4. keep Session and Arrangement sources distinct;
5. normalize musical data into the canonical model;
6. report unknown or unsupported structures.

Version-specific XML differences belong behind Live format adapters, not in the
mapping or user-interface layers.

## Session mapping

Session content maps directly:

| Live | Note |
| --- | --- |
| MIDI track | Track |
| Scene | Scene |
| Clip slot | Clip slot |
| MIDI clip | MIDI clip |
| Clip loop | Clip loop |
| MIDI note | Note event |

When source content exceeds Note's limits, the user must select a subset, split the
output into multiple projects, split long clips, or cancel conversion.

## Arrangement mapping

Arrangement content maps through explicit time ranges. Each selected range becomes
one Note scene. Ranges may come from Live locators, manual bar selections, fixed
four/eight/sixteen-bar divisions, or selected Arrangement clips.

For each track and range, the mapper:

1. finds clips intersecting the range;
2. resolves timeline positions, start offsets, and loop braces;
3. expands audible loop repetitions;
4. crops events to the range;
5. rebases the result to beat zero;
6. merges the result into one Note clip;
7. preserves a loop only when it exactly represents the selected range.

Session and Arrangement data are never combined automatically.

## Destination mapping

The user maps source tracks and scenes or ranges into an explicit eight-by-eight
Note grid. A destination track has one instrument configuration. Assigning clips
from incompatible source tracks to the same destination track produces a visible
conflict rather than silently choosing an instrument.

## Device mapping

Mappings have three confidence levels:

- exact: verified Note-originated or equivalent state;
- translated: a verified subset of parameters;
- fallback: MIDI is preserved with a user-selected Note instrument.

Implementation order:

1. Drift using the structure accepted by Ableton's preset exporter.
2. Compatible Simpler instances to Melodic Sampler.
3. Known Note/Core Library Wavetable presets, followed by verified parameters.
4. Drum Rack and Drum Sampler with no more than sixteen relevant pads.
5. Supported effects and verified automation.

Unsupported plug-ins, Max for Live devices, and instruments must produce an explicit
error or require an explicit fallback policy.

## Samples and audio

The sample pipeline resolves project-relative paths, accepts additional user-provided
roots, reports missing files, deduplicates media by content hash, generates stable
bundle paths, and validates every reference before export.

The browser cannot follow arbitrary paths stored in an `.als`; the user must select
or drop the project directory and any external sample directories. A browser sample
resolver searches only those explicitly supplied files.

Audio clips are not a first-release target because Note has no conventional audio
tracks and does not support Live-style warping. Later constrained conversion may
load suitable audio into Melodic Sampler or Drum Sampler and generate trigger notes.

## Piano-roll preview

The piano roll reads the canonical clip model, never the Live XML directly. It has:

- a source view showing data extracted from Live;
- a target view showing the exact data that will be written to Note.

The initial preview displays pitches, positions, durations, grid divisions, velocity,
clip and loop boundaries, Arrangement range boundaries, muted notes, and conversion
warnings. Target notes that are cropped, repeated, split, altered, or unsupported
must be visually distinguishable.

The preview is initially read-only and rendered with an HTML canvas shared by the
browser and Tauri frontend.

## Application flow

1. Open or drop an `.als` file.
2. Add the Ableton project or sample folders when required.
3. Inspect tracks, scenes, devices, and missing media.
4. Choose Session or Arrangement content.
5. Select tracks and scenes or ranges.
6. Preview source and target clips.
7. Resolve device, media, and format-limit warnings.
8. Export `.ablbundle` and an optional compatibility report.

## Validation

Validation covers archive integrity, supported Live versions, XML extraction,
canonical-model invariants, Note limits, inferred JSON structure, media references,
ZIP structure, and every lossy conversion.

The final integration check for a generated bundle is:

1. open it in Note;
2. verify playback;
3. open the Note Set in Live;
4. compare notes, loops, mixer state, and devices with the expected fixture.

## Milestones

### 1. Format foundation

- Monolithic Rust library and developer inspector.
- Streaming `.als` GZIP/XML reading.
- Basic metadata, track, and clip discovery.
- Synthetic tests and CI.

Acceptance: inspect a real Live 11/12 Set without materializing its XML to disk.

### 2. Session clip extraction

- Parse Session tracks, scenes, MIDI clips, notes, velocity, and loops.
- Populate the canonical model.
- Report unsupported note expression and automation.

Acceptance: golden tests reproduce the musical content of paired Session fixtures.

### 3. Minimal Note writer

- Infer the minimal `.abl` schema.
- Write one track, scene, clip, note, and known Drift preset.
- Package a modern `.ablbundle`.

Acceptance: the generated bundle opens and plays in Note.

### 4. Browser inspection and piano roll

- WebAssembly adapter and local browser file loading.
- Source browser and read-only piano roll.
- Source/target preview and compatibility report.

### 5. Samples and devices

- Browser and native sample resolvers.
- Sample packaging and missing-media reporting.
- Drift, compatible Simpler, known Wavetable presets, and Drum Sampler.

### 6. Arrangement conversion

- Arrangement preview and explicit ranges.
- Locator-derived ranges, loop expansion, cropping, rebasing, and splitting.

### 7. Desktop application

- Tauri packaging and native filesystem resolver.
- Windows, macOS, and Linux builds using the shared frontend.

### 8. Extended fidelity

- Supported effects and automation.
- Additional Wavetable mappings and Drum Racks.
- Multi-project overflow export and optional piano-roll editing.

## Current next step

Complete Session MIDI clip and note extraction into the canonical model using minimal
synthetic fixtures from Live 11 and Live 12.
