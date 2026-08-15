//! Streaming parsing of gzip-compressed Ableton Live Set (`.als`) files.

use std::io::{BufReader, Read};
use std::str::FromStr;

use flate2::read::GzDecoder;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use thiserror::Error;

use crate::model::live::{
    ClipCounts, LiveFormatVersion, LiveLoop, LiveMidiClip, LiveMidiNote, LiveProject,
    LiveSetInspection, TrackCounts,
};

#[derive(Debug, Error)]
pub enum LiveReadError {
    #[error("failed to read Ableton Live XML: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("an XML attribute could not be decoded as UTF-8: {0}")]
    InvalidAttribute(#[from] std::str::Utf8Error),
    #[error("an XML attribute contains an invalid escape sequence: {0}")]
    InvalidEscape(#[from] quick_xml::escape::EscapeError),
    #[error("invalid {element} value: {value}")]
    InvalidValue {
        element: &'static str,
        value: String,
    },
    #[error("{element} is missing required {attribute} attribute")]
    MissingAttribute {
        element: &'static str,
        attribute: &'static str,
    },
    #[error("{parent} is missing required {element} element")]
    MissingElement {
        parent: &'static str,
        element: &'static str,
    },
    #[error("unexpected nested {0} element")]
    UnexpectedStructure(&'static str),
    #[error("the file does not contain an Ableton root element")]
    MissingAbletonRoot,
}

/// Reads a gzip-compressed Ableton Live Set from the supplied stream.
pub struct LiveParser<R> {
    source: R,
}

impl<R: Read> LiveParser<R> {
    pub fn new(source: R) -> Self {
        Self { source }
    }

    /// Inspect the Set without materialising the decompressed XML on disk.
    pub fn inspect(self) -> Result<LiveSetInspection, LiveReadError> {
        self.inspectXml()
    }

    /// Parse supported Live Set content into a format-specific source model.
    ///
    /// This first extraction slice returns Session MIDI clips. Arrangement clips are
    /// counted by the inspection model but remain a separate future parsing path.
    pub fn parse(self) -> Result<LiveProject, LiveReadError> {
        self.parseXml()
    }

    fn inspectXml(self) -> Result<LiveSetInspection, LiveReadError> {
        let decoder = GzDecoder::new(BufReader::new(self.source));
        let mut reader = Reader::from_reader(BufReader::new(decoder));
        reader.config_mut().trim_text(true);

        let mut buffer = Vec::new();
        let mut inspection = InspectionBuilder::default();

        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(element) => {
                    inspection.onElement(&element)?;
                    inspection.onStart(element.name().as_ref());
                }
                Event::Empty(element) => inspection.onElement(&element)?,
                Event::End(element) => inspection.onEnd(element.name().as_ref()),
                Event::Eof => break,
                _ => {}
            }

            buffer.clear();
        }

        inspection.finish()
    }

    fn parseXml(self) -> Result<LiveProject, LiveReadError> {
        let decoder = GzDecoder::new(BufReader::new(self.source));
        let mut reader = Reader::from_reader(BufReader::new(decoder));
        reader.config_mut().trim_text(true);

        let mut buffer = Vec::new();
        let mut inspection = InspectionBuilder::default();
        let mut parser = SessionParser::default();
        let mut ancestors: Vec<Vec<u8>> = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(element) => {
                    inspection.onElement(&element)?;
                    parser.onStart(
                        element.name().as_ref(),
                        &element,
                        ancestors.last().map(Vec::as_slice),
                        inspection.sessionDepth,
                    )?;
                    inspection.onStart(element.name().as_ref());
                    ancestors.push(element.name().as_ref().to_vec());
                }
                Event::Empty(element) => {
                    inspection.onElement(&element)?;
                    parser.onEmpty(
                        element.name().as_ref(),
                        &element,
                        ancestors.last().map(Vec::as_slice),
                        inspection.sessionDepth,
                    )?;
                }
                Event::End(element) => {
                    parser.onEnd(element.name().as_ref())?;
                    inspection.onEnd(element.name().as_ref());
                    ancestors.pop();
                }
                Event::Eof => break,
                _ => {}
            }

            buffer.clear();
        }

        Ok(LiveProject {
            inspection: inspection.finish()?,
            sessionMidiClips: parser.finish()?,
        })
    }
}

#[derive(Default)]
struct InspectionBuilder {
    format: Option<LiveFormatVersion>,
    tempo: Option<f64>,
    tracks: TrackCounts,
    clips: ClipCounts,
    sessionDepth: usize,
    arrangementDepth: usize,
    tempoDepth: usize,
}

impl InspectionBuilder {
    fn onElement(&mut self, element: &BytesStart<'_>) -> Result<(), LiveReadError> {
        let name = element.name();
        let name = name.as_ref();
        let element = LiveElement::new(element);

        match name {
            b"Ableton" if self.format.is_none() => {
                self.format = Some(LiveFormatVersion {
                    major: element.attribute(b"MajorVersion")?,
                    minor: element.attribute(b"MinorVersion")?,
                    creator: element.attribute(b"Creator")?,
                    revision: element.attribute(b"Revision")?,
                });
            }
            b"MidiTrack" => self.tracks.midi += 1,
            b"AudioTrack" => self.tracks.audio += 1,
            b"GroupTrack" => self.tracks.group += 1,
            b"ReturnTrack" => self.tracks.returnTracks += 1,
            b"MasterTrack" | b"MainTrack" => self.tracks.main += 1,
            b"MidiClip" => self.countMidiClip(),
            b"AudioClip" => self.countAudioClip(),
            b"Manual" if self.tempoDepth > 0 && self.tempo.is_none() => {
                self.tempo = element.optionalNumberValue("Manual")?;
            }
            _ => {}
        }

        Ok(())
    }

    fn countMidiClip(&mut self) {
        if self.sessionDepth > 0 {
            self.clips.sessionMidi += 1;
        } else if self.arrangementDepth > 0 {
            self.clips.arrangementMidi += 1;
        } else {
            self.clips.unclassifiedMidi += 1;
        }
    }

    fn countAudioClip(&mut self) {
        if self.sessionDepth > 0 {
            self.clips.sessionAudio += 1;
        } else if self.arrangementDepth > 0 {
            self.clips.arrangementAudio += 1;
        } else {
            self.clips.unclassifiedAudio += 1;
        }
    }

    fn onStart(&mut self, name: &[u8]) {
        match name {
            b"ClipSlotList" => self.sessionDepth += 1,
            b"ArrangerAutomation" => self.arrangementDepth += 1,
            b"Tempo" => self.tempoDepth += 1,
            _ => {}
        }
    }

    fn onEnd(&mut self, name: &[u8]) {
        match name {
            b"ClipSlotList" => self.sessionDepth = self.sessionDepth.saturating_sub(1),
            b"ArrangerAutomation" => {
                self.arrangementDepth = self.arrangementDepth.saturating_sub(1);
            }
            b"Tempo" => self.tempoDepth = self.tempoDepth.saturating_sub(1),
            _ => {}
        }
    }

    fn finish(self) -> Result<LiveSetInspection, LiveReadError> {
        Ok(LiveSetInspection {
            format: self.format.ok_or(LiveReadError::MissingAbletonRoot)?,
            tempo: self.tempo,
            tracks: self.tracks,
            clips: self.clips,
        })
    }
}

#[derive(Default)]
struct SessionParser {
    currentTrackId: Option<String>,
    clipSlotDepth: usize,
    currentSceneIndex: Option<usize>,
    currentClip: Option<LiveMidiClipBuilder>,
    currentLoop: Option<LiveLoopBuilder>,
    currentKeyTrack: Option<KeyTrackBuilder>,
    clips: Vec<LiveMidiClip>,
}

impl SessionParser {
    fn onStart(
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
            b"Loop" if self.currentClip.is_some() => {
                if self
                    .currentLoop
                    .replace(LiveLoopBuilder::default())
                    .is_some()
                {
                    return Err(LiveReadError::UnexpectedStructure("Loop"));
                }
            }
            b"KeyTrack" if self.currentClip.is_some() => {
                if self
                    .currentKeyTrack
                    .replace(KeyTrackBuilder::default())
                    .is_some()
                {
                    return Err(LiveReadError::UnexpectedStructure("KeyTrack"));
                }
            }
            _ => self.readClipElement(name, element, parent)?,
        }

        Ok(())
    }

    fn onEmpty(
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
                self.currentLoop = Some(LiveLoopBuilder::default());
                self.finishLoop()?;
            }
            b"KeyTrack" if self.currentClip.is_some() => {
                self.currentKeyTrack = Some(KeyTrackBuilder::default());
                self.finishKeyTrack()?;
            }
            _ => self.readClipElement(name, element, parent)?,
        }

        Ok(())
    }

    fn onEnd(&mut self, name: &[u8]) -> Result<(), LiveReadError> {
        match name {
            b"MidiTrack" => self.currentTrackId = None,
            b"ClipSlot" if self.clipSlotDepth > 0 => {
                self.clipSlotDepth -= 1;
                if self.clipSlotDepth == 0 {
                    self.currentSceneIndex = None;
                }
            }
            b"MidiClip" if self.currentClip.is_some() => self.finishClip()?,
            b"Loop" if self.currentLoop.is_some() => self.finishLoop()?,
            b"KeyTrack" if self.currentKeyTrack.is_some() => self.finishKeyTrack()?,
            _ => {}
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

        self.currentClip = Some(LiveMidiClipBuilder {
            id,
            trackId,
            sceneIndex,
            ..LiveMidiClipBuilder::default()
        });
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
                self.currentClipMut().currentStart =
                    Some(element.requiredNumberValue("CurrentStart")?);
            }
            (Some(b"MidiClip"), b"CurrentEnd") => {
                self.currentClipMut().currentEnd = Some(element.requiredNumberValue("CurrentEnd")?);
            }
            (Some(b"MidiClip"), b"Name") => {
                self.currentClipMut().name = element.attribute(b"Value")?.unwrap_or_default();
            }
            (Some(b"MidiClip"), b"Disabled") => {
                self.currentClipMut().disabled = element.requiredBooleanValue("Disabled")?;
            }
            (Some(b"Loop"), b"LoopStart") => {
                self.currentLoopMut()?.start = Some(element.requiredNumberValue("LoopStart")?);
            }
            (Some(b"Loop"), b"LoopEnd") => {
                self.currentLoopMut()?.end = Some(element.requiredNumberValue("LoopEnd")?);
            }
            (Some(b"Loop"), b"StartRelative") => {
                self.currentLoopMut()?.startRelative =
                    Some(element.requiredNumberValue("StartRelative")?);
            }
            (Some(b"Loop"), b"LoopOn") => {
                self.currentLoopMut()?.enabled = Some(element.requiredBooleanValue("LoopOn")?);
            }
            (_, b"MidiNoteEvent") if self.currentKeyTrack.is_some() => {
                let note = self.parseNote(&element)?;
                self.currentKeyTrack
                    .as_mut()
                    .expect("guarded above")
                    .notes
                    .push(note);
            }
            (Some(b"KeyTrack"), b"MidiKey") if self.currentKeyTrack.is_some() => {
                self.currentKeyTrack.as_mut().expect("guarded above").pitch =
                    Some(element.requiredNumberValue("MidiKey")?);
            }
            (_, b"AutomationEnvelope") => self.currentClipMut().hasClipAutomation = true,
            (_, b"PerNoteEvent") => self.currentClipMut().hasPerNoteExpression = true,
            _ => {}
        }

        Ok(())
    }

    fn parseNote(&self, element: &LiveElement<'_, '_>) -> Result<PendingMidiNote, LiveReadError> {
        Ok(PendingMidiNote {
            id: element.attribute(b"NoteId")?,
            time: element.requiredNumberAttribute(b"Time", "MidiNoteEvent Time")?,
            duration: element.requiredNumberAttribute(b"Duration", "MidiNoteEvent Duration")?,
            velocity: element.requiredNumberAttribute(b"Velocity", "MidiNoteEvent Velocity")?,
            releaseVelocity: element
                .optionalNumberAttribute(b"OffVelocity", "MidiNoteEvent OffVelocity")?,
            velocityDeviation: element
                .optionalNumberAttribute(b"VelocityDeviation", "MidiNoteEvent VelocityDeviation")?,
            probability: element
                .optionalNumberAttribute(b"Probability", "MidiNoteEvent Probability")?,
            enabled: element.optionalBooleanAttribute(b"IsEnabled", "MidiNoteEvent IsEnabled")?,
        })
    }

    fn currentClipMut(&mut self) -> &mut LiveMidiClipBuilder {
        self.currentClip.as_mut().expect("checked by caller")
    }

    fn currentLoopMut(&mut self) -> Result<&mut LiveLoopBuilder, LiveReadError> {
        self.currentLoop
            .as_mut()
            .ok_or(LiveReadError::UnexpectedStructure("Loop value"))
    }

    fn finishLoop(&mut self) -> Result<(), LiveReadError> {
        let builder = self
            .currentLoop
            .take()
            .ok_or(LiveReadError::UnexpectedStructure("Loop"))?;
        let loopSettings = builder.finish()?;
        self.currentClip
            .as_mut()
            .ok_or(LiveReadError::UnexpectedStructure("Loop outside MidiClip"))?
            .loopSettings = Some(loopSettings);
        Ok(())
    }

    fn finishKeyTrack(&mut self) -> Result<(), LiveReadError> {
        let builder = self
            .currentKeyTrack
            .take()
            .ok_or(LiveReadError::UnexpectedStructure("KeyTrack"))?;
        let notes = builder.finish()?;
        self.currentClip
            .as_mut()
            .ok_or(LiveReadError::UnexpectedStructure(
                "KeyTrack outside MidiClip",
            ))?
            .notes
            .extend(notes);
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

    fn finish(self) -> Result<Vec<LiveMidiClip>, LiveReadError> {
        if self.currentClip.is_some() {
            return Err(LiveReadError::UnexpectedStructure("unclosed MidiClip"));
        }
        Ok(self.clips)
    }
}

#[derive(Default)]
struct LiveMidiClipBuilder {
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
    fn finish(self) -> Result<LiveMidiClip, LiveReadError> {
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

#[derive(Default)]
struct LiveLoopBuilder {
    start: Option<f64>,
    end: Option<f64>,
    startRelative: Option<f64>,
    enabled: Option<bool>,
}

impl LiveLoopBuilder {
    fn finish(self) -> Result<LiveLoop, LiveReadError> {
        Ok(LiveLoop {
            start: self.start.ok_or(LiveReadError::MissingElement {
                parent: "Loop",
                element: "LoopStart",
            })?,
            end: self.end.ok_or(LiveReadError::MissingElement {
                parent: "Loop",
                element: "LoopEnd",
            })?,
            startRelative: self.startRelative.ok_or(LiveReadError::MissingElement {
                parent: "Loop",
                element: "StartRelative",
            })?,
            enabled: self.enabled.ok_or(LiveReadError::MissingElement {
                parent: "Loop",
                element: "LoopOn",
            })?,
        })
    }
}

#[derive(Default)]
struct KeyTrackBuilder {
    pitch: Option<u16>,
    notes: Vec<PendingMidiNote>,
}

impl KeyTrackBuilder {
    fn finish(self) -> Result<Vec<LiveMidiNote>, LiveReadError> {
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

struct PendingMidiNote {
    id: Option<String>,
    time: f64,
    duration: f64,
    velocity: f32,
    releaseVelocity: Option<f32>,
    velocityDeviation: Option<f32>,
    probability: Option<f32>,
    enabled: Option<bool>,
}

impl PendingMidiNote {
    fn withPitch(self, pitch: u16) -> LiveMidiNote {
        LiveMidiNote {
            id: self.id,
            pitch,
            time: self.time,
            duration: self.duration,
            velocity: self.velocity,
            releaseVelocity: self.releaseVelocity,
            velocityDeviation: self.velocityDeviation,
            probability: self.probability,
            enabled: self.enabled,
        }
    }
}

struct LiveElement<'element, 'data> {
    element: &'element BytesStart<'data>,
}

impl<'element, 'data> LiveElement<'element, 'data> {
    fn new(element: &'element BytesStart<'data>) -> Self {
        Self { element }
    }

    fn attribute(&self, key: &[u8]) -> Result<Option<String>, LiveReadError> {
        for attribute in self.element.attributes().with_checks(false).flatten() {
            if attribute.key.as_ref() == key {
                let value = std::str::from_utf8(attribute.value.as_ref())?;
                return Ok(Some(quick_xml::escape::unescape(value)?.into_owned()));
            }
        }

        Ok(None)
    }

    fn requiredAttribute(
        &self,
        key: &[u8],
        elementName: &'static str,
        attributeName: &'static str,
    ) -> Result<String, LiveReadError> {
        self.attribute(key)?.ok_or(LiveReadError::MissingAttribute {
            element: elementName,
            attribute: attributeName,
        })
    }

    fn requiredNumberAttribute<T: FromStr>(
        &self,
        key: &[u8],
        elementName: &'static str,
    ) -> Result<T, LiveReadError> {
        let value = self.requiredAttribute(key, elementName, "numeric value")?;
        self.parseNumber(elementName, value)
    }

    fn optionalNumberAttribute<T: FromStr>(
        &self,
        key: &[u8],
        elementName: &'static str,
    ) -> Result<Option<T>, LiveReadError> {
        self.attribute(key)?
            .map(|value| self.parseNumber(elementName, value))
            .transpose()
    }

    fn requiredNumberValue<T: FromStr>(
        &self,
        elementName: &'static str,
    ) -> Result<T, LiveReadError> {
        self.requiredNumberAttribute(b"Value", elementName)
    }

    fn optionalNumberValue<T: FromStr>(
        &self,
        elementName: &'static str,
    ) -> Result<Option<T>, LiveReadError> {
        self.optionalNumberAttribute(b"Value", elementName)
    }

    fn parseNumber<T: FromStr>(
        &self,
        elementName: &'static str,
        value: String,
    ) -> Result<T, LiveReadError> {
        value.parse().map_err(|_| LiveReadError::InvalidValue {
            element: elementName,
            value,
        })
    }

    fn requiredBooleanValue(&self, elementName: &'static str) -> Result<bool, LiveReadError> {
        let value = self.requiredAttribute(b"Value", elementName, "Value")?;
        self.parseBoolean(elementName, value)
    }

    fn optionalBooleanAttribute(
        &self,
        key: &[u8],
        elementName: &'static str,
    ) -> Result<Option<bool>, LiveReadError> {
        self.attribute(key)?
            .map(|value| self.parseBoolean(elementName, value))
            .transpose()
    }

    fn parseBoolean(
        &self,
        elementName: &'static str,
        value: String,
    ) -> Result<bool, LiveReadError> {
        match value.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(LiveReadError::InvalidValue {
                element: elementName,
                value,
            }),
        }
    }
}
