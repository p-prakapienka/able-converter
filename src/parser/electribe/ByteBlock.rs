/// Little-endian view of a fixed-size electribe data block.
///
/// Callers validate the block length before constructing it, so offsets used by the
/// readers are always inside the block.
pub(super) struct ByteBlock<'a> {
    bytes: &'a [u8],
}

impl<'a> ByteBlock<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub(super) fn byte(&self, offset: usize) -> u8 {
        self.bytes[offset]
    }

    pub(super) fn signedByte(&self, offset: usize) -> i8 {
        i8::from_le_bytes([self.bytes[offset]])
    }

    pub(super) fn flag(&self, offset: usize) -> bool {
        self.bytes[offset] != 0
    }

    pub(super) fn word(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }

    pub(super) fn slice(&self, offset: usize, length: usize) -> &'a [u8] {
        &self.bytes[offset..offset + length]
    }

    pub(super) fn sub(&self, offset: usize, length: usize) -> ByteBlock<'a> {
        ByteBlock::new(self.slice(offset, length))
    }

    /// Decode a fixed-width name, ending at the first NUL and dropping trailing spaces.
    pub(super) fn text(&self, offset: usize, length: usize) -> String {
        let raw = self.slice(offset, length);
        let end = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
        raw[..end]
            .iter()
            .map(|&byte| char::from(byte))
            .collect::<String>()
            .trim_end()
            .to_owned()
    }
}
