//! Packs `manifest.json` and the binary payload into a `.dn2prj` archive.
//!
//! Both entries are stored rather than deflated. Transfer accepts either.

use std::io::{Cursor, Write};

use zip::CompressionMethod;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use super::digitoneiwriteerror::DigitoneIiWriteError;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct ProjectArchiveWriter;

impl ProjectArchiveWriter {
    pub(crate) fn write(
        self,
        manifest: &[u8],
        payloadName: &str,
        payload: &[u8],
    ) -> Result<Vec<u8>, DigitoneIiWriteError> {
        let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        archive.start_file("manifest.json", options)?;
        archive.write_all(manifest)?;
        archive.start_file(payloadName, options)?;
        archive.write_all(payload)?;
        Ok(archive.finish()?.into_inner())
    }
}
