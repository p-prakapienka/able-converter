use crate::model::live::LiveMidiNote;

use super::livereaderror::LiveReadError;
use super::pendingmidinote::PendingMidiNote;

#[derive(Default)]
pub(super) struct KeyTrackBuilder {
    pitch: Option<u16>,
    notes: Vec<PendingMidiNote>,
}

impl KeyTrackBuilder {
    pub(super) fn setPitch(&mut self, pitch: u16) {
        self.pitch = Some(pitch);
    }

    pub(super) fn addNote(&mut self, note: PendingMidiNote) {
        self.notes.push(note);
    }

    pub(super) fn finish(self) -> Result<Vec<LiveMidiNote>, LiveReadError> {
        if self.notes.is_empty() {
            return Ok(Vec::new());
        }

        let pitch = self.pitch.ok_or(LiveReadError::MissingElement {
            parent: "KeyTrack",
            element: "MidiKey",
        })?;

        Ok(self
            .notes
            .into_iter()
            .map(|note| note.withPitch(pitch))
            .collect())
    }
}
