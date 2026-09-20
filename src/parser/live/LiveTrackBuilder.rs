use crate::model::live::{LiveTrack, LiveTrackKind};

pub(super) struct LiveTrackBuilder {
    elementName: Vec<u8>,
    id: String,
    kind: LiveTrackKind,
    effectiveName: String,
    userName: String,
    colour: Option<i32>,
}

impl LiveTrackBuilder {
    pub(super) fn new(elementName: &[u8], id: String, kind: LiveTrackKind) -> Self {
        Self {
            elementName: elementName.to_vec(),
            id,
            kind,
            effectiveName: String::new(),
            userName: String::new(),
            colour: None,
        }
    }

    pub(super) fn matchesElement(&self, elementName: &[u8]) -> bool {
        self.elementName.as_slice() == elementName
    }

    pub(super) fn isDirectChild(&self, parent: Option<&[u8]>) -> bool {
        parent == Some(self.elementName.as_slice())
    }

    pub(super) fn setEffectiveName(&mut self, effectiveName: String) {
        self.effectiveName = effectiveName;
    }

    pub(super) fn setUserName(&mut self, userName: String) {
        self.userName = userName;
    }

    pub(super) fn setColour(&mut self, colour: Option<i32>) {
        self.colour = colour;
    }

    pub(super) fn finish(self) -> LiveTrack {
        LiveTrack {
            id: self.id,
            kind: self.kind,
            effectiveName: self.effectiveName,
            userName: self.userName,
            colour: self.colour,
        }
    }
}
