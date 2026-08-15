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
    pub returnTracks: usize,
    pub main: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipCounts {
    pub sessionMidi: usize,
    pub arrangementMidi: usize,
    pub unclassifiedMidi: usize,
    pub sessionAudio: usize,
    pub arrangementAudio: usize,
    pub unclassifiedAudio: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveSetInspection {
    pub format: LiveFormatVersion,
    pub tempo: Option<f64>,
    pub tracks: TrackCounts,
    pub clips: ClipCounts,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveProject {
    pub inspection: LiveSetInspection,
    pub sessionMidiClips: Vec<LiveMidiClip>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveMidiClip {
    pub id: String,
    pub trackId: String,
    pub sceneIndex: usize,
    pub name: String,
    pub currentStart: f64,
    pub currentEnd: f64,
    pub loopSettings: Option<LiveLoop>,
    pub disabled: bool,
    pub notes: Vec<LiveMidiNote>,
    pub hasClipAutomation: bool,
    pub hasPerNoteExpression: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LiveLoop {
    pub start: f64,
    pub end: f64,
    pub startRelative: f64,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveMidiNote {
    pub id: Option<String>,
    pub pitch: u16,
    pub time: f64,
    pub duration: f64,
    pub velocity: f32,
    pub releaseVelocity: Option<f32>,
    pub velocityDeviation: Option<f32>,
    pub probability: Option<f32>,
    pub enabled: Option<bool>,
}
