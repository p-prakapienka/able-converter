//! Serialization and `.ablbundle` packaging for Ableton Note projects.

use std::io::{Cursor, Write};

use thiserror::Error;
use zip::CompressionMethod;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::model::note::NoteProject;

const SET_ENTRY_NAME: &str = "Song.abl";

#[derive(Debug, Error)]
pub enum NoteExportError {
    #[error("failed to serialize Ableton Note Set JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to write Ableton Note bundle: {0}")]
    Archive(#[from] zip::result::ZipError),
    #[error("failed to write Ableton Note bundle data: {0}")]
    Io(#[from] std::io::Error),
}

/// Reusable stateless service that exports Ableton Note projects.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoteExporter;

impl NoteExporter {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn exportSet(&self, project: &NoteProject) -> Result<Vec<u8>, NoteExportError> {
        Ok(serde_json::to_vec_pretty(project)?)
    }

    pub fn exportBundle(&self, project: &NoteProject) -> Result<Vec<u8>, NoteExportError> {
        let setBytes = self.exportSet(project)?;
        let destination = Cursor::new(Vec::new());
        let mut archive = ZipWriter::new(destination);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        archive.start_file(SET_ENTRY_NAME, options)?;
        archive.write_all(&setBytes)?;
        Ok(archive.finish()?.into_inner())
    }
}
