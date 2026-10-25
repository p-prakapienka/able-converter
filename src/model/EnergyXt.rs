//! energyXT 3.0 project source model: the sequencer subset the converter reads.
//!
//! Field meanings follow the energyxt-format study (see `docs/energyxt-format.md`).
//! Times are energyXT ticks; conversion to beats happens in the mapper.

use serde::{Deserialize, Serialize};

/// energyXT counts 960 ticks per quarter note.
pub const ENERGYXT_TICKS_PER_BEAT: f64 = 960.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnergyXtProject {
    /// The project tempo in BPM (`bpm` record).
    pub tempo: Option<f64>,
    pub tracks: Vec<EnergyXtTrack>,
    /// Parts in the part pool. Placed parts may play one of these by index.
    pub pool: Vec<EnergyXtPart>,
    /// Keys of sequencer records the converter recognises but does not map, such
    /// as `tempo`, `timesig`, `marker` and `pattern`, in file order.
    pub unmappedRecords: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergyXtTrackKind {
    Folder,
    Instrument,
    Audio,
    Drum,
    Unknown(u32),
}

impl EnergyXtTrackKind {
    #[must_use]
    pub fn fromCode(code: u32) -> Self {
        match code {
            1 => Self::Folder,
            2 => Self::Instrument,
            3 => Self::Audio,
            5 => Self::Drum,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnergyXtTrack {
    /// Position of the track in the sequencer, starting at 0.
    pub index: usize,
    pub kind: EnergyXtTrackKind,
    pub name: String,
    /// The `trackCol` value, an energyXT colour rather than a Live palette index.
    pub color: Option<u32>,
    /// Index of the folder track holding this track.
    pub parent: Option<usize>,
    /// MIDI channel from the track record, 0-based; negative when unset.
    pub midiChannel: i8,
    /// Placed parts in file order.
    pub parts: Vec<EnergyXtPart>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergyXtPartKind {
    Midi,
    Audio,
    Drum,
    Unknown(u8),
}

impl EnergyXtPartKind {
    #[must_use]
    pub fn fromCode(code: u8) -> Self {
        match code {
            1 => Self::Midi,
            2 => Self::Audio,
            3 => Self::Drum,
            other => Self::Unknown(other),
        }
    }
}

pub const PART_FLAG_SELECTED: i32 = 1;
pub const PART_FLAG_MUTED: i32 = 4;

/// One part: a placed part on a track or an entry of the part pool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnergyXtPart {
    pub kind: EnergyXtPartKind,
    /// Index into the pool when this placed part plays a pool part.
    pub poolIndex: Option<usize>,
    /// Where the part begins on its track, in ticks.
    pub start: i32,
    /// Where the events' time 0 lies relative to `start`, in ticks.
    pub offset: i32,
    /// The part plays `[start, start + length)`.
    pub length: i32,
    pub flags: i32,
    pub name: String,
    /// The `partCol` value; `None` for the default colour or a pool reference.
    pub color: Option<u32>,
    pub midiEvents: Vec<EnergyXtMidiEvent>,
    pub drumHits: Vec<EnergyXtDrumHit>,
    /// The part carries audio events, which the converter does not map.
    pub hasAudioEvents: bool,
    /// The part carries automation lanes (`amevents`).
    pub hasAutomation: bool,
}

impl EnergyXtPart {
    #[must_use]
    pub fn isMuted(&self) -> bool {
        self.flags & PART_FLAG_MUTED != 0
    }

    /// The absolute tick of an event at `eventTick` in an event list played by
    /// this part.
    #[must_use]
    pub fn eventTime(&self, eventTick: u32) -> i64 {
        i64::from(self.start) + i64::from(self.offset) + i64::from(eventTick)
    }

    /// Whether an event at that absolute tick is inside the part and plays.
    #[must_use]
    pub fn contains(&self, tick: i64) -> bool {
        tick >= i64::from(self.start) && tick < i64::from(self.start) + i64::from(self.length)
    }
}

/// One `mevents` entry with its time resolved to ticks from the list start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnergyXtMidiEvent {
    pub tick: u32,
    pub status: u8,
    pub data1: u8,
    pub data2: u8,
    /// Note length in ticks, present on note-ons only.
    pub length: Option<u32>,
}

impl EnergyXtMidiEvent {
    #[must_use]
    pub fn isNoteOn(&self) -> bool {
        self.status & 0xF0 == 0x90
    }
}

/// Value ids of a drum hit.
pub const DRUM_VALUE_VOLUME: u8 = 0;
pub const DRUM_VALUE_PAN: u8 = 1;
pub const DRUM_VALUE_CUTOFF: u8 = 2;

/// One `devents` entry with its time resolved to ticks from the list start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnergyXtDrumHit {
    pub tick: u32,
    pub lane: u8,
    pub length: u32,
    /// Values that differ from their defaults: volume 1, pan 0, cutoff 1.
    pub values: Vec<(u8, f32)>,
}

impl EnergyXtDrumHit {
    #[must_use]
    pub fn value(&self, id: u8) -> Option<f32> {
        self.values
            .iter()
            .find(|(valueId, _)| *valueId == id)
            .map(|(_, value)| *value)
    }
}
