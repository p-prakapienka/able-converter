//! Platform-neutral models shared by parsers, converters, and user interfaces.

use serde::{Deserialize, Serialize};

/// A musical position measured in quarter-note beats.
///
/// Format adapters are responsible for converting their native time representation
/// at the boundary. Keeping this wrapper explicit prevents accidental mixing with
/// seconds or raw format values.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Beat(pub f64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveFormatVersion {
    pub major: Option<String>,
    pub minor: Option<String>,
    pub creator: Option<String>,
    pub revision: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackCounts {
    pub midi: usize,
    pub audio: usize,
    pub group: usize,
    pub return_tracks: usize,
    pub main: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipCounts {
    pub session_midi: usize,
    pub arrangement_midi: usize,
    pub unclassified_midi: usize,
    pub session_audio: usize,
    pub arrangement_audio: usize,
    pub unclassified_audio: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveSetInspection {
    pub format: LiveFormatVersion,
    pub tempo: Option<f64>,
    pub tracks: TrackCounts,
    pub clips: ClipCounts,
}

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

#[cfg(test)]
mod tests {
    use super::{Beat, BeatRange};

    #[test]
    fn beat_range_rejects_backwards_and_non_finite_ranges() {
        assert!(
            BeatRange {
                start: Beat(0.0),
                end: Beat(4.0),
            }
            .is_valid()
        );
        assert!(
            !BeatRange {
                start: Beat(4.0),
                end: Beat(0.0),
            }
            .is_valid()
        );
        assert!(
            !BeatRange {
                start: Beat(0.0),
                end: Beat(f64::NAN),
            }
            .is_valid()
        );
    }
}
