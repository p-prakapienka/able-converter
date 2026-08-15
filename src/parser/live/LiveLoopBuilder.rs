use crate::model::live::LiveLoop;

use super::livereaderror::LiveReadError;

#[derive(Default)]
pub(super) struct LiveLoopBuilder {
    start: Option<f64>,
    end: Option<f64>,
    startRelative: Option<f64>,
    enabled: Option<bool>,
}

impl LiveLoopBuilder {
    pub(super) fn setStart(&mut self, start: f64) {
        self.start = Some(start);
    }

    pub(super) fn setEnd(&mut self, end: f64) {
        self.end = Some(end);
    }

    pub(super) fn setStartRelative(&mut self, startRelative: f64) {
        self.startRelative = Some(startRelative);
    }

    pub(super) fn setEnabled(&mut self, enabled: bool) {
        self.enabled = Some(enabled);
    }

    pub(super) fn finish(self) -> Result<LiveLoop, LiveReadError> {
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
