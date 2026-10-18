//! Korg electribe sampler (firmware 2.x) source models produced before semantic mapping.
//!
//! Field meanings and offsets are documented in `docs/electribe-format.md`. Values are
//! kept in their native representation; musical interpretation happens in the mapper.

use serde::{Deserialize, Serialize};

/// Number of parts in every electribe sampler pattern.
pub const ELECTRIBE_PART_COUNT: usize = 16;
/// Number of stored steps per part (four bars of sixteen step slots).
pub const ELECTRIBE_STEP_COUNT: usize = 64;
/// Number of motion-sequence slots per pattern.
pub const ELECTRIBE_MOTION_SLOT_COUNT: usize = 24;
/// Number of patterns in an `.e2sallpat` file.
pub const ELECTRIBE_PATTERN_COUNT: usize = 250;
/// Motion slot part number used for pattern-level (master effect) motion.
pub const ELECTRIBE_PATTERN_MOTION_PART: u8 = 17;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElectribeFileKind {
    /// `.e2spat`: one pattern.
    Pattern,
    /// `.e2sallpat`: the global block followed by all 250 patterns.
    AllPatterns,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElectribeProject {
    pub fileKind: ElectribeFileKind,
    /// Product identifier from the file header, for example `e2sampler`.
    pub product: String,
    pub patterns: Vec<ElectribePattern>,
}

/// Step resolution selected by the pattern's BEAT parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElectribeBeat {
    Sixteenth,
    ThirtySecond,
    EighthTriplet,
    SixteenthTriplet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElectribePattern {
    /// One-based pattern slot number on the device.
    pub number: usize,
    pub name: String,
    /// Tempo in tenths of a beat per minute.
    pub tempoTenths: u16,
    /// Swing amount, -48 to +48.
    pub swing: i8,
    /// Pattern length in bars, 1 to 4.
    pub lengthBars: u8,
    pub beat: ElectribeBeat,
    /// Key, 0 = C to 11 = B.
    pub key: u8,
    /// Scale index as listed by the device, 0 = Chromatic.
    pub scale: u8,
    pub chordSet: u8,
    /// Pattern level as stored; the device level is `127 - levelRaw`.
    pub levelRaw: u8,
    pub gateArp: u8,
    pub masterEffect: ElectribeMasterEffect,
    pub alternate1314: bool,
    pub alternate1516: bool,
    /// Chain target pattern number, 0 when no chain is set.
    pub chainTo: u16,
    pub chainRepeat: u8,
    pub parts: Vec<ElectribePart>,
    pub motionSlots: Vec<ElectribeMotionSlot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElectribeMasterEffect {
    pub on: bool,
    pub effectType: u8,
    pub padX: u8,
    pub padY: u8,
    pub hold: bool,
    /// Byte `0x40` of the pattern. Its meaning is not yet established.
    pub unknown40: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElectribePart {
    /// One-based part number.
    pub number: usize,
    /// Steps per bar used by this part; 0 selects the beat's default.
    pub lastStep: u8,
    pub mute: bool,
    pub voiceAssign: u8,
    pub motionSequence: u8,
    pub triggerPadVelocity: bool,
    pub scaleMode: bool,
    pub partPriority: u8,
    pub oscillator: u16,
    /// Byte `0x0A` of the part. Its meaning is not yet established.
    pub unknown0A: u8,
    pub oscillatorEdit: u8,
    pub filterType: u8,
    pub cutoff: u8,
    pub resonance: u8,
    pub egIntensity: i8,
    pub modulationType: u8,
    pub modulationSpeed: u8,
    pub modulationDepth: u8,
    pub attack: u8,
    pub decayRelease: u8,
    pub level: u8,
    pub pan: i8,
    pub ampEg: bool,
    pub mfxSend: bool,
    pub grooveType: u8,
    pub grooveDepth: u8,
    pub ifxOn: bool,
    pub ifxType: u8,
    pub ifxEdit: u8,
    pub pitch: i8,
    pub glide: u8,
    /// Byte `0x26` of the part. Its meaning is not yet established.
    pub unknown26: u8,
    pub steps: Vec<ElectribeStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElectribeStep {
    /// Zero-based storage index, `bar * 16 + stepInBar`.
    pub index: usize,
    pub triggered: bool,
    /// Gate time 0 to 96; values above 96 tie into the next step.
    pub gate: u8,
    pub velocity: u8,
    /// Byte 3 of the step. Its meaning is not yet established.
    pub unknownFlag: u8,
    /// Note slots as stored: MIDI note number plus one, zero for an empty slot.
    pub noteSlots: [u8; 8],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElectribeMotionSlot {
    pub slot: usize,
    /// Part number 1 to 16, or 17 for pattern-level motion.
    pub part: u8,
    /// Motion parameter identifier in the device's motion-specific numbering.
    pub parameterId: u8,
    pub values: Vec<u8>,
}
