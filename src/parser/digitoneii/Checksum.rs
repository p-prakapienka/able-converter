//! Reflected CRC-32 with a zero initial register and a complemented result.
//!
//! Width 32, polynomial `0x04C11DB7` reflected as `0xEDB88320`, init `0`,
//! refin/refout, xorout `0xFFFFFFFF`. This is the Digitone project check field.
//! It is not ISO-HDLC CRC-32, which starts from `0xFFFFFFFF`.

const REVERSED_POLY: u32 = 0xEDB8_8320;
const XOR_OUT: u32 = 0xFFFF_FFFF;

/// Computes the check field over a payload body.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Checksum;

impl Checksum {
    pub(crate) fn crc32ZeroInit(data: &[u8]) -> u32 {
        let mut crc = 0u32;
        for byte in data {
            let index = (crc ^ u32::from(*byte)) & 0xFF;
            crc = (crc >> 8) ^ TABLE[index as usize];
        }
        crc ^ XOR_OUT
    }
}

static TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut index = 0;
    while index < 256 {
        let mut value = index as u32;
        let mut bit = 0;
        while bit < 8 {
            value = if value & 1 == 1 {
                (value >> 1) ^ REVERSED_POLY
            } else {
                value >> 1
            };
            bit += 1;
        }
        table[index] = value;
        index += 1;
    }
    table
};
