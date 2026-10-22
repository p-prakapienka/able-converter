//! Rebuilds a container payload from a preserved header and a decoded image.
//!
//! The header is copied, then the compression flag and the declared image length
//! are set to describe the body actually written. The check field covers that
//! body and is recomputed.

use crate::model::digitoneii::{CONTAINER_HEADER_LEN, CONTAINER_TRAILER_LEN, FOOTER_MAGIC};
use crate::parser::digitoneii::checksum::Checksum;

use super::lz4blockencoder::Lz4BlockEncoder;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct PayloadWriter;

impl PayloadWriter {
    pub(crate) fn write(self, header: &[u8; CONTAINER_HEADER_LEN], image: &[u8]) -> Vec<u8> {
        let chain = Lz4BlockEncoder.encode(image);
        let mut payload =
            Vec::with_capacity(CONTAINER_HEADER_LEN + chain.len() + CONTAINER_TRAILER_LEN);
        payload.extend_from_slice(header);
        let declared = u32::try_from(image.len()).expect("image length fits in u32");
        payload[0x19..0x1D].copy_from_slice(&declared.to_be_bytes());
        payload[0x1D] = 1;
        payload.extend_from_slice(&chain);
        let check = Checksum::crc32ZeroInit(&chain);
        payload.extend_from_slice(&check.to_be_bytes());
        let bodyLength = u32::try_from(chain.len()).expect("chain length fits in u32");
        payload.extend_from_slice(&bodyLength.to_be_bytes());
        payload.extend_from_slice(&FOOTER_MAGIC);
        payload
    }
}
