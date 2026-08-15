use crate::model::live::LiveScene;

pub(super) struct LiveSceneBuilder {
    id: String,
    index: usize,
    name: String,
    color: Option<i32>,
    tempo: Option<f64>,
    tempoEnabled: bool,
    timeSignatureId: Option<i32>,
    timeSignatureEnabled: bool,
}

impl LiveSceneBuilder {
    pub(super) fn new(id: String, index: usize) -> Self {
        Self {
            id,
            index,
            name: String::new(),
            color: None,
            tempo: None,
            tempoEnabled: false,
            timeSignatureId: None,
            timeSignatureEnabled: false,
        }
    }

    pub(super) fn setName(&mut self, name: String) {
        self.name = name;
    }

    pub(super) fn setColor(&mut self, color: Option<i32>) {
        self.color = color;
    }

    pub(super) fn setTempo(&mut self, tempo: Option<f64>) {
        self.tempo = tempo;
    }

    pub(super) fn setTempoEnabled(&mut self, tempoEnabled: bool) {
        self.tempoEnabled = tempoEnabled;
    }

    pub(super) fn setTimeSignatureId(&mut self, timeSignatureId: Option<i32>) {
        self.timeSignatureId = timeSignatureId;
    }

    pub(super) fn setTimeSignatureEnabled(&mut self, timeSignatureEnabled: bool) {
        self.timeSignatureEnabled = timeSignatureEnabled;
    }

    pub(super) fn finish(self) -> LiveScene {
        LiveScene {
            id: self.id,
            index: self.index,
            name: self.name,
            color: self.color,
            tempo: self.tempo,
            tempoEnabled: self.tempoEnabled,
            timeSignatureId: self.timeSignatureId,
            timeSignatureEnabled: self.timeSignatureEnabled,
        }
    }
}
