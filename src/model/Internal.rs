//! Canonical musical model shared by every source and destination format.

use serde::{Deserialize, Serialize};

/// A musical position measured in quarter-note beats.
///
/// Format adapters are responsible for converting their native time representation
/// at the boundary. Keeping this wrapper explicit prevents accidental mixing with
/// seconds or raw format values.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Beat(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BeatRange {
    pub start: Beat,
    pub end: Beat,
}

impl BeatRange {
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.start.0.is_finite() && self.end.0.is_finite() && self.end.0 >= self.start.0
    }
}

/// Identifies whether a canonical clip came from a Session slot or Arrangement range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClipSource {
    Session {
        track_id: String,
        scene_index: usize,
    },
    Arrangement {
        track_id: String,
        timeline_range: BeatRange,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub pitch: u8,
    pub start: Beat,
    pub duration: Beat,
    pub velocity: u8,
    pub muted: bool,
    pub probability: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MidiClip {
    pub id: String,
    pub name: String,
    pub source: ClipSource,
    pub length: Beat,
    pub loop_region: Option<BeatRange>,
    pub notes: Vec<Note>,
}
