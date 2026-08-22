use std::io::{Cursor, Read};

use zip::ZipArchive;

use crate::mapper::internaltonote::InternalToNoteMapper;
use crate::model::internal::{
    Beat, BeatRange, ClipSource, MidiClip, Note, Project, Scene, Track, TrackKind,
};
use crate::model::note::{NOTE_SCHEMA_URI, NoteProject};

use super::note::NoteExporter;

#[test]
fn writesSongAblIntoAStoredBundle() {
    let source = projectWithOneNote();
    let mapped = InternalToNoteMapper::new().map(&source);

    let bytes = NoteExporter::new()
        .exportBundle(&mapped.value)
        .expect("bundle export should succeed");
    let mut archive = ZipArchive::new(Cursor::new(bytes)).expect("valid ZIP archive");

    assert_eq!(archive.len(), 1);
    let mut setEntry = archive.by_name("Song.abl").expect("Song.abl entry");
    assert_eq!(setEntry.compression(), zip::CompressionMethod::Stored);
    let mut json = String::new();
    setEntry.read_to_string(&mut json).expect("UTF-8 Set JSON");
    let exported: NoteProject = serde_json::from_str(&json).expect("typed Note Set JSON");
    assert_eq!(exported.schema, NOTE_SCHEMA_URI);
    assert_eq!(
        exported.tracks[0].clipSlots[0]
            .clip
            .as_ref()
            .unwrap()
            .notes
            .len(),
        1
    );
}

fn projectWithOneNote() -> Project {
    Project {
        tempo: Some(120.0),
        tracks: vec![Track {
            id: "track-1".to_owned(),
            kind: TrackKind::Midi,
            name: "Lead".to_owned(),
            color: Some(10),
        }],
        scenes: vec![Scene {
            id: "scene-1".to_owned(),
            index: 0,
            name: "Intro".to_owned(),
            color: Some(4),
            tempoOverride: None,
        }],
        midiClips: vec![MidiClip {
            id: "clip-1".to_owned(),
            name: "Lead clip".to_owned(),
            source: ClipSource::Session {
                trackId: "track-1".to_owned(),
                sceneIndex: 0,
            },
            contentRange: BeatRange {
                start: Beat(0.0),
                end: Beat(4.0),
            },
            loopSettings: None,
            disabled: false,
            notes: vec![Note {
                pitch: 60,
                start: Beat(0.0),
                duration: Beat(1.0),
                velocity: 100.0,
                releaseVelocity: Some(64.0),
                muted: false,
                probability: None,
            }],
        }],
    }
}
