use crate::model::caustic::{CausticControl, CausticControlCollection};

use super::bytecursor::ByteCursor;
use super::causticreaderror::CausticReadError;

/// Four-character identifier of a control collection.
pub(super) const CONTROL_COLLECTION_TAG: &[u8; 4] = b"CCOL";

/// Bytes occupied by one identifier and value pair.
const CONTROL_ENTRY_LENGTH: usize = 8;

/// Reads the `CCOL` chunk that opens most machine and effect payloads.
#[derive(Default)]
pub(super) struct ControlCollectionParser;

impl ControlCollectionParser {
    pub(super) fn new() -> Self {
        Self
    }

    /// Read a control collection when the cursor is positioned on one.
    ///
    /// Returns `None` without consuming input when the next four bytes are not
    /// the collection tag, because not every payload begins with one.
    pub(super) fn parseOptional(
        &self,
        cursor: &mut ByteCursor<'_>,
    ) -> Result<Option<CausticControlCollection>, CausticReadError> {
        if cursor.peekTag().as_ref() != Some(CONTROL_COLLECTION_TAG) {
            return Ok(None);
        }

        cursor.readTag("control collection")?;
        let length = cursor.readLength("control collection")?;
        let mut payload = ByteCursor::new(cursor.readBytes("control collection", length)?);

        let mut controls = Vec::with_capacity(length / CONTROL_ENTRY_LENGTH);
        while payload.remaining() >= CONTROL_ENTRY_LENGTH {
            controls.push(CausticControl {
                id: payload.readU32("control entry")?,
                value: payload.readF32("control entry")?,
            });
        }

        Ok(Some(CausticControlCollection { controls }))
    }
}
