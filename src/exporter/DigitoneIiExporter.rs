//! Serialization of Digitone II projects back to a container or a `.dn2prj` archive.
//!
//! The compressed body is a new LZ4 encoding of the preserved image. It will not
//! match Elektron's compressor byte for byte. Decoding it produces the same image.

use crate::model::digitoneii::{DigitoneIiProject, IMAGE_SIZE_OS110, IMAGE_SIZE_OS111};

pub use super::digitoneiwriteerror::DigitoneIiWriteError;
use super::payloadwriter::PayloadWriter;
use super::projectarchivewriter::ProjectArchiveWriter;

/// Reusable stateless service that exports Digitone II projects.
#[derive(Debug, Default, Clone, Copy)]
pub struct DigitoneIiExporter;

impl DigitoneIiExporter {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Write the binary container, including its recomputed check field.
    pub fn exportPayload(
        &self,
        project: &DigitoneIiProject,
    ) -> Result<Vec<u8>, DigitoneIiWriteError> {
        self.requireImage(project)?;
        Ok(PayloadWriter.write(project.header(), project.image()))
    }

    /// Write a `.dn2prj` archive using the manifest bytes that were parsed.
    ///
    /// A raw container has no manifest. Packing one requires a payload name,
    /// which only an archive source provides.
    pub fn exportProject(
        &self,
        project: &DigitoneIiProject,
    ) -> Result<Vec<u8>, DigitoneIiWriteError> {
        if project.manifest.payloadName.is_empty() || project.manifest.bytes().is_empty() {
            return Err(DigitoneIiWriteError::MissingPayloadName);
        }
        let payload = self.exportPayload(project)?;
        ProjectArchiveWriter.write(
            project.manifest.bytes(),
            &project.manifest.payloadName,
            &payload,
        )
    }

    fn requireImage(&self, project: &DigitoneIiProject) -> Result<(), DigitoneIiWriteError> {
        let length = project.image().len();
        if length != IMAGE_SIZE_OS110 && length != IMAGE_SIZE_OS111 {
            return Err(DigitoneIiWriteError::ImageSize { length });
        }
        Ok(())
    }
}
