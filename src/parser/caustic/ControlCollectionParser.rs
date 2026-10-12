use crate::model::caustic::{CausticControl, CausticControlCollection};

use super::bytecursor::ByteCursor;
use super::causticreaderror::CausticReadError;

/// Four-character identifier of a control collection.
pub(super) const CONTROL_COLLECTION_TAG: &[u8; 4] = b"CCOL";

/// Four-character identifier of one control inside a collection.
const CONTROL_TAG: &[u8; 4] = b"CTRL";

/// Payload length of a control chunk: a `u32` identifier and an `f32` value.
const CONTROL_PAYLOAD_LENGTH: usize = 8;

/// Bytes occupied by one control chunk, tag and length included.
const CONTROL_CHUNK_LENGTH: usize = 16;

/// Reads the `CCOL` chunk that opens the control section of machine and effect
/// payloads.
///
/// `UIControlCollection::Serialize` writes each control as its own `CTRL`
/// chunk: the tag, a `u32` length of 8, then the identifier and the value. The
/// engine writes no other entry kind, and its reader cannot step over one, so
/// anything else is rejected rather than skipped.
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

        let mut controls = Vec::with_capacity(length / CONTROL_CHUNK_LENGTH);
        while !payload.isExhausted() {
            controls.push(self.parseControl(&mut payload)?);
        }

        Ok(Some(CausticControlCollection { controls }))
    }

    fn parseControl(
        &self,
        payload: &mut ByteCursor<'_>,
    ) -> Result<CausticControl, CausticReadError> {
        let offset = payload.position();
        let tag = payload.readTag("control entry")?;
        let length = payload.readLength("control entry")?;
        if &tag != CONTROL_TAG || length != CONTROL_PAYLOAD_LENGTH {
            return Err(CausticReadError::UnexpectedControlEntry {
                offset,
                tag: String::from_utf8_lossy(&tag).into_owned(),
                length,
            });
        }

        Ok(CausticControl {
            id: payload.readU32("control entry")?,
            value: payload.readF32("control entry")?,
        })
    }
}
