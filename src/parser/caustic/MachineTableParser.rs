use crate::model::caustic::{
    CausticMachine, CausticMachineSlot, EMPTY_MACHINE_ID, MACHINE_SLOT_COUNT,
};

use super::bytecursor::ByteCursor;
use super::causticreaderror::CausticReadError;
use super::controlcollectionparser::{CONTROL_COLLECTION_TAG, ControlCollectionParser};

/// Bytes holding a machine identifier in the slot table.
const MACHINE_ID_LENGTH: usize = 4;

/// Bytes following each slot identifier.
const SLOT_PADDING_LENGTH: usize = 1;

/// Bytes holding a machine display name.
const MACHINE_NAME_LENGTH: usize = 10;

/// Bytes written between a machine name and its payload length.
const MACHINE_HEADER_LENGTH: usize = 4;

/// Bytes of the version word some machine blocks write before their controls.
const BLOCK_VERSION_LENGTH: usize = 4;

/// Reads the fixed slot table and the machine payloads that follow it.
#[derive(Default)]
pub(super) struct MachineTableParser {
    controlCollectionParser: ControlCollectionParser,
}

impl MachineTableParser {
    pub(super) fn new() -> Self {
        Self {
            controlCollectionParser: ControlCollectionParser::new(),
        }
    }

    pub(super) fn parse(
        &self,
        cursor: &mut ByteCursor<'_>,
    ) -> Result<(Vec<CausticMachineSlot>, Vec<CausticMachine>), CausticReadError> {
        let slots = self.parseSlotTable(cursor)?;
        let mut machines = Vec::new();

        for slot in slots.iter().filter(|slot| slot.occupied) {
            machines.push(self.parseMachine(cursor, slot)?);
        }

        Ok((slots, machines))
    }

    fn parseSlotTable(
        &self,
        cursor: &mut ByteCursor<'_>,
    ) -> Result<Vec<CausticMachineSlot>, CausticReadError> {
        let mut slots = Vec::with_capacity(MACHINE_SLOT_COUNT);

        for slot in 0..MACHINE_SLOT_COUNT {
            let machineId =
                cursor.readFixedString("machine slot", "identifier", MACHINE_ID_LENGTH)?;
            cursor.readBytes("machine slot", SLOT_PADDING_LENGTH)?;

            let occupied = machineId != EMPTY_MACHINE_ID;
            if occupied && machineId.len() != MACHINE_ID_LENGTH {
                return Err(CausticReadError::InvalidMachineId { slot, machineId });
            }

            slots.push(CausticMachineSlot {
                slot,
                machineId,
                occupied,
            });
        }

        Ok(slots)
    }

    fn parseMachine(
        &self,
        cursor: &mut ByteCursor<'_>,
        slot: &CausticMachineSlot,
    ) -> Result<CausticMachine, CausticReadError> {
        let name = cursor.readFixedString("machine entry", "name", MACHINE_NAME_LENGTH)?;
        let headerBytes = cursor.readVec("machine entry", MACHINE_HEADER_LENGTH)?;
        let length = cursor.readLength("machine entry")?;

        let mut payload = ByteCursor::new(cursor.readBytes("machine payload", length)?);
        let blockVersion = self.parseBlockVersion(&mut payload)?;
        let controls = self.controlCollectionParser.parseOptional(&mut payload)?;
        let remaining = payload.remaining();
        let body = payload.readVec("machine payload", remaining)?;

        Ok(CausticMachine {
            slot: slot.slot,
            machineId: slot.machineId.clone(),
            name,
            headerBytes,
            blockVersion,
            controls,
            body,
        })
    }

    /// Read the block version that precedes the control collection in most
    /// machine payloads.
    ///
    /// `SubSynth::Serialize` writes a `u32` version (`0x03010000`) between its
    /// block length and `UIControlCollection::Serialize`, while
    /// `PadSynth::Serialize` and `Organ::Serialize` go straight to the
    /// collection. The word is only taken as a version when a `CCOL` tag
    /// follows it, so payloads with any other opening are left untouched.
    fn parseBlockVersion(
        &self,
        payload: &mut ByteCursor<'_>,
    ) -> Result<Option<u32>, CausticReadError> {
        if payload.peekTag().as_ref() == Some(CONTROL_COLLECTION_TAG) {
            return Ok(None);
        }

        let afterVersion = payload.position() + BLOCK_VERSION_LENGTH;
        if payload.peekTagAt(afterVersion).as_ref() != Some(CONTROL_COLLECTION_TAG) {
            return Ok(None);
        }

        Ok(Some(payload.readU32("machine block version")?))
    }
}
