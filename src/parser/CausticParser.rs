//! Parsing of Caustic 3 song (`.caustic`) files.
//!
//! Only the current song layout is parsed. The engine also carries older
//! layouts, visible as `Sequencer::SerializeLegacy`, `BassLine::SerializeLegacy`,
//! and `PCMSynth::SerializeLegacy`. No stored format version has been located
//! yet, so the parser cannot recognise a legacy song and will fail on one
//! wherever the layouts diverge rather than silently misreading it. Supporting
//! them is deliberately left for later.

use std::io::Read;

use crate::model::caustic::{CausticInspection, CausticProject};

pub use super::causticreaderror::CausticReadError;
use super::rackparser::RackParser;

/// Reads a Caustic 3 song from the supplied stream.
///
/// The format nests length-prefixed chunks, so the song is buffered in full
/// rather than parsed as a stream.
pub struct CausticParser<R> {
    source: R,
}

impl<R: Read> CausticParser<R> {
    pub fn new(source: R) -> Self {
        Self { source }
    }

    /// Inspect the song structure without keeping machine payloads.
    pub fn inspect(self) -> Result<CausticInspection, CausticReadError> {
        Ok(self.parse()?.inspection)
    }

    /// Parse supported song content into a format-specific source model.
    ///
    /// The rack container, machine table, and control collections are decoded.
    /// Machine bodies and section payloads are preserved verbatim because their
    /// field layouts are not established yet.
    pub fn parse(mut self) -> Result<CausticProject, CausticReadError> {
        let mut data = Vec::new();
        self.source.read_to_end(&mut data)?;
        RackParser::new().parse(&data)
    }
}
