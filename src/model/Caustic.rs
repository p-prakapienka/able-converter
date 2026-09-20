//! Caustic 3-specific models produced before semantic mapping.
//!
//! The `.caustic` song format is undocumented. These models describe only the
//! container structure that has been established from the shipped engine binary,
//! and preserve every byte whose meaning is still unknown. See
//! `docs/caustic-format.md` for the evidence behind each field.

use serde::{Deserialize, Serialize};

/// Number of machine slots written by the rack, occupied or not.
pub const MACHINE_SLOT_COUNT: usize = 14;

/// Length of the fixed block that follows the `RACK` chunk header.
pub const RACK_HEADER_LENGTH: usize = 264;

/// Identifier written into an unoccupied machine slot.
pub const EMPTY_MACHINE_ID: &str = "NULL";

/// Four-character chunk identifiers recognised inside the rack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CausticSectionTag {
    /// Transport and machine table.
    OutputPanel,
    /// Master effect chain.
    Effects,
    /// Machine mixer.
    Mixer,
    /// Master section.
    Master,
    /// Song sequencer.
    Sequencer,
}

impl CausticSectionTag {
    pub fn fromBytes(tag: &[u8; 4]) -> Option<Self> {
        match tag {
            b"OUTP" => Some(Self::OutputPanel),
            b"EFFX" => Some(Self::Effects),
            b"MIXR" => Some(Self::Mixer),
            b"MSTR" => Some(Self::Master),
            b"SEQN" => Some(Self::Sequencer),
            _ => None,
        }
    }

    pub fn asBytes(self) -> &'static [u8; 4] {
        match self {
            Self::OutputPanel => b"OUTP",
            Self::Effects => b"EFFX",
            Self::Mixer => b"MIXR",
            Self::Master => b"MSTR",
            Self::Sequencer => b"SEQN",
        }
    }
}

/// A single automatable control value inside a `CCOL` collection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CausticControl {
    /// Engine-assigned control identifier. Names are resolved by the engine's
    /// per-machine control tables and are not stored in the file.
    pub id: u32,
    pub value: f32,
}

/// A `CCOL` control collection as written by a machine or effect.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CausticControlCollection {
    pub controls: Vec<CausticControl>,
}

impl CausticControlCollection {
    pub fn valueOf(&self, id: u32) -> Option<f32> {
        self.controls
            .iter()
            .find(|control| control.id == id)
            .map(|control| control.value)
    }
}

/// One occupied machine slot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausticMachine {
    /// Zero-based index into the rack's fixed slot table.
    pub slot: usize,
    /// Four-character machine identifier such as `SSYN` or `PCMS`.
    pub machineId: String,
    /// Display name as stored in the slot entry.
    pub name: String,
    /// Four bytes written between the machine name and its payload length.
    pub headerBytes: Vec<u8>,
    /// Control collection when the machine body opens with a `CCOL` chunk.
    pub controls: Option<CausticControlCollection>,
    /// Machine body after any leading control collection, kept verbatim because
    /// per-machine field layouts are not decoded yet.
    pub body: Vec<u8>,
}

/// An unoccupied machine slot, preserved so slot order survives a round trip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CausticMachineSlot {
    pub slot: usize,
    pub machineId: String,
    pub occupied: bool,
}

/// A top-level section of the rack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausticSection {
    pub tag: CausticSectionTag,
    /// Section payload exactly as stored.
    pub payload: Vec<u8>,
    /// Bytes found between this section's payload and the next recognised tag.
    /// Their meaning is unknown; they are preserved rather than discarded.
    pub trailingBytes: Vec<u8>,
}

/// Values read from the `OUTP` payload.
///
/// Both fields are provisional: the offsets are established by existing
/// third-party decoders and have not been confirmed against the engine binary.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct CausticTransport {
    pub tempo: Option<f32>,
    pub beatsPerBar: Option<u8>,
}

/// High-level structure of a song without its machine payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausticInspection {
    pub rackLength: usize,
    pub transport: CausticTransport,
    pub slots: Vec<CausticMachineSlot>,
    pub sectionTags: Vec<CausticSectionTag>,
    pub occupiedSlots: usize,
    pub controlCount: usize,
}

/// A parsed `.caustic` song.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausticProject {
    pub inspection: CausticInspection,
    /// Fixed block following the `RACK` chunk header, kept verbatim. Any stored
    /// format version is expected to live here.
    pub rackHeader: Vec<u8>,
    pub machines: Vec<CausticMachine>,
    pub sections: Vec<CausticSection>,
    /// Bytes after the last recognised section.
    pub trailingBytes: Vec<u8>,
}
