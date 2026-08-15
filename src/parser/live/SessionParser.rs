use quick_xml::events::BytesStart;

use crate::model::live::LiveMidiClip;

use super::keytrackbuilder::KeyTrackBuilder;
use super::liveelement::LiveElement;
use super::liveloopbuilder::LiveLoopBuilder;
use super::livemidiclipbuilder::LiveMidiClipBuilder;
use super::livereaderror::LiveReadError;
use super::pendingmidinote::PendingMidiNote;

#[derive(Default)]
pub(super) struct SessionParser {
    currentTrackId: Option<String>,
    clipSlotDepth: usize,
    currentSceneIndex: Option<usize>,
    currentClip: Option<LiveMidiClipBuilder>,
    currentLoop: Option<LiveLoopBuilder>,
    currentKeyTrack: Option<KeyTrackBuilder>,
    clips: Vec<LiveMidiClip>,
}

impl SessionParser {
    pub(super) fn onStart(
        &mut self,
        name: &[u8],
        element: &BytesStart<'_>,
        parent: Option<&[u8]>,
        sessionDepth: usize,
    ) -> Result<(), LiveReadError> {
        match name {
            b"MidiTrack" => self.currentTrackId = LiveElement::new(element).attribute(b"Id")?,
            b"ClipSlot" if sessionDepth > 0 => self.startClipSlot(element)?,
            b"MidiClip" if sessionDepth > 0 => self.startClip(element)?,
            b"Loop" if self.currentClip.is_some() => self.startLoop()?,
            b"KeyTrack" if self.currentClip.is_some() => self.startKeyTrack()?,
            _ => self.readClipElement(name, element, parent)?,
        }

        Ok(())
    }

    pub(super) fn onEmpty(
        &mut self,
        name: &[u8],
        element: &BytesStart<'_>,
        parent: Option<&[u8]>,
        sessionDepth: usize,
    ) -> Result<(), LiveReadError> {
        match name {
            b"MidiClip" if sessionDepth > 0 => {
                self.startClip(element)?;
                self.finishClip()?;
            }
            b"Loop" if self.currentClip.is_some() => {
                self.startLoop()?;
                self.finishLoop()?;
            }
            b"KeyTrack" if self.currentClip.is_some() => {
                self.startKeyTrack()?;
                self.finishKeyTrack()?;
            }
            _ => self.readClipElement(name, element, parent)?,
        }

        Ok(())
    }

    pub(super) fn onEnd(&mut self, name: &[u8]) -> Result<(), LiveReadError> {
        match name {
            b"MidiTrack" => self.currentTrackId = None,
            b"ClipSlot" if self.clipSlotDepth > 0 => self.finishClipSlot(),
            b"MidiClip" if self.currentClip.is_some() => self.finishClip()?,
            b"Loop" if self.currentLoop.is_some() => self.finishLoop()?,
            b"KeyTrack" if self.currentKeyTrack.is_some() => self.finishKeyTrack()?,
            _ => {}
        }

        Ok(())
    }

    pub(super) fn finish(self) -> Result<Vec<LiveMidiClip>, LiveReadError> {
        if self.currentClip.is_some() {
            return Err(LiveReadError::UnexpectedStructure("unclosed MidiClip"));
        }
        Ok(self.clips)
    }

    fn startLoop(&mut self) -> Result<(), LiveReadError> {
        if self
            .currentLoop
            .replace(LiveLoopBuilder::default())
            .is_some()
        {
            return Err(LiveReadError::UnexpectedStructure("Loop"));
        }
        Ok(())
    }

    fn startKeyTrack(&mut self) -> Result<(), LiveReadError> {
        if self
            .currentKeyTrack
            .replace(KeyTrackBuilder::default())
            .is_some()
        {
            return Err(LiveReadError::UnexpectedStructure("KeyTrack"));
        }
        Ok(())
    }

    fn startClipSlot(&mut self, element: &BytesStart<'_>) -> Result<(), LiveReadError> {
        self.clipSlotDepth += 1;
        if self.clipSlotDepth == 1 {
            let element = LiveElement::new(element);
            self.currentSceneIndex = element
                .attribute(b"Id")?
                .map(|value| element.parseNumber("ClipSlot Id", value))
                .transpose()?;
        }
        Ok(())
    }

    fn finishClipSlot(&mut self) {
        self.clipSlotDepth -= 1;
        if self.clipSlotDepth == 0 {
            self.currentSceneIndex = None;
        }
    }

    fn startClip(&mut self, element: &BytesStart<'_>) -> Result<(), LiveReadError> {
        if self.currentClip.is_some() {
            return Err(LiveReadError::UnexpectedStructure("MidiClip"));
        }

        let id = LiveElement::new(element).requiredAttribute(b"Id", "MidiClip", "Id")?;
        let trackId = self
            .currentTrackId
            .clone()
            .ok_or(LiveReadError::MissingAttribute {
                element: "MidiTrack",
                attribute: "Id",
            })?;
        let sceneIndex = self
            .currentSceneIndex
            .ok_or(LiveReadError::MissingAttribute {
                element: "ClipSlot",
                attribute: "Id",
            })?;

        self.currentClip = Some(LiveMidiClipBuilder::new(id, trackId, sceneIndex));
        Ok(())
    }

    fn readClipElement(
        &mut self,
        name: &[u8],
        element: &BytesStart<'_>,
        parent: Option<&[u8]>,
    ) -> Result<(), LiveReadError> {
        if self.currentClip.is_none() {
            return Ok(());
        }
        let element = LiveElement::new(element);

        match (parent, name) {
            (Some(b"MidiClip"), b"CurrentStart") => {
                let start = element.requiredNumberValue("CurrentStart")?;
                self.currentClipMut()?.setCurrentStart(start);
            }
            (Some(b"MidiClip"), b"CurrentEnd") => {
                let end = element.requiredNumberValue("CurrentEnd")?;
                self.currentClipMut()?.setCurrentEnd(end);
            }
            (Some(b"MidiClip"), b"Name") => {
                let name = element.attribute(b"Value")?.unwrap_or_default();
                self.currentClipMut()?.setName(name);
            }
            (Some(b"MidiClip"), b"Disabled") => {
                let disabled = element.requiredBooleanValue("Disabled")?;
                self.currentClipMut()?.setDisabled(disabled);
            }
            (Some(b"Loop"), b"LoopStart") => {
                let start = element.requiredNumberValue("LoopStart")?;
                self.currentLoopMut()?.setStart(start);
            }
            (Some(b"Loop"), b"LoopEnd") => {
                let end = element.requiredNumberValue("LoopEnd")?;
                self.currentLoopMut()?.setEnd(end);
            }
            (Some(b"Loop"), b"StartRelative") => {
                let startRelative = element.requiredNumberValue("StartRelative")?;
                self.currentLoopMut()?.setStartRelative(startRelative);
            }
            (Some(b"Loop"), b"LoopOn") => {
                let enabled = element.requiredBooleanValue("LoopOn")?;
                self.currentLoopMut()?.setEnabled(enabled);
            }
            (_, b"MidiNoteEvent") if self.currentKeyTrack.is_some() => {
                let note = self.parseNote(&element)?;
                self.currentKeyTrackMut()?.addNote(note);
            }
            (Some(b"KeyTrack"), b"MidiKey") if self.currentKeyTrack.is_some() => {
                let pitch = element.requiredNumberValue("MidiKey")?;
                self.currentKeyTrackMut()?.setPitch(pitch);
            }
            (_, b"AutomationEnvelope") => self.currentClipMut()?.markClipAutomation(),
            (_, b"PerNoteEvent") => self.currentClipMut()?.markPerNoteExpression(),
            _ => {}
        }

        Ok(())
    }

    fn parseNote(&self, element: &LiveElement<'_, '_>) -> Result<PendingMidiNote, LiveReadError> {
        let mut note = PendingMidiNote::new(
            element.attribute(b"NoteId")?,
            element.requiredNumberAttribute(b"Time", "MidiNoteEvent Time")?,
            element.requiredNumberAttribute(b"Duration", "MidiNoteEvent Duration")?,
            element.requiredNumberAttribute(b"Velocity", "MidiNoteEvent Velocity")?,
        );
        note.setReleaseVelocity(
            element.optionalNumberAttribute(b"OffVelocity", "MidiNoteEvent OffVelocity")?,
        );
        note.setVelocityDeviation(
            element
                .optionalNumberAttribute(b"VelocityDeviation", "MidiNoteEvent VelocityDeviation")?,
        );
        note.setProbability(
            element.optionalNumberAttribute(b"Probability", "MidiNoteEvent Probability")?,
        );
        note.setEnabled(element.optionalBooleanAttribute(b"IsEnabled", "MidiNoteEvent IsEnabled")?);
        Ok(note)
    }

    fn currentClipMut(&mut self) -> Result<&mut LiveMidiClipBuilder, LiveReadError> {
        self.currentClip
            .as_mut()
            .ok_or(LiveReadError::UnexpectedStructure("MidiClip value"))
    }

    fn currentLoopMut(&mut self) -> Result<&mut LiveLoopBuilder, LiveReadError> {
        self.currentLoop
            .as_mut()
            .ok_or(LiveReadError::UnexpectedStructure("Loop value"))
    }

    fn currentKeyTrackMut(&mut self) -> Result<&mut KeyTrackBuilder, LiveReadError> {
        self.currentKeyTrack
            .as_mut()
            .ok_or(LiveReadError::UnexpectedStructure("KeyTrack value"))
    }

    fn finishLoop(&mut self) -> Result<(), LiveReadError> {
        let builder = self
            .currentLoop
            .take()
            .ok_or(LiveReadError::UnexpectedStructure("Loop"))?;
        let loopSettings = builder.finish()?;
        self.currentClipMut()?.setLoopSettings(loopSettings);
        Ok(())
    }

    fn finishKeyTrack(&mut self) -> Result<(), LiveReadError> {
        let builder = self
            .currentKeyTrack
            .take()
            .ok_or(LiveReadError::UnexpectedStructure("KeyTrack"))?;
        let notes = builder.finish()?;
        self.currentClipMut()?.addNotes(notes);
        Ok(())
    }

    fn finishClip(&mut self) -> Result<(), LiveReadError> {
        if self.currentLoop.is_some() {
            self.finishLoop()?;
        }
        if self.currentKeyTrack.is_some() {
            self.finishKeyTrack()?;
        }

        let clip = self
            .currentClip
            .take()
            .ok_or(LiveReadError::UnexpectedStructure("MidiClip"))?
            .finish()?;
        self.clips.push(clip);
        Ok(())
    }
}
