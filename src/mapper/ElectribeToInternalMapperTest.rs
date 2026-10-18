use crate::model::electribe::{
    ElectribeBeat, ElectribeFileKind, ElectribeMasterEffect, ElectribeMotionSlot, ElectribePart,
    ElectribePattern, ElectribeProject, ElectribeStep,
};
use crate::model::internal::{
    ClipSource, DiagnosticCode, DiagnosticSeverity, MappingResult, MidiClip, Project, TrackKind,
};

use super::electribetointernal::ElectribeToInternalMapper;
use super::internaltonote::InternalToNoteMapper;

const EPSILON: f64 = 1e-9;

fn emptyStep(index: usize) -> ElectribeStep {
    ElectribeStep {
        index,
        triggered: false,
        gate: 72,
        velocity: 96,
        unknownFlag: 0,
        noteSlots: [0; 8],
    }
}

fn emptyPart(number: usize) -> ElectribePart {
    ElectribePart {
        number,
        lastStep: 0,
        mute: false,
        voiceAssign: 1,
        motionSequence: 1,
        triggerPadVelocity: false,
        scaleMode: false,
        partPriority: 0,
        oscillator: 0,
        unknown0A: 0,
        oscillatorEdit: 0,
        filterType: 1,
        cutoff: 127,
        resonance: 0,
        egIntensity: 0,
        modulationType: 0,
        modulationSpeed: 0,
        modulationDepth: 0,
        attack: 0,
        decayRelease: 127,
        level: 127,
        pan: 0,
        ampEg: false,
        mfxSend: false,
        grooveType: 0,
        grooveDepth: 0,
        ifxOn: false,
        ifxType: 0,
        ifxEdit: 64,
        pitch: 0,
        glide: 0,
        unknown26: 0,
        steps: (0..64).map(emptyStep).collect(),
    }
}

fn emptyPattern(number: usize, name: &str) -> ElectribePattern {
    ElectribePattern {
        number,
        name: name.to_owned(),
        tempoTenths: 1200,
        swing: 0,
        lengthBars: 1,
        beat: ElectribeBeat::Sixteenth,
        key: 0,
        scale: 0,
        chordSet: 0,
        levelRaw: 0,
        gateArp: 0,
        masterEffect: ElectribeMasterEffect {
            on: false,
            effectType: 0,
            padX: 0,
            padY: 0,
            hold: false,
            unknown40: 1,
        },
        alternate1314: false,
        alternate1516: false,
        chainTo: 0,
        chainRepeat: 0,
        parts: (1..=16).map(emptyPart).collect(),
        motionSlots: Vec::new(),
    }
}

fn project(patterns: Vec<ElectribePattern>) -> ElectribeProject {
    ElectribeProject {
        fileKind: ElectribeFileKind::AllPatterns,
        product: "e2sampler".to_owned(),
        patterns,
    }
}

fn setStep(pattern: &mut ElectribePattern, part: usize, step: usize, gate: u8, notes: &[u8]) {
    let target = &mut pattern.parts[part - 1].steps[step];
    target.triggered = true;
    target.gate = gate;
    for (slot, note) in notes.iter().enumerate() {
        target.noteSlots[slot] = note + 1;
    }
}

fn clip<'a>(result: &'a MappingResult<Project>, id: &str) -> &'a MidiClip {
    result
        .value
        .midiClips
        .iter()
        .find(|clip| clip.id == id)
        .unwrap_or_else(|| panic!("missing clip {id}"))
}

fn hasDiagnostic(result: &MappingResult<Project>, code: DiagnosticCode, sourceId: &str) -> bool {
    result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == code && diagnostic.sourceId == sourceId)
}

#[test]
fn mapsPartsToTracksAndPatternsToScenes() {
    let mut first = emptyPattern(1, "Intro");
    setStep(&mut first, 1, 0, 72, &[36]);
    let mut second = emptyPattern(2, "Drop");
    second.tempoTenths = 1285;
    setStep(&mut second, 3, 4, 72, &[60]);

    let result = ElectribeToInternalMapper::new().map(&project(vec![first, second]));

    assert!(!result.hasErrors());
    assert_eq!(result.value.tempo, Some(120.0));
    assert_eq!(result.value.tracks.len(), 16);
    assert_eq!(result.value.tracks[0].id, "part:1");
    assert_eq!(result.value.tracks[0].name, "Part 1");
    assert_eq!(result.value.tracks[0].kind, TrackKind::Midi);
    assert_eq!(result.value.scenes.len(), 2);
    assert_eq!(result.value.scenes[0].name, "Intro");
    assert_eq!(result.value.scenes[0].tempoOverride, None);
    assert_eq!(result.value.scenes[1].index, 1);
    assert_eq!(result.value.scenes[1].tempoOverride, Some(128.5));
    assert_eq!(result.value.midiClips.len(), 2);

    let drop = clip(&result, "pattern:2:part:3");
    assert!(matches!(
        &drop.source,
        ClipSource::Session { trackId, sceneIndex: 1 } if trackId == "part:3"
    ));
    assert!((drop.contentRange.end.0 - 4.0).abs() < EPSILON);
    assert!(
        drop.loopSettings
            .is_some_and(|loopSettings| loopSettings.enabled)
    );
    assert_eq!(drop.notes.len(), 1);
    assert_eq!(drop.notes[0].pitch, 60);
    assert!((drop.notes[0].start.0 - 1.0).abs() < EPSILON);
    assert!((drop.notes[0].duration.0 - 0.1875).abs() < EPSILON);
    assert_eq!(drop.notes[0].velocity, 96.0);

    assert!(hasDiagnostic(
        &result,
        DiagnosticCode::UnsupportedInstrumentSettings,
        "part:1"
    ));
    assert!(!hasDiagnostic(
        &result,
        DiagnosticCode::UnsupportedInstrumentSettings,
        "part:2"
    ));
}

#[test]
fn followsTiedStepsAndKeepsChords() {
    let mut pattern = emptyPattern(1, "Ties");
    setStep(&mut pattern, 1, 0, 0xFF, &[60, 64, 67]);
    setStep(&mut pattern, 1, 1, 127, &[64, 60]);
    setStep(&mut pattern, 1, 2, 48, &[60]);
    setStep(&mut pattern, 1, 3, 72, &[62]);

    let result = ElectribeToInternalMapper::new().map(&project(vec![pattern]));
    let notes = &clip(&result, "pattern:1:part:1").notes;

    assert_eq!(notes.len(), 4);
    assert_eq!(notes[0].pitch, 60);
    assert!((notes[0].duration.0 - 0.625).abs() < EPSILON);
    assert_eq!(notes[1].pitch, 64);
    assert!((notes[1].duration.0 - 0.5).abs() < EPSILON);
    assert_eq!(notes[2].pitch, 67);
    assert!((notes[2].duration.0 - 0.25).abs() < EPSILON);
    assert_eq!(notes[3].pitch, 62);
    assert!((notes[3].start.0 - 0.75).abs() < EPSILON);
}

#[test]
fn mapsUntriggeredNoteDataAsMutedNotes() {
    let mut pattern = emptyPattern(1, "Muted");
    setStep(&mut pattern, 1, 2, 72, &[48]);
    pattern.parts[0].steps[2].triggered = false;

    let result = ElectribeToInternalMapper::new().map(&project(vec![pattern]));
    let notes = &clip(&result, "pattern:1:part:1").notes;

    assert_eq!(notes.len(), 1);
    assert!(notes[0].muted);
}

#[test]
fn timesTripletBeatsAndReportsInactiveSteps() {
    let mut pattern = emptyPattern(1, "Triplets");
    pattern.beat = ElectribeBeat::SixteenthTriplet;
    setStep(&mut pattern, 1, 11, 96, &[60]);
    setStep(&mut pattern, 1, 13, 72, &[60]);
    setStep(&mut pattern, 1, 16, 72, &[60]);

    let result = ElectribeToInternalMapper::new().map(&project(vec![pattern]));
    let partClip = clip(&result, "pattern:1:part:1");

    assert!((partClip.contentRange.end.0 - 2.0).abs() < EPSILON);
    assert_eq!(partClip.notes.len(), 1);
    assert!((partClip.notes[0].start.0 - 11.0 / 6.0).abs() < EPSILON);
    assert!((partClip.notes[0].duration.0 - 1.0 / 6.0).abs() < EPSILON);
    assert!(hasDiagnostic(
        &result,
        DiagnosticCode::OmittedInactiveSteps,
        "pattern:1:part:1"
    ));
}

#[test]
fn usesPartLastStepForThePartLoop() {
    let mut pattern = emptyPattern(1, "Polymeter");
    pattern.lengthBars = 2;
    pattern.parts[0].lastStep = 6;
    setStep(&mut pattern, 1, 5, 72, &[60]);
    setStep(&mut pattern, 1, 17, 72, &[62]);

    let result = ElectribeToInternalMapper::new().map(&project(vec![pattern]));
    let partClip = clip(&result, "pattern:1:part:1");

    assert!((partClip.contentRange.end.0 - 3.0).abs() < EPSILON);
    assert!((partClip.notes[0].start.0 - 1.25).abs() < EPSILON);
    assert!((partClip.notes[1].start.0 - 1.75).abs() < EPSILON);
    assert!(hasDiagnostic(
        &result,
        DiagnosticCode::ApproximatedPartLoop,
        "pattern:1:part:1"
    ));
}

#[test]
fn reportsEveryUnmappedPatternFeature() {
    let mut pattern = emptyPattern(1, "Features");
    pattern.swing = 12;
    pattern.key = 9;
    pattern.levelRaw = 27;
    pattern.chainTo = 4;
    pattern.alternate1314 = true;
    setStep(&mut pattern, 13, 0, 72, &[60]);
    setStep(&mut pattern, 14, 4, 72, &[60]);
    setStep(&mut pattern, 1, 0, 72, &[60]);
    pattern.parts[0].mute = true;
    pattern.parts[0].grooveDepth = 40;
    pattern.motionSlots = vec![
        ElectribeMotionSlot {
            slot: 0,
            part: 1,
            parameterId: 4,
            values: vec![64; 64],
        },
        ElectribeMotionSlot {
            slot: 1,
            part: 17,
            parameterId: 15,
            values: vec![0; 64],
        },
    ];

    let result = ElectribeToInternalMapper::new().map(&project(vec![pattern]));

    assert!(!result.hasErrors());
    assert!(clip(&result, "pattern:1:part:1").disabled);
    for (code, sourceId) in [
        (DiagnosticCode::UnsupportedSwing, "pattern:1"),
        (DiagnosticCode::UnsupportedKeyScale, "pattern:1"),
        (DiagnosticCode::UnsupportedPatternMix, "pattern:1"),
        (DiagnosticCode::UnsupportedPatternChain, "pattern:1"),
        (DiagnosticCode::UnsupportedAlternateChoke, "pattern:1"),
        (DiagnosticCode::UnsupportedClipAutomation, "pattern:1"),
        (DiagnosticCode::UnsupportedPartMute, "pattern:1:part:1"),
        (DiagnosticCode::UnsupportedGroove, "pattern:1:part:1"),
        (
            DiagnosticCode::UnsupportedClipAutomation,
            "pattern:1:part:1",
        ),
    ] {
        assert!(
            hasDiagnostic(&result, code, sourceId),
            "missing {code:?} for {sourceId}"
        );
    }
}

#[test]
fn reportsInvalidNotesAndUnknownPatterns() {
    let mut pattern = emptyPattern(1, "Invalid");
    pattern.parts[0].steps[0].triggered = true;
    pattern.parts[0].steps[0].noteSlots[0] = 200;

    let result = ElectribeToInternalMapper::new().mapPatterns(&project(vec![pattern]), &[1, 7]);

    assert!(result.hasErrors());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidNote
            && diagnostic.severity == DiagnosticSeverity::Error
    }));
    assert!(hasDiagnostic(
        &result,
        DiagnosticCode::UnknownPatternSelection,
        "pattern:7"
    ));
    assert_eq!(result.value.scenes.len(), 1);
}

#[test]
fn selectedPatternsConvertToNoteWithoutErrors() {
    let mut first = emptyPattern(3, "Verse");
    setStep(&mut first, 1, 0, 72, &[36]);
    setStep(&mut first, 2, 4, 72, &[38]);
    let mut second = emptyPattern(4, "Chorus");
    setStep(&mut second, 1, 8, 72, &[36]);

    let internal =
        ElectribeToInternalMapper::new().mapPatterns(&project(vec![first, second]), &[3, 4]);
    let note = InternalToNoteMapper::new().mapSelected(
        &internal.value,
        vec!["part:1".to_owned(), "part:2".to_owned()],
        vec![2, 3],
    );

    assert!(!internal.hasErrors());
    assert!(!note.hasErrors(), "{:?}", note.diagnostics);
    assert_eq!(note.value.tracks.len(), 2);
    assert_eq!(note.value.scenes.len(), 2);
    assert_eq!(note.value.scenes[0].name, "Verse");
    let firstSlot = note.value.tracks[0].clipSlots[0]
        .clip
        .as_ref()
        .expect("part 1 clip in Verse");
    assert_eq!(firstSlot.notes.len(), 1);
    assert_eq!(firstSlot.notes[0].noteNumber, 36);
    assert!(note.value.tracks[1].clipSlots[1].clip.is_none());
}

#[test]
fn reportsInvalidTempoOncePerPattern() {
    let mut pattern = emptyPattern(1, "Too slow");
    pattern.tempoTenths = 50;

    let result = ElectribeToInternalMapper::new().map(&project(vec![pattern]));

    assert_eq!(result.value.tempo, None);
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == DiagnosticCode::InvalidSceneTempo)
            .count(),
        1
    );
}
