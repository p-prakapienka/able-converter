use crate::model::energyxt::{
    EnergyXtDrumHit, EnergyXtMidiEvent, EnergyXtPart, EnergyXtPartKind, EnergyXtProject,
    EnergyXtTrack, EnergyXtTrackKind, PART_FLAG_MUTED,
};
use crate::model::internal::{ClipSource, DiagnosticCode, DiagnosticSeverity, MidiClip, TrackKind};

use super::energyxttointernal::EnergyXtToInternalMapper;

fn part(
    kind: EnergyXtPartKind,
    pool: Option<usize>,
    start: i32,
    length: i32,
    name: &str,
) -> EnergyXtPart {
    EnergyXtPart {
        kind,
        poolIndex: pool,
        start,
        offset: 0,
        length,
        flags: 0,
        name: name.to_owned(),
        color: None,
        midiEvents: Vec::new(),
        drumHits: Vec::new(),
        hasAudioEvents: false,
        hasAutomation: false,
    }
}

fn note(tick: u32, key: u8, velocity: u8, length: u32) -> EnergyXtMidiEvent {
    EnergyXtMidiEvent {
        tick,
        status: 0x90,
        data1: key,
        data2: velocity,
        length: Some(length),
    }
}

fn track(
    index: usize,
    kind: EnergyXtTrackKind,
    name: &str,
    parts: Vec<EnergyXtPart>,
) -> EnergyXtTrack {
    EnergyXtTrack {
        index,
        kind,
        name: name.to_owned(),
        color: Some(0x0012_3456),
        parent: None,
        midiChannel: 0,
        parts,
    }
}

fn project(tracks: Vec<EnergyXtTrack>, pool: Vec<EnergyXtPart>) -> EnergyXtProject {
    EnergyXtProject {
        tempo: Some(120.0),
        tracks,
        pool,
        unmappedRecords: Vec::new(),
    }
}

fn hasDiagnostic(diagnostics: &[crate::model::internal::Diagnostic], code: DiagnosticCode) -> bool {
    diagnostics.iter().any(|diagnostic| diagnostic.code == code)
}

fn timeline(clip: &MidiClip) -> (f64, f64) {
    match &clip.source {
        ClipSource::Arrangement { timelineRange, .. } => {
            (timelineRange.start.0, timelineRange.end.0)
        }
        ClipSource::Session { .. } => panic!("expected an Arrangement clip"),
    }
}

#[test]
fn mapsPartsToArrangementClipsThroughThePool() {
    let mut riff = part(EnergyXtPartKind::Midi, None, 7680, 960, "riff");
    riff.midiEvents = vec![
        note(0, 60, 100, 480),
        EnergyXtMidiEvent {
            tick: 240,
            status: 0xB0,
            data1: 7,
            data2: 90,
            length: None,
        },
        note(1920, 64, 90, 240), // past the pool part's length, inside the placement
    ];
    let placed = part(EnergyXtPartKind::Midi, Some(0), 3840, 2880, "riff copy");
    let source = project(
        vec![track(
            0,
            EnergyXtTrackKind::Instrument,
            "Keys",
            vec![placed],
        )],
        vec![riff],
    );

    let result = EnergyXtToInternalMapper::new().map(&source);

    assert_eq!(result.value.tempo, Some(120.0));
    assert_eq!(result.value.tracks[0].id, "xt:track:0");
    assert_eq!(result.value.tracks[0].kind, TrackKind::Midi);
    assert_eq!(result.value.tracks[0].color, None);
    assert!(result.value.scenes.is_empty());
    let clip = &result.value.midiClips[0];
    assert_eq!(clip.name, "riff copy");
    assert_eq!(timeline(clip), (4.0, 7.0));
    assert_eq!(clip.contentRange.end.0, 3.0);
    assert!(clip.loopSettings.is_none());
    assert_eq!(clip.notes.len(), 2);
    assert_eq!(clip.notes[0].pitch, 60);
    assert_eq!(clip.notes[0].start.0, 0.0);
    assert_eq!(clip.notes[0].duration.0, 0.5);
    assert_eq!(clip.notes[0].velocity, 100.0);
    assert_eq!(clip.notes[1].start.0, 2.0);
    assert!(hasDiagnostic(
        &result.diagnostics,
        DiagnosticCode::UnsupportedMidiMessage
    ));
}

#[test]
fn appliesTheOffsetAndSkipsEventsOutsideThePart() {
    // The left edge was dragged one beat right: start +960, offset -960.
    let mut trimmed = part(EnergyXtPartKind::Midi, None, 960, 2880, "trimmed");
    trimmed.offset = -960;
    trimmed.midiEvents = vec![note(0, 60, 100, 240), note(1440, 62, 100, 240)];
    let source = project(
        vec![track(
            0,
            EnergyXtTrackKind::Instrument,
            "Keys",
            vec![trimmed],
        )],
        Vec::new(),
    );

    let result = EnergyXtToInternalMapper::new().map(&source);

    let clip = &result.value.midiClips[0];
    assert_eq!(timeline(clip), (1.0, 4.0));
    assert_eq!(clip.notes.len(), 1);
    assert_eq!(clip.notes[0].pitch, 62);
    assert_eq!(clip.notes[0].start.0, 0.5);
}

#[test]
fn turnsDrumHitsIntoNotes() {
    let mut beat = part(EnergyXtPartKind::Drum, None, 0, 3840, "beat");
    beat.drumHits = vec![
        EnergyXtDrumHit {
            tick: 0,
            lane: 0,
            length: 240,
            values: Vec::new(),
        },
        EnergyXtDrumHit {
            tick: 960,
            lane: 2,
            length: 240,
            values: vec![(0, 0.5), (1, -0.3)],
        },
    ];
    let source = project(
        vec![track(0, EnergyXtTrackKind::Drum, "Drums", vec![beat])],
        Vec::new(),
    );

    let result = EnergyXtToInternalMapper::new().map(&source);

    let notes = &result.value.midiClips[0].notes;
    assert_eq!(notes[0].pitch, 36);
    assert_eq!(notes[0].velocity, 127.0);
    assert_eq!(notes[1].pitch, 38);
    assert_eq!(notes[1].velocity, 64.0);
    assert_eq!(notes[1].start.0, 1.0);
    assert!(hasDiagnostic(
        &result.diagnostics,
        DiagnosticCode::UnsupportedDrumHitValue
    ));
}

#[test]
fn reportsWhatItDoesNotMap() {
    let mut muted = part(EnergyXtPartKind::Midi, None, 0, 960, "muted");
    muted.flags = PART_FLAG_MUTED;
    let broken = part(EnergyXtPartKind::Midi, Some(5), 960, 960, "broken");
    let audio = part(EnergyXtPartKind::Audio, None, 0, 3840, "loop.wav");
    let mut source = project(
        vec![
            track(
                0,
                EnergyXtTrackKind::Instrument,
                "Keys",
                vec![muted, broken],
            ),
            track(1, EnergyXtTrackKind::Audio, "Loops", vec![audio]),
            track(2, EnergyXtTrackKind::Folder, "Group", Vec::new()),
        ],
        Vec::new(),
    );
    source.unmappedRecords = vec!["timesig".to_owned(), "timesig".to_owned()];

    let result = EnergyXtToInternalMapper::new().map(&source);

    assert!(result.value.midiClips.is_empty());
    assert_eq!(result.value.tracks[1].kind, TrackKind::Audio);
    assert_eq!(result.value.tracks[2].kind, TrackKind::Group);
    assert!(hasDiagnostic(
        &result.diagnostics,
        DiagnosticCode::MutedPartOmitted
    ));
    assert!(hasDiagnostic(
        &result.diagnostics,
        DiagnosticCode::UnsupportedAudioPart
    ));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidPoolReference
            && diagnostic.severity == DiagnosticSeverity::Error
    }));
    let timeline = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::UnsupportedTimelineRecord)
        .collect::<Vec<_>>();
    assert_eq!(timeline.len(), 1);
    assert!(timeline[0].message.starts_with("2 "));
}
