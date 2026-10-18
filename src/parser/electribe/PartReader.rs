use crate::model::electribe::{ELECTRIBE_STEP_COUNT, ElectribePart};

use super::byteblock::ByteBlock;
use super::stepreader::{STEP_SIZE, StepReader};

pub(super) const PART_SIZE: usize = 0x330;
const STEP_BASE: usize = 0x30;

/// Decodes one 0x330-byte part: a 0x30-byte header followed by 64 steps.
pub(super) struct PartReader {
    stepReader: StepReader,
}

impl PartReader {
    pub(super) fn new() -> Self {
        Self {
            stepReader: StepReader::new(),
        }
    }

    pub(super) fn read(&self, block: &ByteBlock<'_>, number: usize) -> ElectribePart {
        let steps = (0..ELECTRIBE_STEP_COUNT)
            .map(|index| {
                self.stepReader
                    .read(&block.sub(STEP_BASE + index * STEP_SIZE, STEP_SIZE), index)
            })
            .collect();

        ElectribePart {
            number,
            lastStep: block.byte(0x00),
            mute: block.flag(0x01),
            voiceAssign: block.byte(0x02),
            motionSequence: block.byte(0x03),
            triggerPadVelocity: block.flag(0x04),
            scaleMode: block.flag(0x05),
            partPriority: block.byte(0x06),
            oscillator: block.word(0x08),
            unknown0A: block.byte(0x0A),
            oscillatorEdit: block.byte(0x0B),
            filterType: block.byte(0x0C),
            cutoff: block.byte(0x0D),
            resonance: block.byte(0x0E),
            egIntensity: block.signedByte(0x0F),
            modulationType: block.byte(0x10),
            modulationSpeed: block.byte(0x11),
            modulationDepth: block.byte(0x12),
            attack: block.byte(0x14),
            decayRelease: block.byte(0x15),
            level: block.byte(0x18),
            pan: block.signedByte(0x19),
            ampEg: block.flag(0x1A),
            mfxSend: block.flag(0x1B),
            grooveType: block.byte(0x1C),
            grooveDepth: block.byte(0x1D),
            ifxOn: block.flag(0x20),
            ifxType: block.byte(0x21),
            ifxEdit: block.byte(0x22),
            pitch: block.signedByte(0x24),
            glide: block.byte(0x25),
            unknown26: block.byte(0x26),
            steps,
        }
    }
}
