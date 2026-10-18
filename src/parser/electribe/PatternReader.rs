use crate::model::electribe::{
    ELECTRIBE_PART_COUNT, ElectribeBeat, ElectribeMasterEffect, ElectribePattern,
};

use super::byteblock::ByteBlock;
use super::electribereaderror::ElectribeReadError;
use super::motionslotreader::MotionSlotReader;
use super::partreader::{PART_SIZE, PartReader};

pub(super) const PATTERN_SIZE: usize = 0x4000;
const PART_BASE: usize = 0x800;
const END_MARKER_OFFSET: usize = 0x3BFC;

/// Decodes one 0x4000-byte pattern block.
pub(super) struct PatternReader {
    partReader: PartReader,
    motionSlotReader: MotionSlotReader,
}

impl PatternReader {
    pub(super) fn new() -> Self {
        Self {
            partReader: PartReader::new(),
            motionSlotReader: MotionSlotReader::new(),
        }
    }

    pub(super) fn read(
        &self,
        bytes: &[u8],
        number: usize,
    ) -> Result<ElectribePattern, ElectribeReadError> {
        let block = ByteBlock::new(bytes);
        if block.slice(0, 4) != b"PTST" || block.slice(END_MARKER_OFFSET, 4) != b"PTED" {
            return Err(ElectribeReadError::InvalidPatternMarkers { number });
        }

        let length = block.byte(0x25);
        if length > 3 {
            return Err(self.invalidValue(number, "length", u32::from(length)));
        }

        let parts = (0..ELECTRIBE_PART_COUNT)
            .map(|index| {
                self.partReader.read(
                    &block.sub(PART_BASE + index * PART_SIZE, PART_SIZE),
                    index + 1,
                )
            })
            .collect();

        Ok(ElectribePattern {
            number,
            name: block.text(0x10, 18),
            tempoTenths: block.word(0x22),
            swing: block.signedByte(0x24),
            lengthBars: length + 1,
            beat: self.readBeat(&block, number)?,
            key: block.byte(0x27),
            scale: block.byte(0x28),
            chordSet: block.byte(0x29),
            levelRaw: block.byte(0x2A),
            gateArp: block.byte(0x31),
            masterEffect: ElectribeMasterEffect {
                on: block.flag(0x3C),
                effectType: block.byte(0x3D),
                padX: block.byte(0x3E),
                padY: block.byte(0x3F),
                hold: block.flag(0x41),
                unknown40: block.byte(0x40),
            },
            alternate1314: block.flag(0x44),
            alternate1516: block.flag(0x45),
            chainTo: block.word(0x3B00),
            chainRepeat: block.byte(0x3B02),
            parts,
            motionSlots: self.motionSlotReader.read(&block),
        })
    }

    fn readBeat(
        &self,
        block: &ByteBlock<'_>,
        number: usize,
    ) -> Result<ElectribeBeat, ElectribeReadError> {
        match block.byte(0x26) {
            0 => Ok(ElectribeBeat::Sixteenth),
            1 => Ok(ElectribeBeat::ThirtySecond),
            2 => Ok(ElectribeBeat::EighthTriplet),
            3 => Ok(ElectribeBeat::SixteenthTriplet),
            other => Err(self.invalidValue(number, "beat", u32::from(other))),
        }
    }

    fn invalidValue(&self, number: usize, field: &'static str, value: u32) -> ElectribeReadError {
        ElectribeReadError::InvalidPatternValue {
            number,
            field,
            value,
        }
    }
}
