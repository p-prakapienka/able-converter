use crate::mapper::internaltonote::InternalToNoteMapper;
use crate::model::internal::{
    Beat, BeatRange, ClipSource, MappingResult, MidiClip, Note, Project, Scene, Track, TrackKind,
};
use crate::web::model::preview::PreviewNoteState;

use super::projectpreviewfactory::ProjectPreviewFactory;

#[test]
fn createsAlignedSourceAndTargetPianoRolls() {
    let source = projectWithLossyNotes();
    let internalResult = MappingResult {
        value: source,
        diagnostics: Vec::new(),
    };
    let noteResult = InternalToNoteMapper::new().map(&internalResult.value);

    let preview = ProjectPreviewFactory::new().create(&internalResult, &noteResult);

    assert_eq!(preview.summary.sourceTrackCount, 1);
    assert_eq!(preview.summary.targetTrackCount, 1);
    assert_eq!(preview.summary.sourceClipCount, 1);
    assert_eq!(preview.summary.targetClipCount, 1);
    assert_eq!(preview.sourceTracks[0].clips[0].notes.len(), 3);
    assert_eq!(preview.targetTracks[0].clips[0].notes.len(), 2);
    assert_eq!(
        preview.sourceTracks[0].clips[0].notes[1].state,
        PreviewNoteState::Lossy
    );
    assert_eq!(
        preview.sourceTracks[0].clips[0].notes[2].state,
        PreviewNoteState::Omitted
    );
    assert_eq!(
        preview.targetTracks[0].clips[0].sourceId.as_deref(),
        Some("clip-1")
    );
    assert_eq!(preview.compatibility.warningCount, 2);
    assert_eq!(preview.compatibility.errorCount, 0);
}

fn projectWithLossyNotes() -> Project {
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
            notes: vec![
                Note {
                    pitch: 60,
                    start: Beat(0.0),
                    duration: Beat(1.0),
                    velocity: 100.0,
                    releaseVelocity: Some(64.0),
                    muted: false,
                    probability: None,
                },
                Note {
                    pitch: 64,
                    start: Beat(1.0),
                    duration: Beat(1.0),
                    velocity: 90.0,
                    releaseVelocity: None,
                    muted: false,
                    probability: Some(0.5),
                },
                Note {
                    pitch: 67,
                    start: Beat(2.0),
                    duration: Beat(1.0),
                    velocity: 80.0,
                    releaseVelocity: None,
                    muted: true,
                    probability: None,
                },
            ],
        }],
    }
}
