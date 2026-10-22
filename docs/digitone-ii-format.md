# Digitone II project format

This crate reads and writes `.dn2prj` projects. The layout below is the part
implemented here. It follows public research rather than a vendor specification:

- [NoiseAndMatter/DNX](https://github.com/NoiseAndMatter/DNX) `docs/dn2-format.md`,
  `docs/dn2-pattern-format.md`, and the container correction in
  `packages/core/src/project/container.ts` (2026-10-05). That correction
  supersedes the earlier little-endian reading of the container header.
- [xrcstrecords/syxgrid-digitone-ii](https://github.com/xrcstrecords/syxgrid-digitone-ii),
  the pattern editor listed on elektronmods, whose SysEx map describes the same
  pattern record a project stores.

DNX and SYXGRID are AGPL-3.0. This implementation is an independent reader and
writer of the published field layout. It does not vendor either codebase.
Elektron firmware, presets, and user projects are not included.

Offsets that those notes mark speculative are not decoded. They stay inside the
preserved image and are written back unchanged.

## Archive

A `.dn2prj` file is a ZIP with `manifest.json` and one binary entry named by
`manifest.Payload`. Either stored or deflated entries are accepted. A raw
container, as a +Drive read returns, is also accepted, but it has no manifest
and cannot be packed as an archive until one is supplied.

## Container

| Offset | Contents |
| --- | --- |
| `0x00` | `AC 11 D3 03` |
| `0x04` | `02 00 05 00` |
| `0x08` | device `15` for Digitone II |
| `0x09` | four ASCII format characters, `"0050"` or `"0059"` |
| `0x0D` | u32be content kind, `1` for a project |
| `0x11` | u32be root object version |
| `0x15` | u32be index; byte `0x17` is the bank and byte `0x18` is the slot |
| `0x19` | u32be uncompressed image length |
| `0x1D` | `1` when the body is an LZ4 chain, `0` when it is raw |
| `0x1E` | `12`, the trailer length |
| `0x1F` | body |
| end-12 | u32be check field |
| end-8 | u32be body length |
| end-4 | `AA A1 DA AA` |

The check field is reflected CRC-32 over the body, polynomial `0x04C11DB7`,
initial register `0`, result complemented. It is not ISO-HDLC CRC-32.

The body is a chain of LZ4 blocks. Each block is a u32be size, then that many
bytes. A zero size ends the chain. Blocks are linked: a match may refer to an
earlier block. The exporter writes a new valid chain, not Elektron's bytes.
Compare decoded images, not compressed payloads.

## Image

| Firmware | Format | Decoded size | Project object version |
| --- | --- | --- | --- |
| OS 1.10E | `"0050"` | 12,889,604 | 2 or 3 |
| OS 1.11 | `"0059"` | 12,890,116 | 5 |

Pattern and kit offsets are the same for both sizes. OS 1.11 inserts 512 bytes
further along the tail, in the song array, which this crate does not decode.

| Region | Offset | Count | Stride |
| --- | --- | --- | --- |
| Header | `0x000000` | 1 | 512 |
| Patterns | `0x000200` | 128 | 89,088 |
| Kits | `0xAE0200` | 128 | 10,752 |
| Tail | `0xC30200` | | remainder |

The header starts with `BE EF BA CE`, a u32be version, a 16-byte NUL-padded
name, and a u32be identity token at `0x18`. The token is preserved. It is not a
content hash and this crate does not mint a new one.

Pattern `k` is paired with kit `k`. Pattern index `0` is `A1` and index `97` is
`G2`.

## What is decoded

Version-3 pattern records:

- 16 track records of 1,187 bytes from offset 4.
- Step flag words, one u16be per step. Bit `0x0001` is a trig. Masking off
  `0x0010`, `0x0200`, and the top nibble leaves `0x0181` for a note trig and
  `0x0801` for a lock trig. Other shapes stay unrecognised.
- Trig condition at track `+0x100`, FILL at `+0x180`, probability percent at
  `+0x200`, sound-lock index at `+0x400`. `0xFF` means unset. Condition names
  are returned only for codes the published captures measured.
- 8,192 trigger slots of 6 bytes at `0x4A34`: track, step, note, velocity,
  length, signed micro timing. A slot whose track byte is `0xFF` is unused.
  `0xFF` in the micro-timing byte is −1, not "unset". `0xFF` in velocity or
  length inherits the track default. The reader scans the whole pool.

Kit records, when they start with `BE EF BA CE`:

- 16-byte name.
- 16 track levels as u16le. Observed values are 0..=127.
- 16 sound slots of 359 bytes. The name is at `+0x0C`. The machine byte at
  `+244` is FM TONE, WAVETONE, FM DRUM, SWARMER, MIDI, or unrecognised.
- MIDI-track bitmask as u16be at kit `+10260`, and only when the project object
  version is 3. Bit *t* set means track *t* is MIDI.

Note length uses the measured band table. The model exposes it as sixteenths of
a step, with byte 127 meaning infinite and byte 255 meaning inherit. Bytes
128..=254 are not in that table and are reported as unrecognised.

## What is written

Names, track levels, the payload of an existing trigger slot, and a new note
trig. A written payload is the note, velocity, length byte, and micro timing.
A new note trig uses the importer flag word `0x0181` plus the odd-step
bit `0x0010`. Device-authored trigs often set extra high bits; those are not
invented here. Everything else in the image is copied.

There is no canonical-model mapper on this branch. Live and Note conversion is
unchanged.
