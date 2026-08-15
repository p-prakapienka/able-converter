use crate::model::live::{LiveLoop, LiveMidiClip, LiveMidiNote};

use super::livereaderror::LiveReadError;

pub(super) struct LiveMidiClipBuilder {
    id: String,
    trackId: String,
    sceneIndex: usize,
    name: String,
    currentStart: Option<f64>,
    currentEnd: Option<f64>,
    loopSettings: Option<LiveLoop>,
    disabled: bool,
    notes: Vec<LiveMidiNote>,
    hasClipAutomation: bool,
    hasPerNoteExpression: bool,
}

impl LiveMidiClipBuilder {
    pub(super) fn new(id: String, trackId: String, sceneIndex: usize) -> Self {
        Self {
            id,
            trackId,
            sceneIndex,
            name: String::new(),
            currentStart: None,
            currentEnd: None,
            loopSettings: None,
            disabled: false,
            notes: Vec::new(),
            hasClipAutomation: false,
            hasPerNoteExpression: false,
        }
    }

    pub(super) fn setName(&mut self, name: String) {
        self.name = name;
    }

    pub(super) fn setCurrentStart(&mut self, currentStart: f64) {
        self.currentStart = Some(currentStart);
    }

    pub(super) fn setCurrentEnd(&mut self, currentEnd: f64) {
        self.currentEnd = Some(currentEnd);
    }

    pub(super) fn setLoopSettings(&mut self, loopSettings: LiveLoop) {
        self.loopSettings = Some(loopSettings);
    }

    pub(super) fn setDisabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub(super) fn addNotes(&mut self, notes: Vec<LiveMidiNote>) {
        self.notes.extend(notes);
    }

    pub(super) fn markClipAutomation(&mut self) {
        self.hasClipAutomation = true;
    }

    pub(super) fn markPerNoteExpression(&mut self) {
        self.hasPerNoteExpression = true;
    }

    pub(super) fn finish(self) -> Result<LiveMidiClip, LiveReadError> {
        Ok(LiveMidiClip {
            id: self.id,
            trackId: self.trackId,
            sceneIndex: self.sceneIndex,
            name: self.name,
            currentStart: self.currentStart.ok_or(LiveReadError::MissingElement {
                parent: "MidiClip",
                element: "CurrentStart",
            })?,
            currentEnd: self.currentEnd.ok_or(LiveReadError::MissingElement {
                parent: "MidiClip",
                element: "CurrentEnd",
            })?,
            loopSettings: self.loopSettings,
            disabled: self.disabled,
            notes: self.notes,
            hasClipAutomation: self.hasClipAutomation,
            hasPerNoteExpression: self.hasPerNoteExpression,
        })
    }
}
