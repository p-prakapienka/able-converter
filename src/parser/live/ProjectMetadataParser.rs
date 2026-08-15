use quick_xml::events::BytesStart;

use crate::model::live::{LiveScene, LiveTrack, LiveTrackKind};

use super::liveelement::LiveElement;
use super::livereaderror::LiveReadError;
use super::livescenebuilder::LiveSceneBuilder;
use super::livetrackbuilder::LiveTrackBuilder;

#[derive(Default)]
pub(super) struct ProjectMetadataParser {
    currentTrack: Option<LiveTrackBuilder>,
    readingTrackName: bool,
    currentScene: Option<LiveSceneBuilder>,
    tracks: Vec<LiveTrack>,
    scenes: Vec<LiveScene>,
}

impl ProjectMetadataParser {
    pub(super) fn onStart(
        &mut self,
        name: &[u8],
        element: &BytesStart<'_>,
        parent: Option<&[u8]>,
    ) -> Result<(), LiveReadError> {
        if let Some(kind) = self.trackKind(name) {
            self.startTrack(name, kind, element)?;
        } else if name == b"Scene" && parent == Some(b"Scenes") {
            self.startScene(element)?;
        } else if name == b"Name" && self.isDirectTrackChild(parent) {
            self.readingTrackName = true;
        } else {
            self.readElement(name, element, parent)?;
        }

        Ok(())
    }

    pub(super) fn onEmpty(
        &mut self,
        name: &[u8],
        element: &BytesStart<'_>,
        parent: Option<&[u8]>,
    ) -> Result<(), LiveReadError> {
        if let Some(kind) = self.trackKind(name) {
            self.startTrack(name, kind, element)?;
            self.finishTrack()?;
        } else if name == b"Scene" && parent == Some(b"Scenes") {
            self.startScene(element)?;
            self.finishScene()?;
        } else {
            self.readElement(name, element, parent)?;
        }

        Ok(())
    }

    pub(super) fn onEnd(&mut self, name: &[u8]) -> Result<(), LiveReadError> {
        if name == b"Name" && self.readingTrackName {
            self.readingTrackName = false;
        } else if self
            .currentTrack
            .as_ref()
            .is_some_and(|track| track.matchesElement(name))
        {
            self.finishTrack()?;
        } else if name == b"Scene" && self.currentScene.is_some() {
            self.finishScene()?;
        }

        Ok(())
    }

    pub(super) fn finish(self) -> Result<(Vec<LiveTrack>, Vec<LiveScene>), LiveReadError> {
        if self.currentTrack.is_some() {
            return Err(LiveReadError::UnexpectedStructure("unclosed track"));
        }
        if self.currentScene.is_some() {
            return Err(LiveReadError::UnexpectedStructure("unclosed Scene"));
        }
        Ok((self.tracks, self.scenes))
    }

    fn trackKind(&self, name: &[u8]) -> Option<LiveTrackKind> {
        match name {
            b"MidiTrack" => Some(LiveTrackKind::Midi),
            b"AudioTrack" => Some(LiveTrackKind::Audio),
            b"GroupTrack" => Some(LiveTrackKind::Group),
            b"ReturnTrack" => Some(LiveTrackKind::Return),
            b"MasterTrack" | b"MainTrack" => Some(LiveTrackKind::Main),
            _ => None,
        }
    }

    fn startTrack(
        &mut self,
        elementName: &[u8],
        kind: LiveTrackKind,
        element: &BytesStart<'_>,
    ) -> Result<(), LiveReadError> {
        if self.currentTrack.is_some() {
            return Err(LiveReadError::UnexpectedStructure("track"));
        }

        let id = LiveElement::new(element).requiredAttribute(b"Id", "track", "Id")?;
        self.currentTrack = Some(LiveTrackBuilder::new(elementName, id, kind));
        Ok(())
    }

    fn startScene(&mut self, element: &BytesStart<'_>) -> Result<(), LiveReadError> {
        if self.currentScene.is_some() {
            return Err(LiveReadError::UnexpectedStructure("Scene"));
        }

        let id = LiveElement::new(element).requiredAttribute(b"Id", "Scene", "Id")?;
        self.currentScene = Some(LiveSceneBuilder::new(id, self.scenes.len()));
        Ok(())
    }

    fn readElement(
        &mut self,
        name: &[u8],
        element: &BytesStart<'_>,
        parent: Option<&[u8]>,
    ) -> Result<(), LiveReadError> {
        let element = LiveElement::new(element);

        if self.readingTrackName {
            match name {
                b"EffectiveName" => {
                    let effectiveName = element.attribute(b"Value")?.unwrap_or_default();
                    self.currentTrackMut()?.setEffectiveName(effectiveName);
                }
                b"UserName" => {
                    let userName = element.attribute(b"Value")?.unwrap_or_default();
                    self.currentTrackMut()?.setUserName(userName);
                }
                _ => {}
            }
        } else if name == b"Color" && self.isDirectTrackChild(parent) {
            let color = element.optionalNumberValue("track Color")?;
            self.currentTrackMut()?.setColor(color);
        }

        if parent == Some(b"Scene") && self.currentScene.is_some() {
            match name {
                b"Name" => {
                    let name = element.attribute(b"Value")?.unwrap_or_default();
                    self.currentSceneMut()?.setName(name);
                }
                b"Color" | b"ColorIndex" => {
                    let color = element.optionalNumberValue("Scene Color")?;
                    self.currentSceneMut()?.setColor(color);
                }
                b"Tempo" => {
                    let tempo = element.optionalNumberValue("Scene Tempo")?;
                    self.currentSceneMut()?.setTempo(tempo);
                }
                b"IsTempoEnabled" | b"TempoEnabled" => {
                    let enabled = element.requiredBooleanValue("Scene TempoEnabled")?;
                    self.currentSceneMut()?.setTempoEnabled(enabled);
                }
                b"TimeSignatureId" => {
                    let id = element.optionalNumberValue("Scene TimeSignatureId")?;
                    self.currentSceneMut()?.setTimeSignatureId(id);
                }
                b"IsTimeSignatureEnabled" | b"TimeSignatureEnabled" => {
                    let enabled = element.requiredBooleanValue("Scene TimeSignatureEnabled")?;
                    self.currentSceneMut()?.setTimeSignatureEnabled(enabled);
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn isDirectTrackChild(&self, parent: Option<&[u8]>) -> bool {
        self.currentTrack
            .as_ref()
            .is_some_and(|track| track.isDirectChild(parent))
    }

    fn currentTrackMut(&mut self) -> Result<&mut LiveTrackBuilder, LiveReadError> {
        self.currentTrack
            .as_mut()
            .ok_or(LiveReadError::UnexpectedStructure("track metadata"))
    }

    fn currentSceneMut(&mut self) -> Result<&mut LiveSceneBuilder, LiveReadError> {
        self.currentScene
            .as_mut()
            .ok_or(LiveReadError::UnexpectedStructure("Scene metadata"))
    }

    fn finishTrack(&mut self) -> Result<(), LiveReadError> {
        self.readingTrackName = false;
        let track = self
            .currentTrack
            .take()
            .ok_or(LiveReadError::UnexpectedStructure("track"))?
            .finish();
        self.tracks.push(track);
        Ok(())
    }

    fn finishScene(&mut self) -> Result<(), LiveReadError> {
        let scene = self
            .currentScene
            .take()
            .ok_or(LiveReadError::UnexpectedStructure("Scene"))?
            .finish();
        self.scenes.push(scene);
        Ok(())
    }
}
