use crate::model::live::LiveMidiNote;

pub(super) struct PendingMidiNote {
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
    pub(super) fn new(id: Option<String>, time: f64, duration: f64, velocity: f32) -> Self {
        Self {
            id,
            time,
            duration,
            velocity,
            releaseVelocity: None,
            velocityDeviation: None,
            probability: None,
            enabled: None,
        }
    }

    pub(super) fn setReleaseVelocity(&mut self, releaseVelocity: Option<f32>) {
        self.releaseVelocity = releaseVelocity;
    }

    pub(super) fn setVelocityDeviation(&mut self, velocityDeviation: Option<f32>) {
        self.velocityDeviation = velocityDeviation;
    }

    pub(super) fn setProbability(&mut self, probability: Option<f32>) {
        self.probability = probability;
    }

    pub(super) fn setEnabled(&mut self, enabled: Option<bool>) {
        self.enabled = enabled;
    }

    pub(super) fn withPitch(self, pitch: u16) -> LiveMidiNote {
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
