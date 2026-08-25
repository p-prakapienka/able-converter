use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectPreview {
    pub summary: ProjectPreviewSummary,
    pub scenes: Vec<PreviewScene>,
    pub sourceTracks: Vec<PreviewTrack>,
    pub targetTracks: Vec<PreviewTrack>,
    pub compatibility: CompatibilityReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProjectPreviewSummary {
    pub tempo: Option<f64>,
    pub sourceTrackCount: usize,
    pub targetTrackCount: usize,
    pub sceneCount: usize,
    pub sourceClipCount: usize,
    pub targetClipCount: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewScene {
    pub index: usize,
    pub name: String,
    pub color: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewTrack {
    pub id: String,
    pub name: String,
    pub color: Option<i32>,
    pub clips: Vec<PreviewClip>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewClip {
    pub id: String,
    pub sourceId: Option<String>,
    pub name: String,
    pub trackId: String,
    pub sceneIndex: usize,
    pub start: f64,
    pub end: f64,
    pub loopStart: f64,
    pub loopEnd: f64,
    pub loopEnabled: bool,
    pub enabled: bool,
    pub notes: Vec<PreviewNote>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PreviewNote {
    pub pitch: u8,
    pub start: f64,
    pub duration: f64,
    pub velocity: f32,
    pub offVelocity: Option<f32>,
    pub probability: Option<f32>,
    pub state: PreviewNoteState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PreviewNoteState {
    Source,
    Lossy,
    Omitted,
    Mapped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub warningCount: usize,
    pub errorCount: usize,
    pub diagnostics: Vec<PreviewDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewDiagnostic {
    pub severity: String,
    pub code: String,
    pub sourceId: String,
    pub message: String,
}
