# Caustic song format evidence

Single Cell Software never published a specification for the Caustic 3 song
(`.caustic`) format, and the application is no longer maintained. The parser
therefore implements only the container structure that is corroborated by direct
evidence, and preserves every byte it cannot explain.

## Evidence sources

- **The shipped engine binary.** `libcaustic.so` (ARMv7, Android build) is stripped
  of `.symtab` but retains roughly 7,200 exported C++ symbols in `.dynsym`. The
  serialisation design is visible directly in those signatures: a single `Archive`
  class exposing `Archive::Serialize(void*, unsigned int)`, and one
  `Serialize(Archive&)` method per persisted class — `OutputPanel`, `MachineRack`,
  `RackMachine`, `Sequencer`, `Mixer`, `MasterMixer`, `EffectsRack`, `Scale`,
  `MIDIMachineMapping`, `SynthPatternEditor`, `UIControlCollection`, and one per
  machine and effect type.
- **Chunk identifiers.** `RACK`, `OUTP`, `EFFX`, `MIXR`, `MSTR`, `SEQN`, `CCOL`,
  `SPAT`, `MCOM`, and `NULL` all appear as literals in the binary's `.rodata`.
  `UIControlCollection::Serialize` reads a four-byte tag and compares it
  byte by byte against `CCOL`, and reads each entry's tag against `CTRL` the
  same way.
- **Machine identifiers.** `SSYN`, `PCMS`, `BBOX`, `PADS`, `8SYN`, `MDLR`, `ORGN`,
  `VCDR`, `FMSN`, `KSSN`, `SAWS`, and `BLNE` appear as literals and correspond to
  the machine classes that implement `Serialize(Archive&)`.
- **[DawVert](https://github.com/SatyrDiamond/DawVert)**, a GPL-3.0 project file
  converter whose `objects/file_proj/proj_caustic.py` reads the same container.
  It was used only to corroborate the container layout described below. No code was
  copied, and the layout facts it confirms are not themselves copyrightable.

## Implemented container structure

- A song is a sequence of chunks. Each chunk is a four-byte ASCII tag followed by a
  little-endian `u32` payload length. All multi-byte values are little-endian.
- The first chunk is `RACK` and holds the whole song. Its payload opens with a fixed
  264-byte block that is not decoded and is preserved verbatim.
- Inside the rack, sections follow in file order: `OUTP`, `EFFX`, `MIXR`, `MSTR`,
  `SEQN`. Each is a tag, a `u32` length, and that many payload bytes.
- The machine slot table follows the `OUTP` payload rather than being contained by
  it. It holds 14 entries of a four-byte machine identifier plus one padding byte.
  An unoccupied slot is written as `NULL`.
- Each occupied slot is then written in slot order as a 10-byte name, four
  header bytes, a `u32` payload length, and that many payload bytes. The four
  bytes and the length sit where each machine's `Serialize(Archive&)` writes its
  own block tag and block length (`SubSynth::Serialize` writes `SSYN` and checks
  for it on load); the bytes are kept as written.
- Most machine payloads open with a `u32` block version before their control
  collection. `SubSynth::Serialize` writes `0x03010000` and branches on it when
  loading; `PadSynth::Serialize` and `Organ::Serialize` call
  `UIControlCollection::Serialize` straight after the length. DawVert reads a
  leading version word (as `u16`, `u8`, `u8`) for `SSYN`, `BLNE`, `FMSN`,
  `KSSN`, `SAWS`, `8SYN`, `BBOX`, `VCDR` and `PCMS`, and none for `PADS` and
  `ORGN`. The parser takes the first word as the block version only when a
  `CCOL` tag follows it.
- A machine payload usually opens with a `CCOL` chunk: a tag and a `u32` length,
  then one `CTRL` chunk per control. Each `CTRL` chunk is the tag, a `u32` length
  of 8, a `u32` control identifier and an `f32` value, so 16 bytes per control.
  `UIControlCollection::Serialize` writes exactly that for every saved control,
  patching both lengths afterwards with `ftell`/`fseek`, and on load reads the
  tag, the length, the identifier and the value in that order. DawVert also
  iterates `CCOL` as nested chunks. The engine writes no other entry kind and
  its reader does not skip one, so the parser rejects any entry that is not a
  `CTRL` chunk of length 8. Control names are not stored; the engine resolves
  them through per-machine tables (`GetControlIDFromName` is implemented by 57
  classes).

## Deliberately provisional behavior

- **Transport values.** Tempo is read as an `f32` at offset 82 of the `OUTP`
  payload and the bar length as the `u8` that follows. These offsets come from
  DawVert and have not yet been confirmed against `OutputPanel::Serialize`. When
  the payload is shorter than that, both values are left unset rather than guessed.
- **Undecoded words between sections.** A section is sometimes followed by one word
  that belongs to no decoded field. The parser steps over at most one such word,
  and only when a recognised tag follows it, keeping the bytes on the preceding
  section. Anything else ends section parsing and is preserved as trailing bytes.
- **Machine bodies.** Everything after a machine's control collection is kept as
  raw bytes. The block version is read but not interpreted; older block
  versions change the layout that follows (SubSynth has fields that appear at
  1.1, 2.0, 3.0 and 3.1), which is why the body stays raw. Per-machine field layouts are recoverable from each class's
  `Serialize(Archive&)` implementation, which writes fields in source order through
  `Archive::Serialize(&field, size)`, but none of them are decoded yet.
- **`SPAT` and `MCOM`.** Pattern and modular chunks are present in the binary and
  appear inside machine bodies. They are not parsed.

## Version scope

Only the current song layout is implemented. The engine binary shows that older
layouts exist and are still readable by the application: `Sequencer`, `BassLine`,
and `PCMSynth` each carry a `SerializeLegacy(Archive&)` alongside their current
`Serialize(Archive&)`, so at least those three branch on a stored version.

No file-format version field has been located. The binary exposes `g_nAppVersion`
and `GetCoreVersion`, but neither is a song-format version, and no version-named
symbol or string corresponds to one. If the format stores a version, the most
likely place is the undecoded 264-byte block at the start of the rack payload.

Until that field is identified, the parser cannot detect a legacy song up front.
It will read one as far as the layouts agree and then fail on a length or tag
check, rather than reporting an unsupported version. Legacy support is out of
scope for now.

## Verification status

The parser has been exercised only against synthetic fixtures built byte by byte in
`src/parser/CausticParserTest.rs`. It has **not** yet been run against a song saved
by Caustic itself. Real `.caustic` files are not committed to this repository.
