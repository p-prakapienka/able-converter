use crate::model::caustic::{CausticSectionTag, MACHINE_SLOT_COUNT, RACK_HEADER_LENGTH};

use super::caustic::CausticParser;

/// Builds synthetic songs byte by byte so no copyrighted project is committed.
#[derive(Default)]
struct SongBuilder {
    body: Vec<u8>,
}

impl SongBuilder {
    fn new() -> Self {
        Self {
            body: vec![0u8; RACK_HEADER_LENGTH],
        }
    }

    fn withOutputPanel(mut self, tempo: f32, beatsPerBar: u8) -> Self {
        let mut payload = vec![0u8; 82];
        payload.extend_from_slice(&tempo.to_le_bytes());
        payload.push(beatsPerBar);
        self.pushSection(b"OUTP", &payload);
        self
    }

    fn withSlotTable(mut self, occupied: &[(usize, &str)]) -> Self {
        for slot in 0..MACHINE_SLOT_COUNT {
            let machineId = occupied
                .iter()
                .find(|(index, _)| *index == slot)
                .map(|(_, id)| *id)
                .unwrap_or("NULL");
            self.body.extend_from_slice(machineId.as_bytes());
            self.body.push(0);
        }
        self
    }

    fn withMachine(mut self, name: &str, controls: &[(u32, f32)], body: &[u8]) -> Self {
        let mut payload = Vec::new();
        if !controls.is_empty() {
            let mut collection = Vec::new();
            for (id, value) in controls {
                collection.extend_from_slice(&id.to_le_bytes());
                collection.extend_from_slice(&value.to_le_bytes());
            }
            payload.extend_from_slice(b"CCOL");
            payload.extend_from_slice(&(collection.len() as u32).to_le_bytes());
            payload.extend_from_slice(&collection);
        }
        payload.extend_from_slice(body);

        let mut fixedName = name.as_bytes().to_vec();
        fixedName.resize(10, 0);
        self.body.extend_from_slice(&fixedName);
        self.body.extend_from_slice(&[1, 2, 3, 4]);
        self.body
            .extend_from_slice(&(payload.len() as u32).to_le_bytes());
        self.body.extend_from_slice(&payload);
        self
    }

    fn withSection(mut self, tag: &[u8; 4], payload: &[u8]) -> Self {
        self.pushSection(tag, payload);
        self
    }

    fn withRawBytes(mut self, bytes: &[u8]) -> Self {
        self.body.extend_from_slice(bytes);
        self
    }

    fn pushSection(&mut self, tag: &[u8; 4], payload: &[u8]) {
        self.body.extend_from_slice(tag);
        self.body
            .extend_from_slice(&(payload.len() as u32).to_le_bytes());
        self.body.extend_from_slice(payload);
    }

    fn finish(self) -> Vec<u8> {
        let mut song = Vec::new();
        song.extend_from_slice(b"RACK");
        song.extend_from_slice(&(self.body.len() as u32).to_le_bytes());
        song.extend_from_slice(&self.body);
        song
    }
}

#[test]
fn parserReadsRackHeaderTransportAndSectionOrder() {
    let song = SongBuilder::new()
        .withOutputPanel(128.5, 4)
        .withSlotTable(&[])
        .withSection(b"EFFX", &[1, 2, 3, 4])
        .withSection(b"MIXR", &[5, 6])
        .withSection(b"MSTR", &[])
        .withSection(b"SEQN", &[7])
        .finish();

    let project = CausticParser::new(song.as_slice())
        .parse()
        .expect("parse fixture");

    assert_eq!(project.rackHeader.len(), RACK_HEADER_LENGTH);
    assert_eq!(project.inspection.transport.tempo, Some(128.5));
    assert_eq!(project.inspection.transport.beatsPerBar, Some(4));
    assert_eq!(
        project.inspection.sectionTags,
        vec![
            CausticSectionTag::OutputPanel,
            CausticSectionTag::Effects,
            CausticSectionTag::Mixer,
            CausticSectionTag::Master,
            CausticSectionTag::Sequencer,
        ]
    );
    assert_eq!(project.sections[1].payload, vec![1, 2, 3, 4]);
    assert_eq!(project.sections[2].payload, vec![5, 6]);
    assert!(project.trailingBytes.is_empty());
}

#[test]
fn parserReadsOccupiedSlotsWithControlCollections() {
    let song = SongBuilder::new()
        .withOutputPanel(120.0, 4)
        .withSlotTable(&[(0, "SSYN"), (2, "PCMS")])
        .withMachine("Lead", &[(1, 0.25), (7, -1.5)], &[9, 9, 9])
        .withMachine("Drums", &[], &[42])
        .withSection(b"SEQN", &[])
        .finish();

    let project = CausticParser::new(song.as_slice())
        .parse()
        .expect("parse fixture");

    assert_eq!(project.inspection.occupiedSlots, 2);
    assert_eq!(project.inspection.controlCount, 2);
    assert_eq!(project.machines.len(), 2);

    let lead = &project.machines[0];
    assert_eq!(lead.slot, 0);
    assert_eq!(lead.machineId, "SSYN");
    assert_eq!(lead.name, "Lead");
    assert_eq!(lead.headerBytes, vec![1, 2, 3, 4]);
    assert_eq!(lead.body, vec![9, 9, 9]);

    let controls = lead.controls.as_ref().expect("control collection");
    assert_eq!(controls.controls.len(), 2);
    assert_eq!(controls.valueOf(1), Some(0.25));
    assert_eq!(controls.valueOf(7), Some(-1.5));
    assert_eq!(controls.valueOf(99), None);

    let drums = &project.machines[1];
    assert_eq!(drums.slot, 2);
    assert_eq!(drums.machineId, "PCMS");
    assert!(drums.controls.is_none());
    assert_eq!(drums.body, vec![42]);
}

#[test]
fn parserPreservesAnUndecodedWordBetweenSections() {
    let song = SongBuilder::new()
        .withOutputPanel(120.0, 4)
        .withSlotTable(&[])
        .withSection(b"EFFX", &[])
        .withRawBytes(&[0xDE, 0xAD, 0xBE, 0xEF])
        .withSection(b"MIXR", &[])
        .finish();

    let project = CausticParser::new(song.as_slice())
        .parse()
        .expect("parse fixture");

    assert_eq!(
        project.sections[1].trailingBytes,
        vec![0xDE, 0xAD, 0xBE, 0xEF]
    );
    assert_eq!(
        project.inspection.sectionTags.last(),
        Some(&CausticSectionTag::Mixer)
    );
}

#[test]
fn parserKeepsBytesAfterTheLastKnownSection() {
    let song = SongBuilder::new()
        .withOutputPanel(120.0, 4)
        .withSlotTable(&[])
        .withRawBytes(b"WXYZ\x00\x00\x00\x00")
        .finish();

    let project = CausticParser::new(song.as_slice())
        .parse()
        .expect("parse fixture");

    assert_eq!(project.trailingBytes, b"WXYZ\x00\x00\x00\x00".to_vec());
}

#[test]
fn parserLeavesTransportUnsetWhenOutputPanelIsTooShort() {
    let mut builder = SongBuilder::new();
    builder.pushSection(b"OUTP", &[0u8; 8]);
    let song = builder.withSlotTable(&[]).finish();

    let inspection = CausticParser::new(song.as_slice())
        .inspect()
        .expect("inspect fixture");

    assert_eq!(inspection.transport.tempo, None);
    assert_eq!(inspection.transport.beatsPerBar, None);
}

#[test]
fn parserRejectsFilesWithoutARackChunk() {
    let error = CausticParser::new(b"NOPE\x00\x00\x00\x00".as_slice())
        .parse()
        .expect_err("reject fixture");

    assert!(error.to_string().contains("RACK"));
}

#[test]
fn parserRejectsDeclaredLengthsBeyondTheEndOfTheSong() {
    let mut song = Vec::new();
    song.extend_from_slice(b"RACK");
    song.extend_from_slice(&1_000_000u32.to_le_bytes());
    song.extend_from_slice(&[0u8; 16]);

    let error = CausticParser::new(song.as_slice())
        .parse()
        .expect_err("reject fixture");

    assert!(error.to_string().contains("declares a length"));
}

#[test]
fn parserRejectsTruncatedRackHeaders() {
    let mut song = Vec::new();
    song.extend_from_slice(b"RACK");
    song.extend_from_slice(&8u32.to_le_bytes());
    song.extend_from_slice(&[0u8; 8]);

    let error = CausticParser::new(song.as_slice())
        .parse()
        .expect_err("reject fixture");

    assert!(error.to_string().contains("rack header"));
}
