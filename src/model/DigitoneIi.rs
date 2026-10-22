//! Digitone II project model.
//!
//! The fields here are the ones established by public project-format research
//! (see `docs/digitone-ii-format.md`). Bytes that research does not explain stay
//! in the preserved image and are written back unchanged.

use thiserror::Error;

/// Decoded size of a Digitone II project written by OS 1.10E, format `"0050"`.
pub const IMAGE_SIZE_OS110: usize = 12_889_604;

/// Decoded size of a Digitone II project written by OS 1.11, format `"0059"`.
pub const IMAGE_SIZE_OS111: usize = 12_890_116;

/// Bytes before the pattern array. Root object and project name live here.
pub const HEADER_SIZE: usize = 0x200;

pub const PATTERN_COUNT: usize = 128;
pub const PATTERN_SIZE: usize = 89_088;
pub const KIT_BASE: usize = 0x00AE_0200;
pub const KIT_SIZE: usize = 10_752;
pub const TAIL_BASE: usize = 0x00C3_0200;

pub const TRACK_COUNT: usize = 16;
pub const STEP_COUNT: usize = 128;
pub const TRACK_SIZE: usize = 1_187;
pub const TRIG_SLOT_COUNT: usize = 8_192;
pub const TRIG_SLOT_SIZE: usize = 6;

pub const NAME_FIELD_LEN: usize = 16;
pub const SOUND_SIZE: usize = 359;
pub const MIDI_TRACK_SIZE: usize = 268;

pub const CONTAINER_HEADER_LEN: usize = 0x1F;
pub const CONTAINER_TRAILER_LEN: usize = 12;
pub const DEVICE_DIGITONE_II: u8 = 15;
pub const CONTENT_KIND_PROJECT: u32 = 1;

pub const CONTAINER_MAGIC: [u8; 4] = [0xAC, 0x11, 0xD3, 0x03];
pub const FOOTER_MAGIC: [u8; 4] = [0xAA, 0xA1, 0xDA, 0xAA];
pub const OBJECT_MAGIC: [u8; 4] = [0xBE, 0xEF, 0xBA, 0xCE];
pub const CONTAINER_PAIR: [u8; 4] = [0x02, 0x00, 0x05, 0x00];

const PATTERN_VERSION_OFFSET: usize = 0;
const TRACK_ARRAY_OFFSET: usize = 4;
const TRIG_ARRAY_OFFSET: usize = 0x4A34;
const STEP_FLAGS_OFFSET: usize = 0;
const CONDITION_OFFSET: usize = 0x100;
const FILL_OFFSET: usize = 0x180;
const PROBABILITY_OFFSET: usize = 0x200;
const SOUND_LOCK_OFFSET: usize = 0x400;
const KIT_NAME_OFFSET: usize = 8;
const LEVEL_OFFSET: usize = 0x1C;
const SOUND_ARRAY_OFFSET: usize = 60;
const SOUND_NAME_OFFSET: usize = 0x0C;
const MACHINE_OFFSET: usize = 244;
const MIDI_MASK_OFFSET: usize = 10_260;
const PROJECT_NAME_OFFSET: usize = 8;
const IDENTITY_OFFSET: usize = 0x18;
const NOTE_TRIG_SHAPE: u16 = 0x0181;
const LOCK_TRIG_SHAPE: u16 = 0x0801;
const TRIG_SHAPE_CLEAR: u16 = 0xF210;
const UNUSED_SLOT: u8 = 0xFF;
const INHERIT: u8 = 0xFF;

/// Why a verified field could not be written.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DigitoneIiEditError {
    #[error("pattern {pattern} is outside 0..{PATTERN_COUNT}")]
    PatternOutOfRange { pattern: usize },
    #[error("track {track} is outside 0..{TRACK_COUNT}")]
    TrackOutOfRange { track: usize },
    #[error("step {step} is outside 0..{STEP_COUNT}")]
    StepOutOfRange { step: usize },
    #[error("trigger slot {slot} is outside 0..{TRIG_SLOT_COUNT}")]
    SlotOutOfRange { slot: usize },
    #[error(
        "pattern {pattern} record version is {version}; this edit is only defined for version 3"
    )]
    UnrecognisedPatternVersion { pattern: usize, version: u32 },
    #[error("kit {pattern} has no object header, so its name and levels are not edited")]
    MissingKit { pattern: usize },
    #[error("sound on pattern {pattern} track {track} has no object header")]
    MissingSound { pattern: usize, track: usize },
    #[error("trigger slot {slot} on pattern {pattern} is unused")]
    UnusedSlot { pattern: usize, slot: usize },
    #[error("pattern {pattern} already has a trigger on track {track} step {step}")]
    TrigAlreadyExists {
        pattern: usize,
        track: usize,
        step: usize,
    },
    #[error("pattern {pattern} has no free trigger slot")]
    TrigPoolFull { pattern: usize },
    #[error("note {note} is outside 0..=127")]
    NoteOutOfRange { note: u8 },
    #[error("velocity {velocity} must be 0..=127 or 255 to inherit the track default")]
    VelocityOutOfRange { velocity: u8 },
    #[error("note length {noteLength} must be 0..=127 or 255 to inherit the track default")]
    LengthOutOfRange { noteLength: u8 },
    #[error("micro timing {microTiming} is outside the verified -23..=23 range")]
    MicroTimingOutOfRange { microTiming: i8 },
    #[error("track level {level} is outside the verified 0..=127 range")]
    LevelOutOfRange { level: u16 },
}

/// Manifest stored beside the binary payload in a `.dn2prj` archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigitoneIiManifest {
    pub formatVersion: String,
    pub productType: Vec<String>,
    pub payloadName: String,
    pub fileType: String,
    pub firmwareVersion: String,
    raw: Vec<u8>,
}

impl DigitoneIiManifest {
    pub(crate) fn new(
        formatVersion: String,
        productType: Vec<String>,
        payloadName: String,
        fileType: String,
        firmwareVersion: String,
        raw: Vec<u8>,
    ) -> Self {
        Self {
            formatVersion,
            productType,
            payloadName,
            fileType,
            firmwareVersion,
            raw,
        }
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.raw
    }
}

/// Fields read from the 31-byte container header. Unknown header bytes stay in `bytes`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigitoneIiContainer {
    pub device: u8,
    pub formatVersion: String,
    pub contentKind: u32,
    pub objectVersion: u32,
    pub bank: u8,
    pub slot: u8,
    pub declaredImageLength: u32,
    pub compressed: bool,
    pub bytes: [u8; CONTAINER_HEADER_LEN],
}

/// How a step flag word reads after masking the bits that are not part of trig kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigitoneIiTrigKind {
    Note,
    Lock,
    Unrecognised,
}

/// One occupied step inside a version-3 pattern track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DigitoneIiStep {
    pub track: u8,
    pub step: u8,
    pub flags: u16,
    pub kind: DigitoneIiTrigKind,
    /// Primary trig-condition code. `None` when the stored byte is `0xFF`.
    pub condition: Option<u8>,
    /// FILL-family code. `None` when the stored byte is `0xFF`.
    pub fill: Option<u8>,
    /// Probability percent. `None` when the stored byte is `0xFF`.
    pub probability: Option<u8>,
    /// Sound-pool index. `None` when the stored byte is `0xFF`.
    pub soundLock: Option<u8>,
}

/// One live entry in the pattern's shared trigger pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DigitoneIiTrig {
    pub slot: usize,
    pub track: u8,
    pub step: u8,
    pub note: u8,
    pub velocity: u8,
    /// Raw length byte. `0xFF` inherits the track default. See [`noteLengthSixteenths`].
    pub noteLength: u8,
    pub microTiming: i8,
}

/// Note, velocity, length, and micro timing written into a trigger slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DigitoneIiTrigPayload {
    pub note: u8,
    pub velocity: u8,
    pub noteLength: u8,
    pub microTiming: i8,
}

/// Note length stored in a trigger slot, in sixteenths of a step.
///
/// Bytes `128..=254` are not in the measured table and stay [`Unrecognised`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigitoneIiNoteLength {
    Inherit,
    Infinite,
    Sixteenths(u32),
    Unrecognised(u8),
}

/// Machine byte at sound-slot offset 244. Values above 4 are not named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigitoneIiMachine {
    FmTone,
    Wavetone,
    FmDrum,
    Swarmer,
    Midi,
    Unrecognised(u8),
}

impl DigitoneIiMachine {
    #[must_use]
    pub fn fromByte(byte: u8) -> Self {
        match byte {
            0 => Self::FmTone,
            1 => Self::Wavetone,
            2 => Self::FmDrum,
            3 => Self::Swarmer,
            4 => Self::Midi,
            other => Self::Unrecognised(other),
        }
    }

    #[must_use]
    pub fn asByte(self) -> u8 {
        match self {
            Self::FmTone => 0,
            Self::Wavetone => 1,
            Self::FmDrum => 2,
            Self::Swarmer => 3,
            Self::Midi => 4,
            Self::Unrecognised(byte) => byte,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigitoneIiSound {
    pub track: usize,
    pub present: bool,
    pub version: Option<u32>,
    pub name: String,
    pub machine: Option<DigitoneIiMachine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigitoneIiKit {
    pub index: usize,
    pub present: bool,
    pub version: Option<u32>,
    pub name: String,
    pub trackLevels: Vec<u16>,
    pub sounds: Vec<DigitoneIiSound>,
    /// Synth/MIDI bitmask. Bit *t* set means track *t* is MIDI.
    ///
    /// Only read when the project object version is 3. On version 2 the same
    /// offset is not this mask, and version 5 has not been confirmed.
    pub midiTrackMask: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigitoneIiPattern {
    pub index: usize,
    pub label: String,
    pub version: u32,
    /// `false` when the record version is not 3. Trigs are then left unread.
    pub recognised: bool,
    pub steps: Vec<DigitoneIiStep>,
    pub trigs: Vec<DigitoneIiTrig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigitoneIiInspection {
    pub payloadName: String,
    pub firmwareVersion: String,
    pub formatVersion: String,
    pub containerObjectVersion: u32,
    pub projectObjectVersion: Option<u32>,
    pub projectName: String,
    pub identityToken: u32,
    pub bank: u8,
    pub slot: u8,
    pub imageLength: usize,
    pub sourceCompressed: bool,
    pub patternsWithTrigs: usize,
    pub trigCount: usize,
    pub namedKitCount: usize,
}

/// A parsed Digitone II project. The image is the source of truth for export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigitoneIiProject {
    pub manifest: DigitoneIiManifest,
    header: [u8; CONTAINER_HEADER_LEN],
    image: Vec<u8>,
}

impl DigitoneIiProject {
    pub(crate) fn fromParts(
        manifest: DigitoneIiManifest,
        header: [u8; CONTAINER_HEADER_LEN],
        image: Vec<u8>,
    ) -> Self {
        Self {
            manifest,
            header,
            image,
        }
    }

    #[must_use]
    pub fn header(&self) -> &[u8; CONTAINER_HEADER_LEN] {
        &self.header
    }

    #[must_use]
    pub fn image(&self) -> &[u8] {
        &self.image
    }

    #[must_use]
    pub fn container(&self) -> DigitoneIiContainer {
        let formatVersion = self.header[0x09..0x0D]
            .iter()
            .map(|byte| char::from(*byte))
            .collect();
        DigitoneIiContainer {
            device: self.header[0x08],
            formatVersion,
            contentKind: readBe32(&self.header, 0x0D),
            objectVersion: readBe32(&self.header, 0x11),
            bank: self.header[0x17],
            slot: self.header[0x18],
            declaredImageLength: readBe32(&self.header, 0x19),
            compressed: self.header[0x1D] != 0,
            bytes: self.header,
        }
    }

    #[must_use]
    pub fn projectName(&self) -> String {
        readName(
            self.image
                .get(PROJECT_NAME_OFFSET..PROJECT_NAME_OFFSET + NAME_FIELD_LEN)
                .unwrap_or(&[]),
        )
    }

    pub fn setProjectName(&mut self, name: &str) {
        if let Some(field) = self
            .image
            .get_mut(PROJECT_NAME_OFFSET..PROJECT_NAME_OFFSET + NAME_FIELD_LEN)
        {
            writeName(field, name);
        }
    }

    /// Four bytes at image offset `0x18`. Preserved, not recomputed.
    ///
    /// Public research treats this as an identity or revision token rather than
    /// a checksum. This crate does not invent a new value.
    #[must_use]
    pub fn identityToken(&self) -> u32 {
        readBe32(&self.image, IDENTITY_OFFSET)
    }

    #[must_use]
    pub fn projectObjectVersion(&self) -> Option<u32> {
        if self.image.len() < 8 || self.image[0..4] != OBJECT_MAGIC {
            return None;
        }
        Some(readBe32(&self.image, 4))
    }

    pub fn pattern(&self, index: usize) -> Result<DigitoneIiPattern, DigitoneIiEditError> {
        self.requirePattern(index)?;
        let base = patternOffset(index);
        let version = readBe32(&self.image, base + PATTERN_VERSION_OFFSET);
        let recognised = version == 3;
        let (steps, trigs) = if recognised {
            (self.readSteps(base), self.readTrigs(base))
        } else {
            (Vec::new(), Vec::new())
        };
        Ok(DigitoneIiPattern {
            index,
            label: patternLabel(index),
            version,
            recognised,
            steps,
            trigs,
        })
    }

    pub fn kit(&self, index: usize) -> Result<DigitoneIiKit, DigitoneIiEditError> {
        self.requirePattern(index)?;
        let base = kitOffset(index);
        let present = self.hasMagic(base);
        let version = present.then(|| readBe32(&self.image, base + 4));
        let name = if present {
            readName(&self.image[base + KIT_NAME_OFFSET..base + KIT_NAME_OFFSET + NAME_FIELD_LEN])
        } else {
            String::new()
        };
        let trackLevels = if present {
            (0..TRACK_COUNT)
                .map(|track| readLe16(&self.image, base + LEVEL_OFFSET + track * 2))
                .collect()
        } else {
            Vec::new()
        };
        let sounds = (0..TRACK_COUNT)
            .map(|track| self.readSound(base, track))
            .collect();
        let midiTrackMask = if present && self.projectObjectVersion() == Some(3) {
            Some(readBe16(&self.image, base + MIDI_MASK_OFFSET))
        } else {
            None
        };
        Ok(DigitoneIiKit {
            index,
            present,
            version,
            name,
            trackLevels,
            sounds,
            midiTrackMask,
        })
    }

    pub fn setKitName(&mut self, pattern: usize, name: &str) -> Result<(), DigitoneIiEditError> {
        self.requirePattern(pattern)?;
        let base = kitOffset(pattern);
        if !self.hasMagic(base) {
            return Err(DigitoneIiEditError::MissingKit { pattern });
        }
        writeName(
            &mut self.image[base + KIT_NAME_OFFSET..base + KIT_NAME_OFFSET + NAME_FIELD_LEN],
            name,
        );
        Ok(())
    }

    pub fn setSoundName(
        &mut self,
        pattern: usize,
        track: usize,
        name: &str,
    ) -> Result<(), DigitoneIiEditError> {
        self.requireTrack(pattern, track)?;
        let base = self.soundOffset(pattern, track);
        if !self.hasMagic(base) {
            return Err(DigitoneIiEditError::MissingSound { pattern, track });
        }
        writeName(
            &mut self.image[base + SOUND_NAME_OFFSET..base + SOUND_NAME_OFFSET + NAME_FIELD_LEN],
            name,
        );
        Ok(())
    }

    pub fn setTrackLevel(
        &mut self,
        pattern: usize,
        track: usize,
        level: u16,
    ) -> Result<(), DigitoneIiEditError> {
        self.requireTrack(pattern, track)?;
        if level > 127 {
            return Err(DigitoneIiEditError::LevelOutOfRange { level });
        }
        let base = kitOffset(pattern);
        if !self.hasMagic(base) {
            return Err(DigitoneIiEditError::MissingKit { pattern });
        }
        let at = base + LEVEL_OFFSET + track * 2;
        self.image[at] = u8::try_from(level).expect("level was checked against 127");
        self.image[at + 1] = 0;
        Ok(())
    }

    /// Replace the note payload of an existing trigger slot. Track and step stay put.
    pub fn setTrigPayload(
        &mut self,
        pattern: usize,
        slot: usize,
        payload: DigitoneIiTrigPayload,
    ) -> Result<(), DigitoneIiEditError> {
        self.requireVersion3(pattern)?;
        self.requirePayload(payload)?;
        if slot >= TRIG_SLOT_COUNT {
            return Err(DigitoneIiEditError::SlotOutOfRange { slot });
        }
        let at = patternOffset(pattern) + TRIG_ARRAY_OFFSET + slot * TRIG_SLOT_SIZE;
        if self.image[at] == UNUSED_SLOT {
            return Err(DigitoneIiEditError::UnusedSlot { pattern, slot });
        }
        self.writePayload(at, payload);
        Ok(())
    }

    /// Append a note trig using the importer flag word `0x0181` plus step parity.
    ///
    /// That word is what Elektron's own conversions store for a note trig. A trig
    /// made on the instrument often sets extra high bits; those bits are not
    /// written here. The slot is the first `0xFF`-headed hole, which is how the
    /// device frees a slot.
    pub fn insertNoteTrig(
        &mut self,
        pattern: usize,
        track: usize,
        step: usize,
        payload: DigitoneIiTrigPayload,
    ) -> Result<usize, DigitoneIiEditError> {
        self.requireVersion3(pattern)?;
        self.requireTrack(pattern, track)?;
        if step >= STEP_COUNT {
            return Err(DigitoneIiEditError::StepOutOfRange { step });
        }
        self.requirePayload(payload)?;
        let base = patternOffset(pattern);
        if self.slotFor(base, track, step).is_some() {
            return Err(DigitoneIiEditError::TrigAlreadyExists {
                pattern,
                track,
                step,
            });
        }
        let Some(slot) = self.freeSlot(base) else {
            return Err(DigitoneIiEditError::TrigPoolFull { pattern });
        };
        let at = base + TRIG_ARRAY_OFFSET + slot * TRIG_SLOT_SIZE;
        self.image[at] = u8::try_from(track).expect("track was checked");
        self.image[at + 1] = u8::try_from(step).expect("step was checked");
        self.writePayload(at, payload);
        let parity = if step.is_multiple_of(2) { 0 } else { 0x0010 };
        writeBe16(
            &mut self.image,
            base + TRACK_ARRAY_OFFSET + track * TRACK_SIZE + step * 2,
            parity | NOTE_TRIG_SHAPE,
        );
        Ok(slot)
    }

    #[must_use]
    pub fn inspect(&self) -> DigitoneIiInspection {
        let container = self.container();
        let mut patternsWithTrigs = 0;
        let mut trigCount = 0;
        let mut namedKitCount = 0;
        for index in 0..PATTERN_COUNT {
            let pattern = self
                .pattern(index)
                .expect("pattern index stays inside the constant range");
            if !pattern.trigs.is_empty() {
                patternsWithTrigs += 1;
                trigCount += pattern.trigs.len();
            }
            let kit = self
                .kit(index)
                .expect("kit index stays inside the constant range");
            if kit.present && !kit.name.is_empty() {
                namedKitCount += 1;
            }
        }
        DigitoneIiInspection {
            payloadName: self.manifest.payloadName.clone(),
            firmwareVersion: self.manifest.firmwareVersion.clone(),
            formatVersion: container.formatVersion,
            containerObjectVersion: container.objectVersion,
            projectObjectVersion: self.projectObjectVersion(),
            projectName: self.projectName(),
            identityToken: self.identityToken(),
            bank: container.bank,
            slot: container.slot,
            imageLength: self.image.len(),
            sourceCompressed: container.compressed,
            patternsWithTrigs,
            trigCount,
            namedKitCount,
        }
    }

    fn freeSlot(&self, patternBase: usize) -> Option<usize> {
        (0..TRIG_SLOT_COUNT).find(|slot| {
            self.image[patternBase + TRIG_ARRAY_OFFSET + slot * TRIG_SLOT_SIZE] == UNUSED_SLOT
        })
    }

    fn slotFor(&self, patternBase: usize, track: usize, step: usize) -> Option<usize> {
        self.readTrigs(patternBase).into_iter().find_map(|trig| {
            (usize::from(trig.track) == track && usize::from(trig.step) == step)
                .then_some(trig.slot)
        })
    }

    fn readSteps(&self, patternBase: usize) -> Vec<DigitoneIiStep> {
        let mut steps = Vec::new();
        for track in 0..TRACK_COUNT {
            let trackBase = patternBase + TRACK_ARRAY_OFFSET + track * TRACK_SIZE;
            for step in 0..STEP_COUNT {
                let flags = readBe16(&self.image, trackBase + STEP_FLAGS_OFFSET + step * 2);
                if flags & 0x0001 == 0 {
                    continue;
                }
                let kind = match flags & !TRIG_SHAPE_CLEAR {
                    NOTE_TRIG_SHAPE => DigitoneIiTrigKind::Note,
                    LOCK_TRIG_SHAPE => DigitoneIiTrigKind::Lock,
                    _ => DigitoneIiTrigKind::Unrecognised,
                };
                steps.push(DigitoneIiStep {
                    track: u8::try_from(track).expect("track count fits in a byte"),
                    step: u8::try_from(step).expect("step count fits in a byte"),
                    flags,
                    kind,
                    condition: optionalByte(self.image[trackBase + CONDITION_OFFSET + step]),
                    fill: optionalByte(self.image[trackBase + FILL_OFFSET + step]),
                    probability: optionalByte(self.image[trackBase + PROBABILITY_OFFSET + step]),
                    soundLock: optionalByte(self.image[trackBase + SOUND_LOCK_OFFSET + step]),
                });
            }
        }
        steps
    }

    fn readTrigs(&self, patternBase: usize) -> Vec<DigitoneIiTrig> {
        let mut trigs = Vec::new();
        for slot in 0..TRIG_SLOT_COUNT {
            let at = patternBase + TRIG_ARRAY_OFFSET + slot * TRIG_SLOT_SIZE;
            if self.image[at] == UNUSED_SLOT {
                continue;
            }
            trigs.push(DigitoneIiTrig {
                slot,
                track: self.image[at],
                step: self.image[at + 1],
                note: self.image[at + 2],
                velocity: self.image[at + 3],
                noteLength: self.image[at + 4],
                microTiming: self.image[at + 5].cast_signed(),
            });
        }
        trigs
    }

    fn readSound(&self, kitBase: usize, track: usize) -> DigitoneIiSound {
        let base = kitBase + SOUND_ARRAY_OFFSET + track * SOUND_SIZE;
        let present = self.hasMagic(base);
        DigitoneIiSound {
            track,
            present,
            version: present.then(|| readBe32(&self.image, base + 4)),
            name: if present {
                readName(
                    &self.image
                        [base + SOUND_NAME_OFFSET..base + SOUND_NAME_OFFSET + NAME_FIELD_LEN],
                )
            } else {
                String::new()
            },
            machine: present
                .then(|| DigitoneIiMachine::fromByte(self.image[base + MACHINE_OFFSET])),
        }
    }

    fn hasMagic(&self, offset: usize) -> bool {
        self.image.get(offset..offset + 4) == Some(&OBJECT_MAGIC)
    }

    fn soundOffset(&self, pattern: usize, track: usize) -> usize {
        kitOffset(pattern) + SOUND_ARRAY_OFFSET + track * SOUND_SIZE
    }

    fn requirePattern(&self, pattern: usize) -> Result<(), DigitoneIiEditError> {
        if pattern >= PATTERN_COUNT {
            return Err(DigitoneIiEditError::PatternOutOfRange { pattern });
        }
        Ok(())
    }

    fn requireTrack(&self, pattern: usize, track: usize) -> Result<(), DigitoneIiEditError> {
        self.requirePattern(pattern)?;
        if track >= TRACK_COUNT {
            return Err(DigitoneIiEditError::TrackOutOfRange { track });
        }
        Ok(())
    }

    fn requireVersion3(&self, pattern: usize) -> Result<(), DigitoneIiEditError> {
        self.requirePattern(pattern)?;
        let version = readBe32(&self.image, patternOffset(pattern));
        if version != 3 {
            return Err(DigitoneIiEditError::UnrecognisedPatternVersion { pattern, version });
        }
        Ok(())
    }

    fn requirePayload(&self, payload: DigitoneIiTrigPayload) -> Result<(), DigitoneIiEditError> {
        if payload.note > 127 {
            return Err(DigitoneIiEditError::NoteOutOfRange { note: payload.note });
        }
        if payload.velocity != INHERIT && payload.velocity > 127 {
            return Err(DigitoneIiEditError::VelocityOutOfRange {
                velocity: payload.velocity,
            });
        }
        if payload.noteLength != INHERIT && payload.noteLength > 127 {
            return Err(DigitoneIiEditError::LengthOutOfRange {
                noteLength: payload.noteLength,
            });
        }
        if !(-23..=23).contains(&payload.microTiming) {
            return Err(DigitoneIiEditError::MicroTimingOutOfRange {
                microTiming: payload.microTiming,
            });
        }
        Ok(())
    }

    fn writePayload(&mut self, at: usize, payload: DigitoneIiTrigPayload) {
        self.image[at + 2] = payload.note;
        self.image[at + 3] = payload.velocity;
        self.image[at + 4] = payload.noteLength;
        self.image[at + 5] = payload.microTiming.cast_unsigned();
    }
}

/// Bank letter and 1-based slot, matching the device (`0` is `A1`, `97` is `G2`).
#[must_use]
pub fn patternLabel(index: usize) -> String {
    let bank = char::from(b'A' + u8::try_from(index / 16).unwrap_or(0));
    let slot = index % 16 + 1;
    format!("{bank}{slot}")
}

/// Length byte as sixteenths of a step.
#[must_use]
pub fn noteLengthSixteenths(byte: u8) -> DigitoneIiNoteLength {
    match byte {
        INHERIT => DigitoneIiNoteLength::Inherit,
        127 => DigitoneIiNoteLength::Infinite,
        0..=29 => DigitoneIiNoteLength::Sixteenths(u32::from(byte) + 2),
        30..=45 => DigitoneIiNoteLength::Sixteenths(32 + u32::from(byte - 30) * 2),
        46..=61 => DigitoneIiNoteLength::Sixteenths(64 + u32::from(byte - 46) * 4),
        62..=77 => DigitoneIiNoteLength::Sixteenths(128 + u32::from(byte - 62) * 8),
        78..=93 => DigitoneIiNoteLength::Sixteenths(256 + u32::from(byte - 78) * 16),
        94..=109 => DigitoneIiNoteLength::Sixteenths(512 + u32::from(byte - 94) * 32),
        110..=126 => DigitoneIiNoteLength::Sixteenths(1024 + u32::from(byte - 110) * 64),
        128..=254 => DigitoneIiNoteLength::Unrecognised(byte),
    }
}

/// Name for a verified trig-condition code. Unknown and predicted codes return `None`.
#[must_use]
pub fn conditionLabel(code: u8) -> Option<&'static str> {
    Some(match code {
        0x00 => "PRE",
        0x01 => "not PRE",
        0x02 => "NEI",
        0x03 => "not NEI",
        0x04 => "1ST",
        0x05 => "not 1ST",
        0x06 => "LST",
        0x07 => "not LST",
        0x08 => "1:2",
        0x09 => "2:2",
        0x0A => "1:3",
        0x0B => "not 1:3",
        0x0C => "2:3",
        0x0D => "not 2:3",
        0x0E => "3:3",
        0x0F => "not 3:3",
        0x10 => "1:4",
        0x11 => "not 1:4",
        0x12 => "2:4",
        0x14 => "3:4",
        0x16 => "4:4",
        0x17 => "not 4:4",
        0x2C => "6:6",
        0x2D => "not 6:6",
        0x3C => "1:8",
        0x3D => "not 1:8",
        0x4A => "8:8",
        0x4B => "not 8:8",
        _ => return None,
    })
}

fn optionalByte(byte: u8) -> Option<u8> {
    if byte == UNUSED_SLOT {
        None
    } else {
        Some(byte)
    }
}

fn patternOffset(index: usize) -> usize {
    HEADER_SIZE + index * PATTERN_SIZE
}

fn kitOffset(index: usize) -> usize {
    KIT_BASE + index * KIT_SIZE
}

fn readBe16(data: &[u8], at: usize) -> u16 {
    u16::from_be_bytes([data[at], data[at + 1]])
}

fn readLe16(data: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([data[at], data[at + 1]])
}

fn readBe32(data: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]])
}

fn writeBe16(data: &mut [u8], at: usize, value: u16) {
    data[at..at + 2].copy_from_slice(&value.to_be_bytes());
}

fn readName(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    bytes[..end].iter().copied().map(char::from).collect()
}

fn writeName(destination: &mut [u8], name: &str) {
    destination.fill(0);
    let mut at = 0;
    for character in name.chars() {
        if at + 1 >= destination.len() {
            break;
        }
        let code = u32::from(character);
        if (0x20..=0xFF).contains(&code) {
            destination[at] = u8::try_from(code).expect("code was checked against 0xFF");
            at += 1;
        }
    }
}
