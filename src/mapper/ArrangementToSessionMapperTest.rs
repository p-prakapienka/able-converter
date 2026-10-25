use crate::model::internal::{
    Beat, BeatRange, ClipSource, Diagnostic, DiagnosticCode, DiagnosticSeverity, MidiClip, Note,
    Project, Track, TrackKind,
};

use super::arrangementtosession::ArrangementToSessionMapper;

fn note(pitch: u8, start: f64, duration: f64) -> Note {
    Note {
        pitch,
        start: Beat(start),
        duration: Beat(duration),
        velocity: 100.0,
        releaseVelocity: None,
        muted: false,
        probability: None,
    }
}

fn clip(track: usize, name: &str, start: f64, end: f64, notes: Vec<Note>) -> MidiClip {
    MidiClip {
        id: format!("t{track}:{name}:{start}"),
        name: name.to_owned(),
        source: ClipSource::Arrangement {
            trackId: format!("t{track}"),
            timelineRange: BeatRange {
                start: Beat(start),
                end: Beat(end),
            },
        },
        contentRange: BeatRange {
            start: Beat(0.0),
            end: Beat(end - start),
        },
        loopSettings: None,
        disabled: false,
        notes,
    }
}

fn project(trackCount: usize, midiClips: Vec<MidiClip>) -> Project {
    Project {
        tempo: Some(120.0),
        tracks: (0..trackCount)
            .map(|index| Track {
                id: format!("t{index}"),
                kind: TrackKind::Midi,
                name: format!("Track {index}"),
                color: None,
            })
            .collect(),
        scenes: Vec::new(),
        midiClips,
    }
}

/// The clip in a slot, or `None` for an empty slot.
fn slot(project: &Project, track: usize, scene: usize) -> Option<&MidiClip> {
    let trackId = format!("t{track}");
    project.midiClips.iter().find(|clip| {
        matches!(&clip.source, ClipSource::Session { trackId: id, sceneIndex }
            if *id == trackId && *sceneIndex == scene)
    })
}

fn count(diagnostics: &[Diagnostic], code: DiagnosticCode) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == code)
        .count()
}

#[test]
fn tracksMadeOfClipsKeepTheirClips() {
    let mut clips: Vec<MidiClip> = (0..8)
        .map(|bar| {
            clip(
                0,
                "beat",
                f64::from(bar) * 4.0,
                f64::from(bar + 1) * 4.0,
                vec![note(36, 0.0, 0.25)],
            )
        })
        .collect();
    clips.push(clip(1, "verse", 0.0, 16.0, vec![note(48, 0.0, 4.0)]));
    clips.push(clip(1, "chorus", 16.0, 32.0, vec![note(50, 0.0, 4.0)]));

    let result = ArrangementToSessionMapper::new().map(&project(2, clips));

    assert!(result.diagnostics.is_empty());
    let session = &result.value;
    assert_eq!(session.scenes.len(), 2);
    assert_eq!(session.scenes[0].name, "Bar 1");
    assert_eq!(session.scenes[1].name, "Bar 5");
    for scene in 0..2 {
        let beat = slot(session, 0, scene).expect("drum slot");
        assert_eq!(beat.name, "beat");
        assert_eq!(beat.contentRange.end.0, 4.0); // one bar, looping
        assert!(
            beat.loopSettings
                .is_some_and(|loopSettings| loopSettings.enabled)
        );
    }
    assert_eq!(
        slot(session, 1, 0).map(|clip| clip.name.as_str()),
        Some("verse")
    );
    assert_eq!(
        slot(session, 1, 1).map(|clip| clip.name.as_str()),
        Some("chorus")
    );
}

#[test]
fn cutsALongClipWhereOtherTracksChange() {
    let pattern = |name: &str, start: f64, pitch: u8| {
        clip(0, name, start, start + 16.0, vec![note(pitch, 0.0, 1.0)])
    };
    let clips = vec![
        pattern("a", 0.0, 60),
        pattern("a", 16.0, 60),
        pattern("b", 32.0, 62),
        pattern("a", 48.0, 60),
        pattern("a", 64.0, 60),
        // 20 bars on track 1, with a note ringing across beat 32.
        clip(
            1,
            "pad",
            0.0,
            80.0,
            vec![
                note(40, 30.0, 4.0),
                note(41, 40.0, 1.0),
                note(42, 50.0, 1.0),
                note(43, 70.0, 1.0),
            ],
        ),
    ];

    let result = ArrangementToSessionMapper::new().map(&project(2, clips));
    let session = &result.value;

    // Track 0 changes content at 32 and 48 only: the repeats of "a" are one run.
    let names: Vec<&str> = session
        .scenes
        .iter()
        .map(|scene| scene.name.as_str())
        .collect();
    assert_eq!(names, vec!["Bar 1", "Bar 9", "Bar 13"]);
    let first = slot(session, 1, 0).expect("first pad piece");
    assert_eq!(first.name, "pad 1/3");
    assert_eq!(first.contentRange.end.0, 32.0);
    assert_eq!(first.notes[0].duration.0, 2.0); // shortened at the cut
    assert_eq!(
        slot(session, 0, 0).map(|clip| clip.contentRange.end.0),
        Some(16.0)
    );
    let last = slot(session, 1, 2).expect("last pad piece");
    assert_eq!(last.contentRange.end.0, 32.0);
    assert_eq!(last.notes.len(), 2);
    assert_eq!(
        slot(session, 0, 2).map(|clip| clip.name.as_str()),
        Some("a")
    ); // loops twice
    assert_eq!(
        count(&result.diagnostics, DiagnosticCode::NotesShortenedAtCut),
        1
    );
}

#[test]
fn cutsEveryEightBarsWhenNothingElseChanges() {
    let result = ArrangementToSessionMapper::new().map(&project(
        1,
        vec![clip(0, "solo", 0.0, 80.0, vec![note(60, 33.0, 1.0)])],
    ));

    let names: Vec<&str> = result
        .value
        .scenes
        .iter()
        .map(|scene| scene.name.as_str())
        .collect();
    assert_eq!(names, vec!["Bar 1", "Bar 9", "Bar 17"]);
    let second = slot(&result.value, 0, 1).expect("second piece");
    assert_eq!(second.notes[0].start.0, 1.0);

    let fourBars = ArrangementToSessionMapper::new()
        .withFallbackCutBars(4.0)
        .map(&project(1, vec![clip(0, "solo", 0.0, 80.0, Vec::new())]));
    assert_eq!(fourBars.value.scenes.len(), 1); // five empty pieces share one clip
    assert_eq!(
        slot(&fourBars.value, 0, 0).map(|clip| clip.contentRange.end.0),
        Some(16.0)
    );
}

#[test]
fn repeatedSectionsShareAScene() {
    let clips = vec![
        clip(0, "x", 0.0, 16.0, vec![note(60, 0.0, 1.0)]),
        clip(0, "y", 16.0, 32.0, vec![note(62, 0.0, 1.0)]),
        clip(0, "x", 32.0, 48.0, vec![note(60, 0.0, 1.0)]),
        clip(1, "w", 0.0, 16.0, vec![note(36, 0.0, 1.0)]),
        clip(1, "w", 16.0, 32.0, vec![note(36, 0.0, 1.0)]),
        clip(1, "w", 32.0, 48.0, vec![note(36, 0.0, 1.0)]),
    ];

    let result = ArrangementToSessionMapper::new().map(&project(2, clips));

    assert_eq!(result.value.scenes.len(), 2);
    assert_eq!(result.value.midiClips.len(), 4);
}

#[test]
fn rendersOverlappingPhrasesAndSplitsLongRanges() {
    let clips = vec![
        clip(0, "lead", 0.0, 48.0, vec![note(72, 0.0, 1.0)]),
        clip(
            1,
            "bass",
            40.0,
            88.0,
            vec![note(36, 0.0, 1.0), note(38, 6.0, 4.0), note(40, 20.0, 1.0)],
        ),
    ];

    let result = ArrangementToSessionMapper::new().map(&project(2, clips));
    let session = &result.value;

    // No position is free of playing clips before 88, which is too far, so the
    // first range ends at the last edge within 64 beats.
    assert_eq!(session.scenes.len(), 2);
    assert_eq!(
        slot(session, 0, 0).map(|clip| clip.name.as_str()),
        Some("lead")
    );
    let bassIn = slot(session, 1, 0).expect("rendered bass");
    assert_eq!(bassIn.contentRange.end.0, 48.0);
    assert_eq!(bassIn.notes[0].start.0, 40.0);
    assert_eq!(bassIn.notes[1].duration.0, 2.0); // 46 + 4 crosses 48
    assert!(slot(session, 0, 1).is_none());
    let bassOut = slot(session, 1, 1).expect("rest of the bass");
    assert_eq!(bassOut.contentRange.end.0, 40.0);
    assert_eq!(bassOut.notes.len(), 1);
    assert_eq!(bassOut.notes[0].start.0, 12.0);
    assert_eq!(
        count(&result.diagnostics, DiagnosticCode::NotesShortenedAtCut),
        1
    );
}

#[test]
fn skipsSilenceAndRefusesMixedSources() {
    let clips = vec![
        clip(0, "intro", 0.0, 8.0, vec![note(60, 0.0, 1.0)]),
        clip(0, "outro", 32.0, 40.0, vec![note(62, 0.0, 1.0)]),
    ];
    let result = ArrangementToSessionMapper::new().map(&project(1, clips.clone()));
    let names: Vec<&str> = result
        .value
        .scenes
        .iter()
        .map(|scene| scene.name.as_str())
        .collect();
    assert_eq!(names, vec!["Bar 1", "Bar 9"]);

    let mut mixed = project(1, clips);
    mixed.midiClips[1].source = ClipSource::Session {
        trackId: "t0".to_owned(),
        sceneIndex: 0,
    };
    let refused = ArrangementToSessionMapper::new().map(&mixed);
    assert!(refused.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::MixedSessionAndArrangement
            && diagnostic.severity == DiagnosticSeverity::Error
    }));
    assert_eq!(refused.value, mixed);

    let sessionOnly = project(1, Vec::new());
    let unchanged = ArrangementToSessionMapper::new().map(&sessionOnly);
    assert!(unchanged.diagnostics.is_empty());
    assert_eq!(unchanged.value, sessionOnly);
}
