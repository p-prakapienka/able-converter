//! Semantic mapping from Korg electribe sampler source models to the canonical model.
//!
//! Parts become MIDI tracks and patterns become Session scenes; each part of each
//! pattern that holds note data becomes one Session clip.

use std::collections::HashSet;

use crate::model::electribe::{
    ELECTRIBE_PART_COUNT, ELECTRIBE_PATTERN_MOTION_PART, ElectribeBeat, ElectribePart,
    ElectribePattern, ElectribeProject,
};
use crate::model::internal::{
    Beat, BeatRange, ClipLoop, ClipSource, DiagnosticCode, MappingResult, MidiClip, Note, Project,
    Scene, Track, TrackKind,
};

use super::electribesteptimeline::StepTimeline;
use super::electribetointernalmappingcontext::ElectribeToInternalMappingContext;

const MIN_TEMPO_TENTHS: u16 = 200;
const MAX_TEMPO_TENTHS: u16 = 3000;
const MAX_NOTE_SLOT: u8 = 128;

/// Reusable stateless service that maps electribe sampler patterns into the canonical model.
#[derive(Debug, Default, Clone, Copy)]
pub struct ElectribeToInternalMapper;

impl ElectribeToInternalMapper {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Map every pattern in the source file.
    #[must_use]
    pub fn map(&self, source: &ElectribeProject) -> MappingResult<Project> {
        let numbers = source
            .patterns
            .iter()
            .map(|pattern| pattern.number)
            .collect::<Vec<_>>();
        self.mapPatterns(source, &numbers)
    }

    /// Map only the given one-based pattern numbers, in the given order.
    #[must_use]
    pub fn mapPatterns(
        &self,
        source: &ElectribeProject,
        patternNumbers: &[usize],
    ) -> MappingResult<Project> {
        let mut context = ElectribeToInternalMappingContext::new();
        let patterns = self.resolvePatterns(&mut context, source, patternNumbers);
        // Invalid tempos are reported once, by the scene that owns them.
        let tempo = patterns.first().and_then(|pattern| self.tempoOf(pattern));

        let mut scenes = Vec::new();
        let mut midiClips = Vec::new();
        let mut partsWithClips = [false; ELECTRIBE_PART_COUNT];

        for pattern in &patterns {
            scenes.push(self.mapScene(&mut context, pattern, tempo));
            for part in &pattern.parts {
                if let Some(clip) = self.mapClip(&mut context, pattern, part) {
                    partsWithClips[part.number - 1] = true;
                    midiClips.push(clip);
                }
            }
            self.reportPatternMotion(&mut context, pattern);
        }

        let tracks = (1..=ELECTRIBE_PART_COUNT)
            .map(|number| self.mapTrack(&mut context, number, partsWithClips[number - 1]))
            .collect();

        context.finish(Project {
            tempo,
            tracks,
            scenes,
            midiClips,
        })
    }

    fn resolvePatterns<'a>(
        &self,
        context: &mut ElectribeToInternalMappingContext,
        source: &'a ElectribeProject,
        patternNumbers: &[usize],
    ) -> Vec<&'a ElectribePattern> {
        patternNumbers
            .iter()
            .filter_map(|&number| {
                let pattern = source
                    .patterns
                    .iter()
                    .find(|pattern| pattern.number == number);
                if pattern.is_none() {
                    context.error(
                        self.sceneId(number),
                        DiagnosticCode::UnknownPatternSelection,
                        format!("pattern {number} does not exist in the source file"),
                    );
                }
                pattern
            })
            .collect()
    }

    fn mapTempo(
        &self,
        context: &mut ElectribeToInternalMappingContext,
        pattern: &ElectribePattern,
    ) -> Option<f64> {
        let tempo = self.tempoOf(pattern);
        if tempo.is_none() {
            context.error(
                self.sceneId(pattern.number),
                DiagnosticCode::InvalidSceneTempo,
                format!(
                    "pattern tempo {} is outside the device range 20.0..300.0 BPM",
                    f64::from(pattern.tempoTenths) / 10.0
                ),
            );
        }
        tempo
    }

    fn tempoOf(&self, pattern: &ElectribePattern) -> Option<f64> {
        (MIN_TEMPO_TENTHS..=MAX_TEMPO_TENTHS)
            .contains(&pattern.tempoTenths)
            .then(|| f64::from(pattern.tempoTenths) / 10.0)
    }

    fn mapTrack(
        &self,
        context: &mut ElectribeToInternalMappingContext,
        number: usize,
        hasClips: bool,
    ) -> Track {
        let id = self.trackId(number);
        if hasClips {
            context.warning(
                id.clone(),
                DiagnosticCode::UnsupportedInstrumentSettings,
                "part sound settings (oscillator/sample, filter, modulation, envelope, effects, \
                 level and pan) are not mapped; they remain in the electribe source model",
            );
        }

        Track {
            id,
            kind: TrackKind::Midi,
            name: format!("Part {number}"),
            color: None,
        }
    }

    fn mapScene(
        &self,
        context: &mut ElectribeToInternalMappingContext,
        pattern: &ElectribePattern,
        projectTempo: Option<f64>,
    ) -> Scene {
        let id = self.sceneId(pattern.number);
        let tempo = self.mapTempo(context, pattern);
        self.reportPatternSettings(context, pattern, &id);

        Scene {
            id,
            index: pattern.number - 1,
            name: pattern.name.clone(),
            color: None,
            tempoOverride: tempo.filter(|&tempo| Some(tempo) != projectTempo),
        }
    }

    fn reportPatternSettings(
        &self,
        context: &mut ElectribeToInternalMappingContext,
        pattern: &ElectribePattern,
        sceneId: &str,
    ) {
        if pattern.swing != 0 {
            context.warning(
                sceneId.to_owned(),
                DiagnosticCode::UnsupportedSwing,
                format!(
                    "pattern swing {:+} is not applied to note timing",
                    pattern.swing
                ),
            );
        }
        if pattern.key != 0 || pattern.scale != 0 {
            context.warning(
                sceneId.to_owned(),
                DiagnosticCode::UnsupportedKeyScale,
                format!(
                    "pattern key {} and scale {} are not represented in the canonical model",
                    pattern.key, pattern.scale
                ),
            );
        }
        if pattern.levelRaw != 0 || pattern.masterEffect.on {
            context.warning(
                sceneId.to_owned(),
                DiagnosticCode::UnsupportedPatternMix,
                format!(
                    "pattern level {} and master effect (type {}, {}) are not mapped",
                    127_u8.saturating_sub(pattern.levelRaw),
                    pattern.masterEffect.effectType,
                    if pattern.masterEffect.on { "on" } else { "off" }
                ),
            );
        }
        if pattern.chainTo != 0 {
            context.warning(
                sceneId.to_owned(),
                DiagnosticCode::UnsupportedPatternChain,
                format!(
                    "chain to pattern {} ({} repeats) is not mapped",
                    pattern.chainTo, pattern.chainRepeat
                ),
            );
        }
        self.reportAlternate(context, pattern, sceneId, pattern.alternate1314, 13);
        self.reportAlternate(context, pattern, sceneId, pattern.alternate1516, 15);
    }

    fn reportAlternate(
        &self,
        context: &mut ElectribeToInternalMappingContext,
        pattern: &ElectribePattern,
        sceneId: &str,
        enabled: bool,
        firstPart: usize,
    ) {
        let bothUsed = [firstPart, firstPart + 1]
            .iter()
            .all(|&number| self.hasNoteData(&pattern.parts[number - 1]));
        if enabled && bothUsed {
            context.warning(
                sceneId.to_owned(),
                DiagnosticCode::UnsupportedAlternateChoke,
                format!(
                    "parts {firstPart} and {} alternate (choke each other) on the device; \
                     both play independently after conversion",
                    firstPart + 1
                ),
            );
        }
    }

    fn mapClip(
        &self,
        context: &mut ElectribeToInternalMappingContext,
        pattern: &ElectribePattern,
        part: &ElectribePart,
    ) -> Option<MidiClip> {
        if !self.hasNoteData(part) {
            return None;
        }

        let id = self.clipId(pattern.number, part.number);
        let timeline = StepTimeline::new(pattern.beat, pattern.lengthBars, part.lastStep);
        let region = BeatRange {
            start: Beat(0.0),
            end: Beat(timeline.length()),
        };
        let notes = self.mapNotes(context, part, &timeline, &id);
        self.reportPartFeatures(context, pattern, part, &timeline, &id);

        Some(MidiClip {
            id,
            name: format!("Part {}", part.number),
            source: ClipSource::Session {
                trackId: self.trackId(part.number),
                sceneIndex: pattern.number - 1,
            },
            contentRange: region,
            loopSettings: Some(ClipLoop {
                region,
                startRelative: Beat(0.0),
                enabled: true,
            }),
            disabled: part.mute,
            notes,
        })
    }

    fn mapNotes(
        &self,
        context: &mut ElectribeToInternalMappingContext,
        part: &ElectribePart,
        timeline: &StepTimeline,
        clipId: &str,
    ) -> Vec<Note> {
        let played = timeline.playedSteps();
        let mut consumed = HashSet::new();
        let mut notes = Vec::new();

        for (position, &stepIndex) in played.iter().enumerate() {
            let step = &part.steps[stepIndex];
            for (slot, &rawNote) in step.noteSlots.iter().enumerate() {
                if rawNote == 0 || consumed.contains(&(position, slot)) {
                    continue;
                }
                if rawNote > MAX_NOTE_SLOT || !(1..=127).contains(&step.velocity) {
                    context.error(
                        format!("{clipId}:step:{}", stepIndex + 1),
                        DiagnosticCode::InvalidNote,
                        format!(
                            "invalid step note: slot value {rawNote}, velocity {}",
                            step.velocity
                        ),
                    );
                    continue;
                }

                let duration =
                    self.tiedDuration(part, timeline, &played, position, rawNote, &mut consumed);
                notes.push(Note {
                    pitch: rawNote - 1,
                    start: Beat(timeline.positionOf(position)),
                    duration: Beat(duration),
                    velocity: f32::from(step.velocity),
                    releaseVelocity: None,
                    muted: !step.triggered,
                    probability: None,
                });
            }
        }

        notes.sort_by(|left, right| {
            left.start
                .0
                .total_cmp(&right.start.0)
                .then(left.pitch.cmp(&right.pitch))
        });
        notes
    }

    /// Length of a note starting at `position`, following TIE gates into later steps that
    /// hold the same note. Continuation slots are marked consumed so they are not emitted
    /// again. Ties do not wrap around the end of the part loop.
    fn tiedDuration(
        &self,
        part: &ElectribePart,
        timeline: &StepTimeline,
        played: &[usize],
        position: usize,
        rawNote: u8,
        consumed: &mut HashSet<(usize, usize)>,
    ) -> f64 {
        let mut gate = part.steps[played[position]].gate;
        let mut duration = 0.0;
        let mut current = position;

        loop {
            if !timeline.isTie(gate) {
                return duration + timeline.gateLength(gate);
            }
            duration += timeline.stepLength();

            let next = current + 1;
            let Some(&nextIndex) = played.get(next) else {
                return duration;
            };
            let nextStep = &part.steps[nextIndex];
            let Some(slot) = nextStep
                .noteSlots
                .iter()
                .enumerate()
                .position(|(slot, &value)| value == rawNote && !consumed.contains(&(next, slot)))
            else {
                return duration;
            };

            consumed.insert((next, slot));
            gate = nextStep.gate;
            current = next;
        }
    }

    fn reportPartFeatures(
        &self,
        context: &mut ElectribeToInternalMappingContext,
        pattern: &ElectribePattern,
        part: &ElectribePart,
        timeline: &StepTimeline,
        clipId: &str,
    ) {
        if part.mute {
            context.warning(
                clipId.to_owned(),
                DiagnosticCode::UnsupportedPartMute,
                "muted part is represented as a disabled clip",
            );
        }
        if part.grooveDepth != 0 {
            context.warning(
                clipId.to_owned(),
                DiagnosticCode::UnsupportedGroove,
                format!(
                    "part groove type {} at depth {} is not applied to note timing",
                    part.grooveType, part.grooveDepth
                ),
            );
        }
        if timeline.usesPartLastStep() {
            context.warning(
                clipId.to_owned(),
                DiagnosticCode::ApproximatedPartLoop,
                format!(
                    "part LAST STEP {} gives a {}-beat loop inside a {}-beat pattern; the \
                     device's phase handling at pattern restart is not verified",
                    part.lastStep,
                    timeline.length(),
                    timeline.patternLength()
                ),
            );
        }

        let played = timeline.playedSteps().into_iter().collect::<HashSet<_>>();
        let inactive = part
            .steps
            .iter()
            .filter(|step| !played.contains(&step.index))
            .filter(|step| step.noteSlots.iter().any(|&value| value != 0))
            .count();
        if inactive > 0 {
            context.warning(
                clipId.to_owned(),
                DiagnosticCode::OmittedInactiveSteps,
                format!(
                    "{inactive} stored steps with notes lie outside the played length \
                     ({} bars, {} beat) and are not converted",
                    pattern.lengthBars,
                    self.beatName(pattern)
                ),
            );
        }

        for slot in pattern
            .motionSlots
            .iter()
            .filter(|slot| usize::from(slot.part) == part.number)
        {
            context.warning(
                clipId.to_owned(),
                DiagnosticCode::UnsupportedClipAutomation,
                format!(
                    "motion sequence slot {} (parameter id {}) is not mapped",
                    slot.slot + 1,
                    slot.parameterId
                ),
            );
        }
    }

    fn reportPatternMotion(
        &self,
        context: &mut ElectribeToInternalMappingContext,
        pattern: &ElectribePattern,
    ) {
        for slot in &pattern.motionSlots {
            let partNumber = usize::from(slot.part);
            let partHasClip = (1..=ELECTRIBE_PART_COUNT).contains(&partNumber)
                && self.hasNoteData(&pattern.parts[partNumber - 1]);
            if partHasClip {
                continue;
            }
            let scope = if slot.part == ELECTRIBE_PATTERN_MOTION_PART {
                "pattern-level".to_owned()
            } else {
                format!("part {partNumber}")
            };
            context.warning(
                self.sceneId(pattern.number),
                DiagnosticCode::UnsupportedClipAutomation,
                format!(
                    "{scope} motion sequence slot {} (parameter id {}) is not mapped",
                    slot.slot + 1,
                    slot.parameterId
                ),
            );
        }
    }

    fn hasNoteData(&self, part: &ElectribePart) -> bool {
        part.steps
            .iter()
            .any(|step| step.noteSlots.iter().any(|&value| value != 0))
    }

    fn beatName(&self, pattern: &ElectribePattern) -> &'static str {
        match pattern.beat {
            ElectribeBeat::Sixteenth => "16",
            ElectribeBeat::ThirtySecond => "32",
            ElectribeBeat::EighthTriplet => "8Tri",
            ElectribeBeat::SixteenthTriplet => "16Tri",
        }
    }

    fn trackId(&self, partNumber: usize) -> String {
        format!("part:{partNumber}")
    }

    fn sceneId(&self, patternNumber: usize) -> String {
        format!("pattern:{patternNumber}")
    }

    fn clipId(&self, patternNumber: usize, partNumber: usize) -> String {
        format!("pattern:{patternNumber}:part:{partNumber}")
    }
}
