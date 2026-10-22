//! LZ4 block-format decoder for a linked block chain.
//!
//! Each block is preceded by a big-endian compressed size. A zero size ends the
//! chain. Blocks share one output buffer, so a match may reach into an earlier
//! block.

use super::digitoneiireaderror::DigitoneIiReadError;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Lz4BlockDecoder;

impl Lz4BlockDecoder {
    pub(crate) fn decode(self, body: &[u8]) -> Result<Vec<u8>, DigitoneIiReadError> {
        let mut at = 0;
        let mut output = Vec::new();
        while at + 4 <= body.len() {
            let size = usize::try_from(u32::from_be_bytes(
                body[at..at + 4].try_into().expect("slice is four bytes"),
            ))
            .expect("u32 fits in usize on this target");
            at += 4;
            if size == 0 {
                if at != body.len() {
                    let slack = body.len() - at;
                    return Err(Self::lz4(format!(
                        "{slack} bytes follow the terminating size"
                    )));
                }
                return Ok(output);
            }
            let Some(end) = at.checked_add(size) else {
                return Err(Self::lz4("block size overflows".to_owned()));
            };
            if end > body.len() {
                let remaining = body.len() - at;
                return Err(Self::lz4(format!(
                    "block claims {size} bytes but only {remaining} remain"
                )));
            }
            self.decodeBlock(&body[at..end], &mut output)?;
            at = end;
        }
        Err(Self::lz4(
            "block chain has no terminating zero size".to_owned(),
        ))
    }

    fn decodeBlock(self, input: &[u8], output: &mut Vec<u8>) -> Result<(), DigitoneIiReadError> {
        let mut at = 0;
        while at < input.len() {
            let token = input[at];
            at += 1;
            let literalLength = self.extendedLength(input, &mut at, usize::from(token >> 4), 0)?;
            let literalEnd = at
                .checked_add(literalLength)
                .ok_or_else(|| Self::lz4("literal length overflows".to_owned()))?;
            if literalEnd > input.len() {
                return Err(Self::lz4("literal run exceeds the block".to_owned()));
            }
            output.extend_from_slice(&input[at..literalEnd]);
            at = literalEnd;
            if at >= input.len() {
                break;
            }
            if at + 2 > input.len() {
                return Err(Self::lz4("match offset is truncated".to_owned()));
            }
            let offset = usize::from(u16::from_le_bytes([input[at], input[at + 1]]));
            at += 2;
            if offset == 0 || offset > output.len() {
                let produced = output.len();
                return Err(Self::lz4(format!(
                    "invalid match offset {offset} at output {produced}"
                )));
            }
            let matchLength = self.extendedLength(input, &mut at, usize::from(token & 0x0F), 4)?;
            let from = output.len() - offset;
            for index in 0..matchLength {
                let byte = output[from + index];
                output.push(byte);
            }
        }
        Ok(())
    }

    /// `base` is 0 for a literal run and 4 for a match. A nibble of 15 is extended.
    fn extendedLength(
        self,
        input: &[u8],
        at: &mut usize,
        nibble: usize,
        base: usize,
    ) -> Result<usize, DigitoneIiReadError> {
        let mut length = base + nibble;
        if nibble != 15 {
            return Ok(length);
        }
        loop {
            if *at >= input.len() {
                return Err(Self::lz4("length extension is truncated".to_owned()));
            }
            let extra = usize::from(input[*at]);
            *at += 1;
            length = length
                .checked_add(extra)
                .ok_or_else(|| Self::lz4("length extension overflows".to_owned()))?;
            if extra != 255 {
                break;
            }
        }
        Ok(length)
    }

    fn lz4(detail: String) -> DigitoneIiReadError {
        DigitoneIiReadError::Lz4 { detail }
    }
}
