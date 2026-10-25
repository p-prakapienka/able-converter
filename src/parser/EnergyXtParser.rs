//! Parsing of energyXT 3.0 project (`.xt`) files.

use std::io::Read;

use crate::model::energyxt::EnergyXtProject;

use super::bytecursor::ByteCursor;
pub use super::energyxtreaderror::EnergyXtReadError;
use super::recordreader::RecordReader;
use super::sequencebuilder::SequenceBuilder;

/// A project starts with the string "XTF": its `u32` length, then the letters.
const PROJECT_MAGIC: [u8; 7] = [3, 0, 0, 0, b'X', b'T', b'F'];
const SEQUENCER_MODULE_CLASS: u8 = 4;

/// Reads the sequencer content of an energyXT project from the supplied stream.
pub struct EnergyXtParser<R> {
    source: R,
}

impl<R: Read> EnergyXtParser<R> {
    pub fn new(source: R) -> Self {
        Self { source }
    }

    /// Whether `prefix`, the first bytes of a file, starts like an energyXT project.
    #[must_use]
    pub fn looksLikeProject(prefix: &[u8]) -> bool {
        prefix.starts_with(&PROJECT_MAGIC)
    }

    /// Parse the project tempo, tracks, part pool and placed parts.
    pub fn parse(mut self) -> Result<EnergyXtProject, EnergyXtReadError> {
        let mut bytes = Vec::new();
        self.source.read_to_end(&mut bytes)?;
        self.parseBytes(&bytes)
    }

    fn parseBytes(&self, bytes: &[u8]) -> Result<EnergyXtProject, EnergyXtReadError> {
        if !Self::looksLikeProject(bytes) {
            return Err(EnergyXtReadError::NotAProject);
        }
        let mut reader = RecordReader::new(&bytes[PROJECT_MAGIC.len()..]);
        let mut tempo = None;
        let mut sequence = None;

        while let Some(record) = reader.next()? {
            match record.key {
                "bpm" => tempo = Some(self.readTempo(record.payload)?),
                "comp2" if sequence.is_none() => {
                    let (moduleClass, children) = self.splitModule(record.payload)?;
                    if moduleClass == SEQUENCER_MODULE_CLASS {
                        sequence = Some(self.readSequencer(children)?);
                    }
                }
                _ => {}
            }
        }

        let sequence = sequence.ok_or(EnergyXtReadError::MissingSequencer)?;
        Ok(sequence.finish(tempo))
    }

    fn readTempo(&self, payload: &[u8]) -> Result<f64, EnergyXtReadError> {
        let mut cursor = ByteCursor::new(payload, "bpm");
        let tempo = cursor.f32()?;
        cursor.expectEnd()?;
        Ok(f64::from(tempo))
    }

    /// A module record holds a header (`u8` version, `u8` class, `u8` role, two
    /// `i32` positions, the name and four `u32`), then its child records.
    fn splitModule<'a>(&self, payload: &'a [u8]) -> Result<(u8, &'a [u8]), EnergyXtReadError> {
        let mut cursor = ByteCursor::new(payload, "comp2");
        let _version = cursor.u8()?;
        let moduleClass = cursor.u8()?;
        let _role = cursor.u8()?;
        let _x = cursor.i32()?;
        let _y = cursor.i32()?;
        let _name = cursor.string()?;
        cursor.take(16)?;
        let headerLength = payload.len() - cursor.remaining();
        Ok((moduleClass, &payload[headerLength..]))
    }

    fn readSequencer(&self, children: &[u8]) -> Result<SequenceBuilder, EnergyXtReadError> {
        let mut reader = RecordReader::new(children);
        let mut builder = SequenceBuilder::new();
        while let Some(record) = reader.next()? {
            builder.onRecord(record)?;
        }
        Ok(builder)
    }
}
