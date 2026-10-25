use super::energyxtreaderror::EnergyXtReadError;
use super::xtrecord::XtRecord;

const MAX_KEY_LENGTH: usize = 32;

/// Iterates the records of a stream: `u32` key length, ASCII key, `u32` payload
/// size, payload. Container payloads are returned whole; callers read them with
/// another `RecordReader` when they need their children.
pub(super) struct RecordReader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> RecordReader<'a> {
    pub(super) fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    pub(super) fn next(&mut self) -> Result<Option<XtRecord<'a>>, EnergyXtReadError> {
        if self.position == self.data.len() {
            return Ok(None);
        }
        let start = self.position;
        let keyLength = self.u32At(start)? as usize;
        if keyLength == 0 || keyLength > MAX_KEY_LENGTH {
            return Err(EnergyXtReadError::InvalidKey { offset: start });
        }
        let keyBytes = self.slice(start + 4, keyLength)?;
        if !keyBytes.iter().all(u8::is_ascii_alphanumeric) {
            return Err(EnergyXtReadError::InvalidKey { offset: start });
        }
        let key = std::str::from_utf8(keyBytes)
            .map_err(|_| EnergyXtReadError::InvalidKey { offset: start })?;
        let sizeAt = start + 4 + keyLength;
        let size = self.u32At(sizeAt)? as usize;
        let payload = self.slice(sizeAt + 4, size)?;
        self.position = sizeAt + 4 + size;
        Ok(Some(XtRecord { key, payload }))
    }

    fn u32At(&self, offset: usize) -> Result<u32, EnergyXtReadError> {
        let bytes = self.slice(offset, 4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn slice(&self, offset: usize, count: usize) -> Result<&'a [u8], EnergyXtReadError> {
        offset
            .checked_add(count)
            .filter(|end| *end <= self.data.len())
            .map(|end| &self.data[offset..end])
            .ok_or(EnergyXtReadError::Truncated {
                context: "record stream",
                offset,
            })
    }
}
