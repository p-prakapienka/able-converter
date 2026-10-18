use crate::model::electribe::{ELECTRIBE_MOTION_SLOT_COUNT, ElectribeMotionSlot};

use super::byteblock::ByteBlock;

const MOTION_PART_BASE: usize = 0x100;
const MOTION_PARAMETER_BASE: usize = 0x118;
const MOTION_VALUE_BASE: usize = 0x130;
const MOTION_VALUE_COUNT: usize = 64;

/// Decodes the 24 motion-sequence slots stored in a pattern.
pub(super) struct MotionSlotReader;

impl MotionSlotReader {
    pub(super) fn new() -> Self {
        Self
    }

    /// Returns only occupied slots; a part number of zero marks an empty slot.
    pub(super) fn read(&self, pattern: &ByteBlock<'_>) -> Vec<ElectribeMotionSlot> {
        (0..ELECTRIBE_MOTION_SLOT_COUNT)
            .filter_map(|slot| self.readSlot(pattern, slot))
            .collect()
    }

    fn readSlot(&self, pattern: &ByteBlock<'_>, slot: usize) -> Option<ElectribeMotionSlot> {
        let part = pattern.byte(MOTION_PART_BASE + slot);
        if part == 0 {
            return None;
        }

        Some(ElectribeMotionSlot {
            slot,
            part,
            parameterId: pattern.byte(MOTION_PARAMETER_BASE + slot),
            values: pattern
                .slice(
                    MOTION_VALUE_BASE + slot * MOTION_VALUE_COUNT,
                    MOTION_VALUE_COUNT,
                )
                .to_vec(),
        })
    }
}
