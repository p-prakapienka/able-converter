use crate::model::internal::{ClipSource, DiagnosticCode, DiagnosticSeverity};
use crate::model::live::{
    ClipCounts, LiveFormatVersion, LiveLoop, LiveMidiClip, LiveMidiNote, LiveProject,
    LiveSetInspection, TrackCounts,
};

use super::livetointernal::LiveToInternalMapper;

#[test]
fn mapsSessionClipAndReportsLossyFeatures() {
    let mut source = projectWithClip();
    source.sessionMidiClips[0].hasClipAutomation = true;
    source.sessionMidiClips[0].hasPerNoteExpression = true;
    source.sessionMidiClips[0].notes.push(LiveMidiNote {
        id: Some("invalid".to_owned()),
        pitch: 200,
        time: 1.0,
        duration: 0.5,
        velocity: 100.0,
        releaseVelocity: None,
        velocityDeviation: None,
        probability: None,
        enabled: None,
    });

    let result = LiveToInternalMapper::new(&source).map();

    assert_eq!(result.value.midiClips.len(), 1);
    let clip = &result.value.midiClips[0];
    assert_eq!(clip.name, "Bass");
    assert_eq!(clip.notes.len(), 1);
    assert_eq!(clip.notes[0].pitch, 43);
    assert!(clip.notes[0].muted);
    assert_eq!(clip.notes[0].velocity, 75.5);
    assert_eq!(clip.notes[0].probability, Some(0.75));
    assert!(matches!(
        &clip.source,
        ClipSource::Session {
            trackId,
            sceneIndex: 3
        } if trackId == "42"
    ));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidNote
            && diagnostic.severity == DiagnosticSeverity::Error
    }));
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::UnsupportedClipAutomation)
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::UnsupportedPerNoteExpression)
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::UnsupportedVelocityDeviation)
    );
}

#[test]
fn rejectsInvalidClipRangeWithAnExplicitError() {
    let mut source = projectWithClip();
    source.sessionMidiClips[0].currentStart = 4.0;
    source.sessionMidiClips[0].currentEnd = 0.0;

    let result = LiveToInternalMapper::new(&source).map();

    assert!(result.value.midiClips.is_empty());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, DiagnosticCode::InvalidClipRange);
    assert_eq!(result.diagnostics[0].severity, DiagnosticSeverity::Error);
}

fn projectWithClip() -> LiveProject {
    LiveProject {
        inspection: LiveSetInspection {
            format: LiveFormatVersion {
                major: Some("5".to_owned()),
                minor: Some("11.0_11300".to_owned()),
                creator: Some("Ableton Live 11".to_owned()),
                revision: None,
            },
            tempo: Some(120.0),
            tracks: TrackCounts {
                midi: 1,
                ..TrackCounts::default()
            },
            clips: ClipCounts {
                sessionMidi: 1,
                ..ClipCounts::default()
            },
        },
        sessionMidiClips: vec![LiveMidiClip {
            id: "7".to_owned(),
            trackId: "42".to_owned(),
            sceneIndex: 3,
            name: "Bass".to_owned(),
            currentStart: 0.0,
            currentEnd: 4.0,
            loopSettings: Some(LiveLoop {
                start: 0.0,
                end: 4.0,
                startRelative: 0.0,
                enabled: true,
            }),
            disabled: false,
            notes: vec![LiveMidiNote {
                id: Some("23".to_owned()),
                pitch: 43,
                time: 0.5,
                duration: 0.25,
                velocity: 75.5,
                releaseVelocity: Some(64.0),
                velocityDeviation: Some(2.0),
                probability: Some(0.75),
                enabled: Some(false),
            }],
            hasClipAutomation: false,
            hasPerNoteExpression: false,
        }],
    }
}
