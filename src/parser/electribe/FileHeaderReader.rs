use crate::model::electribe::ElectribeFileKind;

use super::byteblock::ByteBlock;
use super::electribereaderror::ElectribeReadError;

pub(super) const FILE_HEADER_SIZE: usize = 0x100;
const SUPPORTED_PRODUCT: &str = "e2sampler";

/// Validates the common 256-byte KORG header and identifies the file content.
pub(super) struct FileHeaderReader;

impl FileHeaderReader {
    pub(super) fn new() -> Self {
        Self
    }

    pub(super) fn read(
        &self,
        bytes: &[u8],
    ) -> Result<(String, ElectribeFileKind), ElectribeReadError> {
        if bytes.len() < FILE_HEADER_SIZE + 4 || &bytes[..4] != b"KORG" {
            return Err(ElectribeReadError::MissingFileHeader);
        }

        let header = ByteBlock::new(&bytes[..FILE_HEADER_SIZE]);
        let product = header.text(0x10, 0x10);
        if product != SUPPORTED_PRODUCT {
            return Err(ElectribeReadError::UnsupportedProduct(product));
        }

        let kind = self.contentKind(&bytes[FILE_HEADER_SIZE..FILE_HEADER_SIZE + 4])?;
        Ok((product, kind))
    }

    fn contentKind(&self, marker: &[u8]) -> Result<ElectribeFileKind, ElectribeReadError> {
        match marker {
            b"PTST" => Ok(ElectribeFileKind::Pattern),
            b"GLST" => Ok(ElectribeFileKind::AllPatterns),
            other => Err(ElectribeReadError::UnsupportedContent(
                String::from_utf8_lossy(other).into_owned(),
            )),
        }
    }
}
