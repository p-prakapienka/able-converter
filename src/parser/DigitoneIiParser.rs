//! Parsing of Elektron Digitone II project (`.dn2prj`) files.
//!
//! The reader accepts the ZIP Transfer writes and the raw container a +Drive
//! read returns. Unknown image bytes are kept so the exporter can write them
//! back. See `docs/digitone-ii-format.md`.

use std::io::Read;

use crate::model::digitoneii::{
    DigitoneIiInspection, DigitoneIiProject, IMAGE_SIZE_OS110, IMAGE_SIZE_OS111,
};

use super::containerreader::ContainerReader;
pub use super::digitoneiireaderror::DigitoneIiReadError;
use super::projectarchivereader::ProjectArchiveReader;

/// Reads a Digitone II project from the supplied stream.
pub struct DigitoneIiParser<R> {
    source: R,
}

impl<R: Read> DigitoneIiParser<R> {
    #[must_use]
    pub fn new(source: R) -> Self {
        Self { source }
    }

    /// Summarise the project without exposing the preserved image.
    pub fn inspect(self) -> Result<DigitoneIiInspection, DigitoneIiReadError> {
        Ok(self.parse()?.inspect())
    }

    /// Parse the archive or raw container into a Digitone II project.
    pub fn parse(mut self) -> Result<DigitoneIiProject, DigitoneIiReadError> {
        let mut bytes = Vec::new();
        self.source.read_to_end(&mut bytes)?;
        if bytes.starts_with(b"PK") {
            let (manifest, payload) = ProjectArchiveReader.read(&bytes)?;
            let (header, image) = ContainerReader.read(&payload)?;
            return Self::project(manifest, header, image);
        }
        let (header, image) = ContainerReader.read(&bytes)?;
        let manifest = crate::model::digitoneii::DigitoneIiManifest::new(
            String::new(),
            Vec::new(),
            String::new(),
            String::new(),
            String::new(),
            Vec::new(),
        );
        Self::project(manifest, header, image)
    }

    fn project(
        manifest: crate::model::digitoneii::DigitoneIiManifest,
        header: [u8; crate::model::digitoneii::CONTAINER_HEADER_LEN],
        image: Vec<u8>,
    ) -> Result<DigitoneIiProject, DigitoneIiReadError> {
        if image.len() != IMAGE_SIZE_OS110 && image.len() != IMAGE_SIZE_OS111 {
            return Err(DigitoneIiReadError::ImageSize {
                length: image.len(),
            });
        }
        Ok(DigitoneIiProject::fromParts(manifest, header, image))
    }
}
