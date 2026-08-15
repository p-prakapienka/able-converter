//! Streaming parsing of gzip-compressed Ableton Live Set (`.als`) files.

use std::io::{BufReader, Read};

use flate2::read::GzDecoder;
use quick_xml::Reader;
use quick_xml::events::Event;

use crate::model::live::{LiveProject, LiveSetInspection};

use super::inspectionbuilder::InspectionBuilder;
pub use super::livereaderror::LiveReadError;
use super::projectmetadataparser::ProjectMetadataParser;
use super::sessionparser::SessionParser;

/// Reads a gzip-compressed Ableton Live Set from the supplied stream.
pub struct LiveParser<R> {
    source: R,
}

impl<R: Read> LiveParser<R> {
    pub fn new(source: R) -> Self {
        Self { source }
    }

    /// Inspect the Set without materialising the decompressed XML on disk.
    pub fn inspect(self) -> Result<LiveSetInspection, LiveReadError> {
        self.inspectXml()
    }

    /// Parse supported Live Set content into a format-specific source model.
    ///
    /// This first extraction slice returns Session MIDI clips. Arrangement clips are
    /// counted by the inspection model but remain a separate future parsing path.
    pub fn parse(self) -> Result<LiveProject, LiveReadError> {
        self.parseXml()
    }

    fn inspectXml(self) -> Result<LiveSetInspection, LiveReadError> {
        let decoder = GzDecoder::new(BufReader::new(self.source));
        let mut reader = Reader::from_reader(BufReader::new(decoder));
        reader.config_mut().trim_text(true);

        let mut buffer = Vec::new();
        let mut inspection = InspectionBuilder::default();

        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(element) => {
                    inspection.onElement(&element)?;
                    inspection.onStart(element.name().as_ref());
                }
                Event::Empty(element) => inspection.onElement(&element)?,
                Event::End(element) => inspection.onEnd(element.name().as_ref()),
                Event::Eof => break,
                _ => {}
            }

            buffer.clear();
        }

        inspection.finish()
    }

    fn parseXml(self) -> Result<LiveProject, LiveReadError> {
        let decoder = GzDecoder::new(BufReader::new(self.source));
        let mut reader = Reader::from_reader(BufReader::new(decoder));
        reader.config_mut().trim_text(true);

        let mut buffer = Vec::new();
        let mut inspection = InspectionBuilder::default();
        let mut parser = SessionParser::default();
        let mut metadataParser = ProjectMetadataParser::default();
        let mut ancestors: Vec<Vec<u8>> = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(element) => {
                    inspection.onElement(&element)?;
                    parser.onStart(
                        element.name().as_ref(),
                        &element,
                        ancestors.last().map(Vec::as_slice),
                        inspection.sessionDepth(),
                    )?;
                    metadataParser.onStart(
                        element.name().as_ref(),
                        &element,
                        ancestors.last().map(Vec::as_slice),
                    )?;
                    inspection.onStart(element.name().as_ref());
                    ancestors.push(element.name().as_ref().to_vec());
                }
                Event::Empty(element) => {
                    inspection.onElement(&element)?;
                    parser.onEmpty(
                        element.name().as_ref(),
                        &element,
                        ancestors.last().map(Vec::as_slice),
                        inspection.sessionDepth(),
                    )?;
                    metadataParser.onEmpty(
                        element.name().as_ref(),
                        &element,
                        ancestors.last().map(Vec::as_slice),
                    )?;
                }
                Event::End(element) => {
                    parser.onEnd(element.name().as_ref())?;
                    metadataParser.onEnd(element.name().as_ref())?;
                    inspection.onEnd(element.name().as_ref());
                    ancestors.pop();
                }
                Event::Eof => break,
                _ => {}
            }

            buffer.clear();
        }

        let (tracks, scenes) = metadataParser.finish()?;
        Ok(LiveProject {
            inspection: inspection.finish()?,
            tracks,
            scenes,
            sessionMidiClips: parser.finish()?,
        })
    }
}
