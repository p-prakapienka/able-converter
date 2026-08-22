use crate::model::internal::{
    Beat, BeatRange, ClipLoop, ClipSource, DiagnosticCode, DiagnosticSeverity, MidiClip, Note,
    Project, Scene, Track, TrackKind,
};
use crate::model::note::{NOTE_SCENE_LIMIT, NOTE_SCHEMA_URI, NOTE_TRACK_LIMIT};

use super::internaltonote::InternalToNoteMapper;

#[test]
fn mapsSelectedSessionGridIntoNoteWithAnalogDrift() {
    let source = projectWithClip();

    let result =
        InternalToNoteMapper::new().mapSelected(&source, vec!["track-1".to_owned()], vec![3]);

    assert!(result.diagnostics.is_empty());
    assert_eq!(result.value.schema, NOTE_SCHEMA_URI);
    assert_eq!(result.value.tempo, 124.0);
    assert_eq!(result.value.tracks.len(), 1);
    assert_eq!(result.value.scenes.len(), 1);
    assert_eq!(result.value.tracks[0].name, "Lead");
    assert_eq!(result.value.scenes[0].name, "Verse");
    assert_eq!(result.value.tracks[0].devices[0].name, "Analog Drift");
    assert_eq!(
        result.value.tracks[0].devices[0].presetUri.as_deref(),
        Some("ableton:/packs/abl-core-library/Track%20Presets/Templates/Analog%20Drift.json")
    );

    let clip = result.value.tracks[0].clipSlots[0]
        .clip
        .as_ref()
        .expect("mapped clip");
    assert_eq!(clip.name, "Lead clip");
    assert!(clip.region.r#loop.isEnabled);
    assert_eq!(clip.notes.len(), 1);
    assert_eq!(clip.notes[0].noteNumber, 60);
    assert_eq!(clip.notes[0].offVelocity, 64.0);
}

#[test]
fn reportsLossyNoteAndSceneFeatures() {
    let mut source = projectWithClip();
    source.scenes[0].tempoOverride = Some(98.0);
    source.midiClips[0]
        .loopSettings
        .as_mut()
        .unwrap()
        .startRelative = Beat(1.0);
    source.midiClips[0].notes[0].probability = Some(0.5);
    source.midiClips[0].notes.push(Note {
        pitch: 64,
        start: Beat(1.0),
        duration: Beat(0.5),
        velocity: 80.0,
        releaseVelocity: None,
        muted: true,
        probability: None,
    });

    let result = InternalToNoteMapper::new().map(&source);

    assert_eq!(
        result.value.tracks[0].clipSlots[0]
            .clip
            .as_ref()
            .unwrap()
            .notes
            .len(),
        1
    );
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::UnsupportedSceneTempoOverride
            && diagnostic.severity == DiagnosticSeverity::Warning
    }));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::UnsupportedLoopStartRelative
            && diagnostic.severity == DiagnosticSeverity::Warning
    }));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::UnsupportedNoteProbability
            && diagnostic.severity == DiagnosticSeverity::Warning
    }));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::UnsupportedMutedNote
            && diagnostic.severity == DiagnosticSeverity::Warning
    }));
}

#[test]
fn reportsLimitsBeforeCreatingAnExplicitlyTruncatedGrid() {
    let mut source = projectWithClip();
    source.tracks = (0..NOTE_TRACK_LIMIT + 1)
        .map(|index| Track {
            id: format!("track-{index}"),
            kind: TrackKind::Midi,
            name: format!("Track {index}"),
            color: None,
        })
        .collect();
    source.scenes = (0..NOTE_SCENE_LIMIT + 1)
        .map(|index| Scene {
            id: format!("scene-{index}"),
            index,
            name: format!("Scene {index}"),
            color: None,
            tempoOverride: None,
        })
        .collect();

    let result = InternalToNoteMapper::new().map(&source);

    assert_eq!(result.value.tracks.len(), NOTE_TRACK_LIMIT);
    assert_eq!(result.value.scenes.len(), NOTE_SCENE_LIMIT);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::NoteTrackLimitExceeded
            && diagnostic.severity == DiagnosticSeverity::Error
    }));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::NoteSceneLimitExceeded
            && diagnostic.severity == DiagnosticSeverity::Error
    }));
}

#[test]
fn reusesMapperWithoutRetainingDiagnostics() {
    let mapper = InternalToNoteMapper::new();
    let mut source = projectWithClip();
    source.tempo = None;
    assert!(!mapper.map(&source).diagnostics.is_empty());

    source.tempo = Some(124.0);

    assert!(mapper.map(&source).diagnostics.is_empty());
}

fn projectWithClip() -> Project {
    Project {
        tempo: Some(124.0),
        tracks: vec![Track {
            id: "track-1".to_owned(),
            kind: TrackKind::Midi,
            name: "Lead".to_owned(),
            color: Some(10),
        }],
        scenes: vec![Scene {
            id: "scene-3".to_owned(),
            index: 3,
            name: "Verse".to_owned(),
            color: Some(6),
            tempoOverride: None,
        }],
        midiClips: vec![MidiClip {
            id: "clip-1".to_owned(),
            name: "Lead clip".to_owned(),
            source: ClipSource::Session {
                trackId: "track-1".to_owned(),
                sceneIndex: 3,
            },
            contentRange: BeatRange {
                start: Beat(0.0),
                end: Beat(4.0),
            },
            loopSettings: Some(ClipLoop {
                region: BeatRange {
                    start: Beat(0.0),
                    end: Beat(4.0),
                },
                startRelative: Beat(0.0),
                enabled: true,
            }),
            disabled: false,
            notes: vec![Note {
                pitch: 60,
                start: Beat(0.0),
                duration: Beat(0.5),
                velocity: 100.0,
                releaseVelocity: Some(64.0),
                muted: false,
                probability: None,
            }],
        }],
    }
}
