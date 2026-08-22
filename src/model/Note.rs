//! Verified subset of the Ableton Note/Move Set JSON model.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const NOTE_SCHEMA_URI: &str = "http://tech.ableton.com/schema/song/1.8.3/song.json";
pub const NOTE_TRACK_LIMIT: usize = 8;
pub const NOTE_SCENE_LIMIT: usize = 8;
pub const NOTE_CLIP_BEAT_LIMIT: f64 = 64.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteProject {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub stepEditorResolution: String,
    pub tempo: f64,
    pub globalGrooveAmount: f64,
    pub timeSignature: NoteTimeSignature,
    pub rootNote: u8,
    pub scale: String,
    pub melodicLayout: String,
    pub tracks: Vec<NoteTrack>,
    pub returnTracks: Vec<NoteTrack>,
    pub masterTrack: NoteMasterTrack,
    pub scenes: Vec<NoteScene>,
    pub grooves: Vec<NoteGroove>,
    pub metadata: NoteMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteTimeSignature {
    pub upper: u8,
    pub lower: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteTrack {
    pub kind: String,
    pub name: String,
    pub color: i32,
    pub isSelected: bool,
    pub clipSlots: Vec<NoteClipSlot>,
    pub isArmed: bool,
    pub isNoteRepeatOn: bool,
    pub noteRepeatRate: String,
    pub noteRepeatArpeggio: NoteRepeatArpeggio,
    pub uiOctaveIndex: u8,
    pub midiInputMode: String,
    pub midiOutputEndpoint: Option<String>,
    pub devices: Vec<NoteDevice>,
    pub mixer: NoteMixer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteClipSlot {
    pub hasStop: bool,
    pub clip: Option<NoteMidiClip>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteMidiClip {
    pub isPlaying: bool,
    pub name: String,
    pub color: i32,
    pub isEnabled: bool,
    pub region: NoteClipRegion,
    pub grooveId: u32,
    pub stepEditorScrollPosition: f64,
    pub notes: Vec<NoteMidiNote>,
    pub envelopes: Vec<NoteEnvelope>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NoteClipRegion {
    pub start: f64,
    pub end: f64,
    pub r#loop: NoteLoop,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NoteLoop {
    pub start: f64,
    pub end: f64,
    pub isEnabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NoteMidiNote {
    pub noteNumber: u8,
    pub startTime: f64,
    pub duration: f64,
    pub velocity: f32,
    pub offVelocity: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteEnvelope {
    pub parameterId: u32,
    pub breakpoints: Vec<NoteBreakpoint>,
    pub region: Option<NoteClipRegion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NoteBreakpoint {
    pub time: f64,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteRepeatArpeggio {
    pub style: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteDevice {
    pub presetUri: Option<String>,
    pub kind: String,
    pub name: String,
    pub lockId: i32,
    pub lockSeal: i32,
    pub parameters: BTreeMap<String, Value>,
    pub chains: Vec<NoteDeviceChain>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteDeviceChain {
    pub name: String,
    pub color: i32,
    pub devices: Vec<NoteDevice>,
    pub mixer: NoteMixer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteMixer {
    pub pan: f64,
    #[serde(rename = "solo-cue")]
    pub soloCue: bool,
    pub speakerOn: bool,
    pub volume: f64,
    pub sends: Vec<NoteSend>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NoteSend {
    pub isEnabled: bool,
    pub amount: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteMasterTrack {
    pub color: i32,
    pub isSelected: bool,
    pub devices: Vec<NoteDevice>,
    pub mixer: NoteMasterMixer,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NoteMasterMixer {
    pub pan: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteScene {
    pub name: String,
    pub color: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteGroove {
    pub id: u32,
    pub name: String,
    pub base: String,
    pub r#loop: NoteGrooveLoop,
    pub events: Vec<NoteGrooveEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NoteGrooveLoop {
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NoteGrooveEvent {
    pub time: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteMetadata {
    pub usedFeatures: Vec<String>,
}
