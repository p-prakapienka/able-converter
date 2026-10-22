//! Reads the 31-byte Digitone container and returns the decoded image.
//!
//! The header layout is the one corrected on 2026-10-05: four big-endian words
//! from offset `0x0D`, a compression flag at `0x1D`, and the body at `0x1F`.
//! The earlier little-endian reading of this header does not match soundbanks
//! outside bank A.

use crate::model::digitoneii::{
    CONTAINER_HEADER_LEN, CONTAINER_MAGIC, CONTAINER_PAIR, CONTAINER_TRAILER_LEN,
    CONTENT_KIND_PROJECT, DEVICE_DIGITONE_II, FOOTER_MAGIC,
};

use super::checksum::Checksum;
use super::digitoneiireaderror::DigitoneIiReadError;
use super::lz4blockdecoder::Lz4BlockDecoder;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct ContainerReader;

impl ContainerReader {
    pub(crate) fn read(
        self,
        payload: &[u8],
    ) -> Result<([u8; CONTAINER_HEADER_LEN], Vec<u8>), DigitoneIiReadError> {
        if payload.len() < CONTAINER_HEADER_LEN + CONTAINER_TRAILER_LEN {
            return Err(DigitoneIiReadError::PayloadTooShort {
                length: payload.len(),
            });
        }
        if payload[0..4] != CONTAINER_MAGIC {
            return Err(DigitoneIiReadError::BadMagic);
        }
        if payload[payload.len() - 4..] != FOOTER_MAGIC {
            return Err(DigitoneIiReadError::BadFooter);
        }
        if payload[4..8] != CONTAINER_PAIR {
            return Err(DigitoneIiReadError::UnexpectedPair {
                found: [payload[4], payload[5], payload[6], payload[7]],
            });
        }
        if payload[0x1E] != u8::try_from(CONTAINER_TRAILER_LEN).expect("trailer length fits") {
            return Err(DigitoneIiReadError::UnexpectedTrailerLength {
                found: payload[0x1E],
            });
        }
        if payload[0x08] != DEVICE_DIGITONE_II {
            return Err(DigitoneIiReadError::NotDigitoneIi {
                device: payload[0x08],
            });
        }
        let contentKind = u32::from_be_bytes(payload[0x0D..0x11].try_into().expect("four bytes"));
        if contentKind != CONTENT_KIND_PROJECT {
            return Err(DigitoneIiReadError::NotProject { contentKind });
        }

        let bodyEnd = payload.len() - CONTAINER_TRAILER_LEN;
        let body = &payload[CONTAINER_HEADER_LEN..bodyEnd];
        let storedLength = u32::from_be_bytes(
            payload[bodyEnd + 4..bodyEnd + 8]
                .try_into()
                .expect("four bytes"),
        );
        let computed =
            u32::try_from(body.len()).map_err(|_| DigitoneIiReadError::LengthMismatch {
                stored: storedLength,
                computed: body.len(),
            })?;
        if storedLength != computed {
            return Err(DigitoneIiReadError::LengthMismatch {
                stored: storedLength,
                computed: body.len(),
            });
        }
        let storedCheck = u32::from_be_bytes(
            payload[bodyEnd..bodyEnd + 4]
                .try_into()
                .expect("four bytes"),
        );
        let computedCheck = Checksum::crc32ZeroInit(body);
        if storedCheck != computedCheck {
            return Err(DigitoneIiReadError::ChecksumMismatch {
                stored: storedCheck,
                computed: computedCheck,
            });
        }

        let declared = u32::from_be_bytes(payload[0x19..0x1D].try_into().expect("four bytes"));
        let compressed = payload[0x1D] != 0;
        let image = if compressed {
            Lz4BlockDecoder.decode(body)?
        } else {
            body.to_vec()
        };
        if u32::try_from(image.len()).ok() != Some(declared) {
            return Err(DigitoneIiReadError::UncompressedLengthMismatch {
                declared,
                actual: image.len(),
            });
        }

        let mut header = [0u8; CONTAINER_HEADER_LEN];
        header.copy_from_slice(&payload[..CONTAINER_HEADER_LEN]);
        Ok((header, image))
    }
}
