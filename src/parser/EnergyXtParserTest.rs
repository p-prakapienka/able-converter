use crate::model::energyxt::{EnergyXtPartKind, EnergyXtTrackKind};

use super::energyxt::{EnergyXtParser, EnergyXtReadError};

/// Builds synthetic energyXT record streams.
#[derive(Default)]
struct StreamWriter {
    bytes: Vec<u8>,
}

impl StreamWriter {
    fn u8(&mut self, value: u8) -> &mut Self {
        self.bytes.push(value);
        self
    }

    fn u32(&mut self, value: u32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    fn i32(&mut self, value: i32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    fn f32(&mut self, value: f32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    fn string(&mut self, text: &str) -> &mut Self {
        self.u32(text.len() as u32);
        self.bytes.extend_from_slice(text.as_bytes());
        self
    }

    fn record(&mut self, key: &str, payload: &[u8]) -> &mut Self {
        self.string(key);
        self.u32(payload.len() as u32);
        self.bytes.extend_from_slice(payload);
        self
    }

    fn take(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.bytes)
    }
}

fn trackRecord(kind: u32, name: &str, parent: i32, channel: u8) -> Vec<u8> {
    let mut w = StreamWriter::default();
    w.u32(kind).u32(0).string(name);
    w.i32(40).i32(256).i32(-1).i32(parent);
    w.u8(0).u8(channel).u8(0).u8(2).i32(0).i32(0).i32(0);
    w.take()
}

fn partRecord(
    kind: u8,
    pool: i32,
    start: i32,
    offset: i32,
    length: i32,
    flags: i32,
    name: &str,
) -> Vec<u8> {
    let mut w = StreamWriter::default();
    w.u8(2).u8(kind).u8(u8::from(pool >= 0 || start != 0));
    w.i32(pool)
        .i32(start)
        .i32(offset)
        .i32(length)
        .i32(flags)
        .string(name);
    w.take()
}

fn moduleRecord(moduleClass: u8, name: &str, children: &[u8]) -> Vec<u8> {
    let mut w = StreamWriter::default();
    w.u8(1).u8(moduleClass).u8(1).i32(10).i32(20).string(name);
    w.u32(1).u32(0).u32(0).u32(0);
    w.bytes.extend_from_slice(children);
    w.take()
}

fn sequencerChildren() -> Vec<u8> {
    let mut w = StreamWriter::default();
    w.record("options", &[0; 32]);
    w.record("track", &trackRecord(2, "Bass", -1, 1));
    w.record("trackCol", &0x0080_4020u32.to_le_bytes());
    w.record("arp", &[0; 32]);
    w.record("track", &trackRecord(5, "Drums", -1, 9));

    // Pool part 0: two notes and a controller.
    w.record("part", &partRecord(1, -1, 7680, 0, 3840, 1, "riff"));
    w.record("partCol", &0xFFFF_FFFFu32.to_le_bytes());
    let mut events = StreamWriter::default();
    events.u32(3);
    events.u32(0).u8(0).u32(0x0064_2490).u32(480); // C2 note, velocity 100
    events.u32(480).u8(0).u32(0x0040_07B0); // controller 7
    events.u32(480).u8(0).u32(0x0050_2690).u32(240);
    w.record("mevents", &events.take());

    w.record("ptrack", &[0]);
    w.record("part", &partRecord(1, 0, 0, 0, 3840, 0, "riff"));
    w.record(
        "part",
        &partRecord(1, 0, 4800, -960, 2880, 4, "riff trimmed"),
    );
    w.record("ptrack", &[0]);
    w.record("part", &partRecord(3, -1, 0, 0, 3840, 0, "beat"));
    w.record("partCol", &0x0011_2233u32.to_le_bytes());
    let mut hits = StreamWriter::default();
    hits.u32(2);
    hits.u32(0).u8(0).u8(0).u32(240).u8(0);
    hits.u32(960)
        .u8(2)
        .u8(0)
        .u32(240)
        .u8(2)
        .u8(0)
        .f32(0.5)
        .u8(1)
        .f32(-0.25);
    w.record("devents", &hits.take());
    w.record("amevents", &[0; 8]);
    w.record("timesig", &[0; 8]);
    w.record("editored", &[0; 32]);
    w.take()
}

fn project(sequencer: &[u8]) -> Vec<u8> {
    let mut w = StreamWriter::default();
    w.string("XTF");
    w.record("bpm", &134.0f32.to_le_bytes());
    w.record("comp2", &moduleRecord(1, "Master", &[]));
    w.record("comp2", &moduleRecord(4, "Sequencer", sequencer));
    w.take()
}

#[test]
fn readsTracksPoolPartsAndEvents() {
    let bytes = project(&sequencerChildren());
    assert!(EnergyXtParser::<&[u8]>::looksLikeProject(&bytes));

    let result = EnergyXtParser::new(bytes.as_slice())
        .parse()
        .expect("parse fixture");

    assert_eq!(result.tempo, Some(134.0));
    assert_eq!(result.unmappedRecords, vec!["timesig".to_owned()]);
    assert_eq!(result.tracks.len(), 2);
    let bass = &result.tracks[0];
    assert_eq!(bass.kind, EnergyXtTrackKind::Instrument);
    assert_eq!(bass.name, "Bass");
    assert_eq!(bass.color, Some(0x0080_4020));
    assert_eq!(bass.midiChannel, 1);
    assert_eq!(bass.parent, None);
    assert_eq!(result.tracks[1].kind, EnergyXtTrackKind::Drum);

    assert_eq!(result.pool.len(), 1);
    let riff = &result.pool[0];
    assert_eq!(riff.kind, EnergyXtPartKind::Midi);
    assert_eq!(riff.color, None);
    assert_eq!(riff.midiEvents.len(), 3);
    assert_eq!(riff.midiEvents[1].tick, 480);
    assert_eq!(riff.midiEvents[1].status, 0xB0);
    assert_eq!(riff.midiEvents[1].length, None);
    assert_eq!(riff.midiEvents[2].tick, 960);
    assert_eq!(riff.midiEvents[2].data1, 0x26);
    assert_eq!(riff.midiEvents[2].length, Some(240));

    assert_eq!(bass.parts.len(), 2);
    assert_eq!(bass.parts[0].poolIndex, Some(0));
    let trimmed = &bass.parts[1];
    assert_eq!(trimmed.offset, -960);
    assert!(trimmed.isMuted());
    assert_eq!(trimmed.eventTime(960), 4800);
    assert!(trimmed.contains(4800));
    assert!(!trimmed.contains(4800 + 2880));

    let beat = &result.tracks[1].parts[0];
    assert_eq!(beat.kind, EnergyXtPartKind::Drum);
    assert_eq!(beat.color, Some(0x0011_2233));
    assert!(beat.hasAutomation);
    assert_eq!(beat.drumHits.len(), 2);
    assert_eq!(beat.drumHits[1].tick, 960);
    assert_eq!(beat.drumHits[1].lane, 2);
    assert_eq!(beat.drumHits[1].value(0), Some(0.5));
    assert_eq!(beat.drumHits[1].value(1), Some(-0.25));
    assert_eq!(beat.drumHits[0].value(0), None);
}

#[test]
fn rejectsFilesWithoutTheProjectHeader() {
    let error = EnergyXtParser::new(&b"\x08\x00\x00\x00filetype"[..])
        .parse()
        .expect_err("presets are not projects");
    assert!(matches!(error, EnergyXtReadError::NotAProject));
}

#[test]
fn rejectsTruncatedRecordsAndBadKeys() {
    let mut bytes = project(&sequencerChildren());
    bytes.truncate(bytes.len() - 3);
    let error = EnergyXtParser::new(bytes.as_slice())
        .parse()
        .expect_err("truncated");
    assert!(matches!(error, EnergyXtReadError::Truncated { .. }));

    let mut w = StreamWriter::default();
    w.string("XTF").string("bad key!").u32(0);
    let error = EnergyXtParser::new(w.take().as_slice())
        .parse()
        .expect_err("bad key");
    assert!(matches!(error, EnergyXtReadError::InvalidKey { .. }));
}

#[test]
fn requiresASequencerAndEventsAfterTheirPart() {
    let mut w = StreamWriter::default();
    w.string("XTF").record("bpm", &120.0f32.to_le_bytes());
    let error = EnergyXtParser::new(w.take().as_slice())
        .parse()
        .expect_err("no sequencer");
    assert!(matches!(error, EnergyXtReadError::MissingSequencer));

    let mut children = StreamWriter::default();
    children.record("mevents", &0u32.to_le_bytes());
    let error = EnergyXtParser::new(project(&children.take()).as_slice())
        .parse()
        .expect_err("events without a part");
    assert!(matches!(
        error,
        EnergyXtReadError::UnexpectedRecord { record: "mevents" }
    ));
}
