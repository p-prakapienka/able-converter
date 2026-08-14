//! Ableton Live-specific models produced before semantic mapping.

use serde::{Deserialize, Serialize};

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
