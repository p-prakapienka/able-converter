use crate::model::caustic::{
    CausticInspection, CausticMachine, CausticProject, CausticSection, CausticSectionTag,
    CausticTransport, RACK_HEADER_LENGTH,
};

use super::bytecursor::ByteCursor;
use super::causticreaderror::CausticReadError;
use super::machinetableparser::MachineTableParser;
use super::transportparser::TransportParser;

/// Four-character identifier of the rack chunk that holds a whole song.
const RACK_TAG: &[u8; 4] = b"RACK";

/// Bytes the parser will step over when looking for the next section tag.
///
/// Sections are sometimes followed by one word that belongs to no decoded
/// field. The bytes are preserved on the preceding section instead of being
/// dropped, and the search is limited to a single word so that unrelated
/// payload bytes are never mistaken for a tag.
const SECTION_RESYNC_LENGTH: usize = 4;

/// Walks the top-level rack chunk and its sections.
///
/// The rack header is not decoded, so if the format carries a version number it
/// is most likely in there. Until it is found, songs written by older releases
/// cannot be told apart from current ones before parsing starts.
#[derive(Default)]
pub(super) struct RackParser {
    machineTableParser: MachineTableParser,
    transportParser: TransportParser,
}

impl RackParser {
    pub(super) fn new() -> Self {
        Self {
            machineTableParser: MachineTableParser::new(),
            transportParser: TransportParser::new(),
        }
    }

    pub(super) fn parse(&self, data: &[u8]) -> Result<CausticProject, CausticReadError> {
        let mut cursor = ByteCursor::new(data);
        if cursor.peekTag().as_ref() != Some(RACK_TAG) {
            return Err(CausticReadError::MissingRackChunk);
        }

        cursor.readTag("rack")?;
        let rackLength = cursor.readLength("rack")?;
        let mut rack = ByteCursor::new(cursor.readBytes("rack", rackLength)?);
        let rackHeader = rack.readVec("rack header", RACK_HEADER_LENGTH)?;

        let mut transport = CausticTransport::default();
        let mut machines: Vec<CausticMachine> = Vec::new();
        let mut slots = Vec::new();
        let mut sections: Vec<CausticSection> = Vec::new();

        while let Some(tag) = rack.peekTag() {
            let Some(sectionTag) = CausticSectionTag::fromBytes(&tag) else {
                break;
            };

            rack.readTag("section")?;
            let length = rack.readLength("section")?;
            let payload = rack.readVec("section", length)?;

            if sectionTag == CausticSectionTag::OutputPanel {
                transport = self.transportParser.parse(&payload)?;
                let (parsedSlots, parsedMachines) = self.machineTableParser.parse(&mut rack)?;
                slots = parsedSlots;
                machines = parsedMachines;
            }

            let trailingBytes = self.resynchronise(&mut rack)?;
            sections.push(CausticSection {
                tag: sectionTag,
                payload,
                trailingBytes,
            });
        }

        let remaining = rack.remaining();
        let mut trailingBytes = rack.readVec("rack", remaining)?;
        let afterRack = cursor.remaining();
        trailingBytes.extend_from_slice(cursor.readBytes("song", afterRack)?);

        let inspection = CausticInspection {
            rackLength,
            transport,
            occupiedSlots: slots.iter().filter(|slot| slot.occupied).count(),
            controlCount: machines
                .iter()
                .filter_map(|machine| machine.controls.as_ref())
                .map(|controls| controls.controls.len())
                .sum(),
            slots,
            sectionTags: sections.iter().map(|section| section.tag).collect(),
        };

        Ok(CausticProject {
            inspection,
            rackHeader,
            machines,
            sections,
            trailingBytes,
        })
    }

    /// Step over at most one undecoded word when it separates two sections.
    fn resynchronise(&self, rack: &mut ByteCursor<'_>) -> Result<Vec<u8>, CausticReadError> {
        if rack.isExhausted() {
            return Ok(Vec::new());
        }

        if rack
            .peekTag()
            .is_some_and(|tag| CausticSectionTag::fromBytes(&tag).is_some())
        {
            return Ok(Vec::new());
        }

        let candidate = rack.position() + SECTION_RESYNC_LENGTH;
        let followsKnownTag = rack
            .peekTagAt(candidate)
            .is_some_and(|tag| CausticSectionTag::fromBytes(&tag).is_some());

        if followsKnownTag {
            rack.readVec("section padding", SECTION_RESYNC_LENGTH)
        } else {
            Ok(Vec::new())
        }
    }
}
