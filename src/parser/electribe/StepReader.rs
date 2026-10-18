use crate::model::electribe::ElectribeStep;

use super::byteblock::ByteBlock;

pub(super) const STEP_SIZE: usize = 12;

/// Decodes one 12-byte sequencer step.
pub(super) struct StepReader;

impl StepReader {
    pub(super) fn new() -> Self {
        Self
    }

    pub(super) fn read(&self, block: &ByteBlock<'_>, index: usize) -> ElectribeStep {
        let mut noteSlots = [0_u8; 8];
        noteSlots.copy_from_slice(block.slice(4, 8));

        ElectribeStep {
            index,
            triggered: block.flag(0),
            gate: block.byte(1),
            velocity: block.byte(2),
            unknownFlag: block.byte(3),
            noteSlots,
        }
    }
}
