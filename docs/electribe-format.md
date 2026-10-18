# Korg electribe sampler pattern format evidence

Korg does not publish the electribe sampler pattern file format. The parser reads only
fields established from the electribe sampler 2.02 system firmware (ARM code loaded at
`0xC0000000`), cross-checked against a real 250-pattern export and an independent
third-party pattern editor. Addresses below refer to that firmware image.

The source model keeps native values. Fields whose meaning is not established are kept
under neutral names (`unknown0A`, `unknown26`, `unknown40`, `unknownFlag`) and are never
given invented semantics.

## Files

All integers are little-endian.

| File | Layout | Size |
| --- | --- | --- |
| `.e2spat` | header `0x100` + pattern `0x4000` | `0x4100` |
| `.e2sallpat` | header `0x100` + global block `0x10000` + 250 patterns × `0x4000` | `0x3F8100` |

The common header (written at `C0037090` and `C002DFBC`) holds `KORG` at `0x00` and the
product `e2sampler` at `0x10`, each NUL-padded to 16 bytes, then u32 `1` at `0x20` and
`0xFF` fill to `0xFF`. Only the `e2sampler` product is accepted; the electribe synth uses
a different product string and is not verified.

The global block starts with `GLST` and has `GLED` at `+0xFC` (checked at `C004718C`). The
remainder is `0xFF` fill in real exports.

## Pattern (`0x4000` bytes)

A pattern is valid when it starts with `PTST` and holds `PTED` at `0x3BFC` (checked at
`C004A310`).

| Offset | Type | Field |
| --- | --- | --- |
| `0x10` | 18 bytes | name, NUL- or space-padded |
| `0x22` | u16 | tempo × 10 (20.0–300.0 BPM) |
| `0x24` | i8 | swing, −48…+48 |
| `0x25` | u8 | length in bars − 1 |
| `0x26` | u8 | beat: 16, 32, 8Tri, 16Tri |
| `0x27` / `0x28` / `0x29` | u8 | key, scale, chord set |
| `0x2A` | u8 | pattern level, stored as `127 − level` |
| `0x31` | u8 | gate arp |
| `0x3C`–`0x41` | u8 | master effect on, type, pad X, pad Y, unknown, hold |
| `0x44` / `0x45` | u8 | alternate 13-14 / 15-16 |
| `0x100` | u8[24] | motion slot part (1–16, 17 = pattern level, 0 = empty) |
| `0x118` | u8[24] | motion slot parameter id (motion-specific numbering) |
| `0x130` | u8[24][64] | motion values |
| `0x800` | part[16] | parts, stride `0x330` |
| `0x3B00` / `0x3B02` | u16 / u8 | chain to, chain repeat |

### Part (`0x330` bytes)

The first `0x30` bytes are the part settings; 64 steps of 12 bytes follow at `0x30`.

| Offset | Field | Offset | Field |
| --- | --- | --- | --- |
| `0x00` | last step (0 = default) | `0x10`–`0x12` | modulation type, speed, depth |
| `0x01` | mute | `0x14` / `0x15` | attack, decay/release |
| `0x02` | voice assign | `0x18` / `0x19` | level, pan (signed) |
| `0x03` | motion sequence | `0x1A` / `0x1B` | amp EG, MFX send |
| `0x04` | trigger-pad velocity | `0x1C` / `0x1D` | groove type, depth |
| `0x05` | scale mode | `0x20`–`0x22` | IFX on, type, edit |
| `0x06` | part priority | `0x24` / `0x25` | pitch (signed), glide |
| `0x08` | u16 oscillator (sample) | `0x0B`–`0x0F` | oscillator edit, filter type, cutoff, resonance, EG intensity (signed) |

Part bytes `0x03`–`0x06` were confirmed from the firmware's own display routines
(`C0080C54` reads motion sequence at `0x03`, `C0080BFC` prints Normal/High from `0x06`). The
third-party editor assigns these four bytes in a different order; real data supports the
firmware order (`0x03` is 1, the "Smooth" default, in 3,994 of 4,000 parts).

### Step (12 bytes)

| Offset | Field |
| --- | --- |
| 0 | triggered |
| 1 | gate: 0–96; any value above 96 is a tie (the UI shows TIE above `0x60` at `C008D124`) |
| 2 | velocity, 1–127 |
| 3 | unknown flag; not a chord flag in real data |
| 4–11 | note slots: MIDI note + 1, 0 = empty; positional |

## Musical interpretation

These rules are implemented by `ElectribeToInternalMapper` and its `StepTimeline`.

- Every bar occupies sixteen step slots. A bar plays its first N slots, where N is the
  part's last step, or 16 for straight beats and 12 for triplet beats when last step is 0.
  This matches the firmware's own clip-length calculation (`C002ED08`) and real data.
- Steps per quarter note: 16 → 4, 32 → 8, 8Tri → 3, 16Tri → 6 (firmware tables at
  `C00A7708`/`C00A770B`).
- A note's length is `gate / 96` of a step (provisional: 96 is assumed to be a full step).
- A tied step holds for the full step and continues into the next played step that holds
  the same note slot value. This mirrors the firmware's tie walk at `C008E4BC`. Ties do not
  wrap around the end of the loop.
- An untriggered step that still holds notes maps to muted notes.
- A part whose last step differs from the default becomes a shorter looping clip. Its phase
  relative to a device pattern restart is not verified, so it is reported.

## Canonical mapping

| electribe sampler | Canonical model |
| --- | --- |
| Part 1–16 | MIDI track `part:N` |
| Pattern | Session scene `pattern:N` (index N − 1) |
| Part with note data in a pattern | looping Session clip `pattern:N:part:M` |
| Step note | note with start, duration and velocity |
| Pattern tempo | project tempo (first mapped pattern), otherwise a scene tempo override |
| Part mute | disabled clip (reported) |

Reported and not mapped: swing, part groove, key and scale, pattern level and master
effect, chains, alternate 13-14/15-16 choke, motion sequences, notes in steps outside the
played length, and all part sound settings. The sample pool (`e2sSample.all`) and event
recordings (`.e2sev`) are not parsed yet.

## Validation

Synthetic tests build pattern bytes in memory; no Korg content is committed. A conversion
of four real patterns (eight parts each) to a Note bundle was checked against an
independent decoder: all 32 clips matched note start and pitch exactly.
