# Ableton Note Set format evidence

Ableton does not publish the Note/Move Set JSON schema as a supported public API.
The writer therefore implements only fields corroborated by independently generated
sets and open-source tools, and keeps unverified behavior explicit.

## Initial verified subset

- A modern `.ablbundle` is a ZIP archive with `Song.abl` at its root.
- `Song.abl` is UTF-8 JSON.
- Recent generated bundles identify the schema as
  `http://tech.ableton.com/schema/song/1.8.3/song.json`.
- The Set root contains tempo, time signature, scale/layout metadata, tracks, scenes,
  return tracks, the master track, grooves, and feature metadata.
- Session content is represented by one `clipSlots` array per MIDI track. A populated
  slot contains a clip region, loop, notes, and envelopes.
- MIDI notes use `noteNumber`, `startTime`, `duration`, `velocity`, and `offVelocity`.
- `ableton:/packs/abl-core-library/Track%20Presets/Templates/Analog%20Drift.json` is a
  known Core Library preset reference used by existing Note/Move sets.
- Stored ZIP entries are accepted by existing bundle readers and avoid adding a
  compression dependency to browser builds.

The initial implementation is based on structural evidence from:

- [MidiToMove](https://github.com/OnjLouis/MidiToMove), an MIT-licensed writer that
  creates current `.ablbundle` files;
- [Extending Move](https://github.com/charlesvestal/extending-move), an MIT-licensed
  collection containing a generated Set schema and synthetic Set examples.

No source code, commercial preset, project, or sample from either repository is
vendored into Able Converter.

## Deliberately provisional behavior

The first mapper emits a typed structural subset and references the Analog Drift
preset rather than embedding opaque preset state. Physical validation must establish
whether Note hydrates that reference by itself or requires an expanded device chain.

The canonical model does not yet contain a global key, scale, or time signature, so
the writer currently emits C major and 4/4. Scene tempo overrides, non-zero Live loop
start-relative values, probability, and muted notes produce diagnostics instead of
being discarded silently.

Live color identifiers are currently carried through unchanged. Their equivalence to
Note colors also remains a paired-fixture question.

## Physical acceptance check

1. Export a one-track, one-scene, one-clip, one-note bundle.
2. Open it in Ableton Note and confirm that Analog Drift loads and the note plays.
3. Export or transfer that Set to Live.
4. Compare tempo, track/scene names, clip and loop bounds, pitch, timing, velocities,
   color, and device state with the expected canonical project.
5. Save the smallest synthetic paired fixtures and record every changed field before
   expanding the schema.
