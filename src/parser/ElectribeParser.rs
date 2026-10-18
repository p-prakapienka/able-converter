//! Parsing of Korg electribe sampler pattern files (`.e2spat` and `.e2sallpat`).

use std::io::Read;

use crate::model::electribe::{ELECTRIBE_PATTERN_COUNT, ElectribeFileKind, ElectribeProject};

pub use super::electribereaderror::ElectribeReadError;
use super::fileheaderreader::{FILE_HEADER_SIZE, FileHeaderReader};
use super::patternreader::{PATTERN_SIZE, PatternReader};

const GLOBAL_BLOCK_SIZE: usize = 0x10000;
const GLOBAL_END_MARKER_OFFSET: usize = 0xFC;

/// Reads an electribe sampler pattern or all-pattern file from the supplied stream.
pub struct ElectribeParser<R> {
    source: R,
}

impl<R: Read> ElectribeParser<R> {
    pub fn new(source: R) -> Self {
        Self { source }
    }

    /// Parse every pattern in the file into the electribe source model.
    pub fn parse(mut self) -> Result<ElectribeProject, ElectribeReadError> {
        let mut bytes = Vec::new();
        self.source.read_to_end(&mut bytes)?;
        self.parseBytes(&bytes)
    }

    fn parseBytes(&self, bytes: &[u8]) -> Result<ElectribeProject, ElectribeReadError> {
        let (product, fileKind) = FileHeaderReader::new().read(bytes)?;
        let body = &bytes[FILE_HEADER_SIZE..];
        let patternReader = PatternReader::new();

        let patterns = match fileKind {
            ElectribeFileKind::Pattern => {
                self.requireLength("pattern file", body, PATTERN_SIZE)?;
                vec![patternReader.read(&body[..PATTERN_SIZE], 1)?]
            }
            ElectribeFileKind::AllPatterns => {
                self.requireLength(
                    "all-pattern file",
                    body,
                    GLOBAL_BLOCK_SIZE + ELECTRIBE_PATTERN_COUNT * PATTERN_SIZE,
                )?;
                self.validateGlobalBlock(&body[..GLOBAL_BLOCK_SIZE])?;
                (0..ELECTRIBE_PATTERN_COUNT)
                    .map(|index| {
                        let start = GLOBAL_BLOCK_SIZE + index * PATTERN_SIZE;
                        patternReader.read(&body[start..start + PATTERN_SIZE], index + 1)
                    })
                    .collect::<Result<Vec<_>, _>>()?
            }
        };

        Ok(ElectribeProject {
            fileKind,
            product,
            patterns,
        })
    }

    fn requireLength(
        &self,
        kind: &'static str,
        body: &[u8],
        expected: usize,
    ) -> Result<(), ElectribeReadError> {
        if body.len() < expected {
            return Err(ElectribeReadError::Truncated {
                kind,
                expected: expected + FILE_HEADER_SIZE,
                actual: body.len() + FILE_HEADER_SIZE,
            });
        }
        Ok(())
    }

    fn validateGlobalBlock(&self, block: &[u8]) -> Result<(), ElectribeReadError> {
        if &block[..4] == b"GLST"
            && &block[GLOBAL_END_MARKER_OFFSET..GLOBAL_END_MARKER_OFFSET + 4] == b"GLED"
        {
            Ok(())
        } else {
            Err(ElectribeReadError::InvalidGlobalBlock)
        }
    }
}
