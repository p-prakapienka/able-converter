use crate::model::electribe::ElectribeBeat;

/// Number of stored step slots per bar, regardless of the beat resolution.
const STEP_SLOTS_PER_BAR: usize = 16;
/// Gate value representing a full step; larger values tie into the next step.
const FULL_GATE: u8 = 96;

/// Playback order and timing of one part's steps.
///
/// Every bar occupies sixteen storage slots. A bar plays its first `stepsPerBar`
/// slots, where the default is sixteen (straight beats) or twelve (triplet beats)
/// and a non-zero part LAST STEP replaces the default. Positions are expressed in
/// quarter-note beats.
pub(super) struct StepTimeline {
    stepsPerBar: usize,
    defaultStepsPerBar: usize,
    stepsPerQuarter: f64,
    bars: usize,
}

impl StepTimeline {
    pub(super) fn new(beat: ElectribeBeat, bars: u8, lastStep: u8) -> Self {
        let (stepsPerQuarter, defaultStepsPerBar) = match beat {
            ElectribeBeat::Sixteenth => (4.0, 16),
            ElectribeBeat::ThirtySecond => (8.0, 16),
            ElectribeBeat::EighthTriplet => (3.0, 12),
            ElectribeBeat::SixteenthTriplet => (6.0, 12),
        };
        let stepsPerBar = if lastStep == 0 {
            defaultStepsPerBar
        } else {
            usize::from(lastStep).min(STEP_SLOTS_PER_BAR)
        };

        Self {
            stepsPerBar,
            defaultStepsPerBar,
            stepsPerQuarter,
            bars: usize::from(bars),
        }
    }

    /// Storage indices of the played steps, in playback order.
    pub(super) fn playedSteps(&self) -> Vec<usize> {
        (0..self.bars)
            .flat_map(|bar| (0..self.stepsPerBar).map(move |step| bar * STEP_SLOTS_PER_BAR + step))
            .collect()
    }

    pub(super) fn stepLength(&self) -> f64 {
        1.0 / self.stepsPerQuarter
    }

    pub(super) fn positionOf(&self, playbackPosition: usize) -> f64 {
        playbackPosition as f64 * self.stepLength()
    }

    /// Loop length of this part in beats.
    pub(super) fn length(&self) -> f64 {
        self.positionOf(self.bars * self.stepsPerBar)
    }

    /// Loop length of the whole pattern in beats, ignoring the part's LAST STEP.
    pub(super) fn patternLength(&self) -> f64 {
        (self.bars * self.defaultStepsPerBar) as f64 * self.stepLength()
    }

    pub(super) fn usesPartLastStep(&self) -> bool {
        self.stepsPerBar != self.defaultStepsPerBar
    }

    pub(super) fn isTie(&self, gate: u8) -> bool {
        gate > FULL_GATE
    }

    /// Audible length of a non-tied step with the given gate.
    pub(super) fn gateLength(&self, gate: u8) -> f64 {
        f64::from(gate.clamp(1, FULL_GATE)) / f64::from(FULL_GATE) * self.stepLength()
    }
}
