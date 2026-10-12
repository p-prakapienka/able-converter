use super::causticreaderror::CausticReadError;

/// Bounds-checked little-endian cursor over an in-memory song.
///
/// Caustic stores nested, length-prefixed chunks, so the parser needs random
/// access to the whole song rather than a streaming reader.
pub(super) struct ByteCursor<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> ByteCursor<'a> {
    pub(super) fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    pub(super) fn position(&self) -> usize {
        self.position
    }

    pub(super) fn remaining(&self) -> usize {
        self.data.len() - self.position
    }

    pub(super) fn isExhausted(&self) -> bool {
        self.remaining() == 0
    }

    pub(super) fn peekTag(&self) -> Option<[u8; 4]> {
        let slice = self.data.get(self.position..self.position + 4)?;
        let mut tag = [0u8; 4];
        tag.copy_from_slice(slice);
        Some(tag)
    }

    pub(super) fn peekTagAt(&self, position: usize) -> Option<[u8; 4]> {
        let slice = self.data.get(position..position + 4)?;
        let mut tag = [0u8; 4];
        tag.copy_from_slice(slice);
        Some(tag)
    }

    pub(super) fn readBytes(
        &mut self,
        structure: &'static str,
        length: usize,
    ) -> Result<&'a [u8], CausticReadError> {
        let end =
            self.position
                .checked_add(length)
                .ok_or(CausticReadError::DeclaredLengthTooLarge {
                    structure,
                    length,
                    available: self.remaining(),
                })?;

        let slice = self
            .data
            .get(self.position..end)
            .ok_or(CausticReadError::UnexpectedEnd {
                structure,
                offset: self.position,
                needed: length,
                available: self.remaining(),
            })?;

        self.position = end;
        Ok(slice)
    }

    pub(super) fn readVec(
        &mut self,
        structure: &'static str,
        length: usize,
    ) -> Result<Vec<u8>, CausticReadError> {
        Ok(self.readBytes(structure, length)?.to_vec())
    }

    pub(super) fn readTag(&mut self, structure: &'static str) -> Result<[u8; 4], CausticReadError> {
        let mut tag = [0u8; 4];
        tag.copy_from_slice(self.readBytes(structure, 4)?);
        Ok(tag)
    }

    pub(super) fn readU32(&mut self, structure: &'static str) -> Result<u32, CausticReadError> {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(self.readBytes(structure, 4)?);
        Ok(u32::from_le_bytes(bytes))
    }

    pub(super) fn readF32(&mut self, structure: &'static str) -> Result<f32, CausticReadError> {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(self.readBytes(structure, 4)?);
        Ok(f32::from_le_bytes(bytes))
    }

    pub(super) fn readLength(
        &mut self,
        structure: &'static str,
    ) -> Result<usize, CausticReadError> {
        let length = self.readU32(structure)? as usize;
        if length > self.remaining() {
            return Err(CausticReadError::DeclaredLengthTooLarge {
                structure,
                length,
                available: self.remaining(),
            });
        }
        Ok(length)
    }

    /// Read a fixed-width field and trim the NUL padding the engine writes.
    pub(super) fn readFixedString(
        &mut self,
        structure: &'static str,
        field: &'static str,
        length: usize,
    ) -> Result<String, CausticReadError> {
        let bytes = self.readBytes(structure, length)?;
        let text = bytes.split(|byte| *byte == 0).next().unwrap_or(bytes);
        if !text.is_ascii() {
            return Err(CausticReadError::InvalidAscii { structure, field });
        }
        Ok(String::from_utf8_lossy(text).trim_end().to_string())
    }
}
