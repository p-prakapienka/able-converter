# energyXT 3.0 project format (reader subset)

The energyXT reader understands the part of an energyXT 3.0 project (`.xt`) that
carries the music: the tempo, the sequencer's tracks, the part pool and the placed
parts with their MIDI notes and drum hits. The format was studied separately, in
the energyxt-format project, from the files the 3.0 installer ships with and the
behaviour of the program. That study documents the full format, including the mixer,
synth presets and embedded samples. This page lists only what the converter uses,
described in our own words.

## Record stream

A project is the string `"XTF"` (a `u32` length of 3, then the letters) followed
by a flat stream of records. Each record is a `u32` key length, an ASCII key of 1
to 32 letters or digits, a `u32` payload size and the payload. Integers are
little-endian, strings are a `u32` length and Latin-1 bytes. Unknown records are
skipped by their size.

Top-level records the reader uses:

| Key | Content |
|---|---|
| `bpm` | `f32` project tempo |
| `comp2` | a module: a header (`u8` version, `u8` class, `u8` role, two `i32` positions, `str` name, four `u32`), then the module's child records |

The first module of class 4 is the sequencer. Its child records are read in order;
container records among them (such as `arp`) are skipped whole.

## Sequencer records

```
track settings, one group per track:  track, trackCol, ...
the part pool:                        part [partCol events]  ...
per track, in track order:            ptrack, then that track's placed parts
```

| Key | Content used |
|---|---|
| `track` | `u32` kind (1 folder, 2 instrument, 3 audio, 5 drum), `u32` flags, `str` name, three `i32` display values, `i32` parent folder (-1 none), `u8`, `u8` MIDI channel |
| `trackCol` | `u32` track colour (an energyXT colour, not a Live palette index) |
| `ptrack` | marks the start of the next track's placed parts |
| `part` | `u8` version, `u8` kind (1 MIDI, 2 audio, 3 drum), `u8`, `i32` pool index (-1: own events), `i32` start, `i32` offset, `i32` length, `i32` flags, `str` name |
| `partCol` | `u32` part colour, `0xFFFFFFFF` for the default |
| `mevents` | `u32` count, then per event: `u32` delta ticks, `u8`, `u32` message (status, data 1, data 2 in its low bytes), and a `u32` length on note-ons |
| `devents` | `u32` count, then per hit: `u32` delta ticks, `u8` lane, `u8`, `u32` length, `u8` value count, then `u8` id and `f32` value per value |
| `achs`, `aevents2`, `aevents` | audio events: recorded, not mapped |
| `amevents` | automation lanes: recorded, not mapped |
| `tempo`, `timesig`, `marker`, `pattern` | timeline records the program can write: recorded, not mapped |

## Timing

- 960 ticks per quarter note.
- An event at time `t` in a part's event list sounds at the part's
  `start + offset + t`, and only if that is at or after `start` and before
  `start + length`. Events outside the part stay silent.
- A placed part with a pool index plays the pool part's events with its own start,
  offset and length. The pool part's own position does not matter. Nothing repeats.
- Flags bit 2 mutes a part; bit 0 is the editor selection.
- Drum hit value ids: 0 volume (default 1), 1 pan (default 0), 2 cutoff (default
  1). A value is stored only when it differs from its default.

## Mapping choices

- Instrument and drum tracks become MIDI tracks, audio tracks audio tracks, folders
  group tracks. Folder membership and track colours are not carried over.
- Each placed, unmuted MIDI or drum part becomes one Arrangement clip spanning the
  part, with the notes that sound inside it.
- Drum hits become notes on key 36 + lane, with velocity = volume × 127 (127 when
  the hit stores no volume).
- Controllers, pitch bend and other non-note messages, drum pan and cutoff values,
  automation, audio parts and timeline records are reported as diagnostics. The
  conversion assumes one tempo and 4/4.
