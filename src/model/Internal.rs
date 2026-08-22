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
    pub fn isValid(self) -> bool {
        self.start.0.is_finite() && self.end.0.is_finite() && self.end.0 >= self.start.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClipLoop {
    pub region: BeatRange,
    pub startRelative: Beat,
    pub enabled: bool,
}

/// Identifies whether a canonical clip came from a Session slot or Arrangement range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClipSource {
    Session {
        trackId: String,
        sceneIndex: usize,
    },
    Arrangement {
        trackId: String,
        timelineRange: BeatRange,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub pitch: u8,
    pub start: Beat,
    pub duration: Beat,
    pub velocity: f32,
    pub releaseVelocity: Option<f32>,
    pub muted: bool,
    pub probability: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MidiClip {
    pub id: String,
    pub name: String,
    pub source: ClipSource,
    pub contentRange: BeatRange,
    pub loopSettings: Option<ClipLoop>,
    pub disabled: bool,
    pub notes: Vec<Note>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackKind {
    Midi,
    Audio,
    Group,
    Return,
    Main,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub kind: TrackKind,
    pub name: String,
    pub color: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    pub index: usize,
    pub name: String,
    pub color: Option<i32>,
    pub tempoOverride: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub tempo: Option<f64>,
    pub tracks: Vec<Track>,
    pub scenes: Vec<Scene>,
    pub midiClips: Vec<MidiClip>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticCode {
    InvalidClipRange,
    InvalidLoopRange,
    InvalidNote,
    UnsupportedClipAutomation,
    UnsupportedPerNoteExpression,
    UnsupportedVelocityDeviation,
    InvalidSceneTempo,
    UnsupportedSceneTimeSignature,
    MissingProjectTempo,
    NoteTrackLimitExceeded,
    NoteSceneLimitExceeded,
    UnknownTrackSelection,
    UnknownSceneSelection,
    UnsupportedTrackKind,
    DuplicateSessionSlot,
    NoteClipLengthExceeded,
    UnsupportedMutedNote,
    UnsupportedNoteProbability,
    UnsupportedLoopStartRelative,
    UnsupportedSceneTempoOverride,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: DiagnosticCode,
    pub sourceId: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MappingResult<T> {
    pub value: T,
    pub diagnostics: Vec<Diagnostic>,
}

impl<T> MappingResult<T> {
    #[must_use]
    pub fn hasErrors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}
