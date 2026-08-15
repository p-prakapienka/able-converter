use quick_xml::events::BytesStart;

use crate::model::live::{ClipCounts, LiveFormatVersion, LiveSetInspection, TrackCounts};

use super::liveelement::LiveElement;
use super::livereaderror::LiveReadError;

#[derive(Default)]
pub(super) struct InspectionBuilder {
    format: Option<LiveFormatVersion>,
    tempo: Option<f64>,
    tracks: TrackCounts,
    clips: ClipCounts,
    sessionDepth: usize,
    arrangementDepth: usize,
    tempoDepth: usize,
}

impl InspectionBuilder {
    pub(super) fn sessionDepth(&self) -> usize {
        self.sessionDepth
    }

    pub(super) fn onElement(&mut self, element: &BytesStart<'_>) -> Result<(), LiveReadError> {
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

    pub(super) fn onStart(&mut self, name: &[u8]) {
        match name {
            b"ClipSlotList" => self.sessionDepth += 1,
            b"ArrangerAutomation" => self.arrangementDepth += 1,
            b"Tempo" => self.tempoDepth += 1,
            _ => {}
        }
    }

    pub(super) fn onEnd(&mut self, name: &[u8]) {
        match name {
            b"ClipSlotList" => self.sessionDepth = self.sessionDepth.saturating_sub(1),
            b"ArrangerAutomation" => {
                self.arrangementDepth = self.arrangementDepth.saturating_sub(1);
            }
            b"Tempo" => self.tempoDepth = self.tempoDepth.saturating_sub(1),
            _ => {}
        }
    }

    pub(super) fn finish(self) -> Result<LiveSetInspection, LiveReadError> {
        Ok(LiveSetInspection {
            format: self.format.ok_or(LiveReadError::MissingAbletonRoot)?,
            tempo: self.tempo,
            tracks: self.tracks,
            clips: self.clips,
        })
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
}
