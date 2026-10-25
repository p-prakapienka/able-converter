use super::energyxtreaderror::EnergyXtReadError;

/// Little-endian reader over one record payload.
pub(super) struct ByteCursor<'a> {
    data: &'a [u8],
    position: usize,
    context: &'static str,
}

impl<'a> ByteCursor<'a> {
    pub(super) fn new(data: &'a [u8], context: &'static str) -> Self {
        Self {
            data,
            position: 0,
            context,
        }
    }

    pub(super) fn remaining(&self) -> usize {
        self.data.len() - self.position
    }

    pub(super) fn u8(&mut self) -> Result<u8, EnergyXtReadError> {
        Ok(self.take(1)?[0])
    }

    pub(super) fn u32(&mut self) -> Result<u32, EnergyXtReadError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(super) fn i32(&mut self) -> Result<i32, EnergyXtReadError> {
        let bytes = self.take(4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(super) fn f32(&mut self) -> Result<f32, EnergyXtReadError> {
        let bytes = self.take(4)?;
        Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// A `u32` length followed by Latin-1 text.
    pub(super) fn string(&mut self) -> Result<String, EnergyXtReadError> {
        let length = self.u32()? as usize;
        Ok(self
            .take(length)?
            .iter()
            .map(|&byte| char::from(byte))
            .collect())
    }

    pub(super) fn take(&mut self, count: usize) -> Result<&'a [u8], EnergyXtReadError> {
        if count > self.remaining() {
            return Err(EnergyXtReadError::Truncated {
                context: self.context,
                offset: self.position,
            });
        }
        let bytes = &self.data[self.position..self.position + count];
        self.position += count;
        Ok(bytes)
    }

    /// Fails unless every byte has been read.
    pub(super) fn expectEnd(&self) -> Result<(), EnergyXtReadError> {
        if self.remaining() == 0 {
            Ok(())
        } else {
            Err(EnergyXtReadError::InvalidRecord {
                record: self.context,
                reason: format!("{} unread bytes", self.remaining()),
            })
        }
    }
}
