//! Semantic mapping from Ableton Live source models to the canonical model.

use crate::model::internal::{
    Beat, BeatRange, ClipLoop, ClipSource, Diagnostic, DiagnosticCode, DiagnosticSeverity,
    MappingResult, MidiClip, Note, Project,
};
use crate::model::live::{LiveMidiClip, LiveMidiNote, LiveProject};

/// Maps one Ableton Live source project into the canonical project model.
pub struct LiveToInternalMapper<'a> {
    source: &'a LiveProject,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> LiveToInternalMapper<'a> {
    pub fn new(source: &'a LiveProject) -> Self {
        Self {
            source,
            diagnostics: Vec::new(),
        }
    }

    #[must_use]
    pub fn map(mut self) -> MappingResult<Project> {
        let source = self.source;
        let mut midiClips = Vec::new();

        for clip in &source.sessionMidiClips {
            self.reportUnsupportedFeatures(clip);
            if let Some(mappedClip) = self.mapClip(clip) {
                midiClips.push(mappedClip);
            }
        }

        MappingResult {
            value: Project {
                tempo: source.inspection.tempo,
                midiClips,
            },
            diagnostics: self.diagnostics,
        }
    }

    fn mapClip(&mut self, clip: &LiveMidiClip) -> Option<MidiClip> {
        let contentRange = self.mapRange(
            clip.currentStart,
            clip.currentEnd,
            clip,
            DiagnosticCode::InvalidClipRange,
            "clip content range",
        )?;

        let loopSettings = match clip.loopSettings {
            Some(sourceLoop) => {
                let region = self.mapRange(
                    sourceLoop.start,
                    sourceLoop.end,
                    clip,
                    DiagnosticCode::InvalidLoopRange,
                    "clip loop range",
                )?;
                if !sourceLoop.startRelative.is_finite() {
                    self.error(
                        clip.id.clone(),
                        DiagnosticCode::InvalidLoopRange,
                        "clip loop start-relative value is not finite",
                    );
                    return None;
                }
                Some(ClipLoop {
                    region,
                    startRelative: Beat(sourceLoop.startRelative),
                    enabled: sourceLoop.enabled,
                })
            }
            None => None,
        };

        let mut notes = clip
            .notes
            .iter()
            .enumerate()
            .filter_map(|(index, note)| self.mapNote(clip, note, index))
            .collect::<Vec<_>>();
        notes.sort_by(|left, right| {
            left.start
                .0
                .total_cmp(&right.start.0)
                .then(left.pitch.cmp(&right.pitch))
        });

        Some(MidiClip {
            id: clip.id.clone(),
            name: clip.name.clone(),
            source: ClipSource::Session {
                trackId: clip.trackId.clone(),
                sceneIndex: clip.sceneIndex,
            },
            contentRange,
            loopSettings,
            disabled: clip.disabled,
            notes,
        })
    }

    fn mapRange(
        &mut self,
        start: f64,
        end: f64,
        clip: &LiveMidiClip,
        code: DiagnosticCode,
        description: &str,
    ) -> Option<BeatRange> {
        let range = BeatRange {
            start: Beat(start),
            end: Beat(end),
        };
        if range.isValid() {
            Some(range)
        } else {
            self.error(
                clip.id.clone(),
                code,
                format!("{description} is invalid: {start}..{end}"),
            );
            None
        }
    }

    fn mapNote(&mut self, clip: &LiveMidiClip, note: &LiveMidiNote, index: usize) -> Option<Note> {
        let sourceId = note.id.as_ref().map_or_else(
            || format!("{}:note:{index}", clip.id),
            |id| format!("{}:{id}", clip.id),
        );

        let validPitch = note.pitch <= 127;
        let validTiming = note.time.is_finite() && note.duration.is_finite() && note.duration > 0.0;
        let validVelocity = note.velocity.is_finite() && (0.0..=127.0).contains(&note.velocity);
        let validReleaseVelocity = note
            .releaseVelocity
            .is_none_or(|value| value.is_finite() && (0.0..=127.0).contains(&value));
        let validProbability = note
            .probability
            .is_none_or(|value| value.is_finite() && (0.0..=1.0).contains(&value));

        if !(validPitch && validTiming && validVelocity && validReleaseVelocity && validProbability)
        {
            self.error(
                sourceId,
                DiagnosticCode::InvalidNote,
                format!(
                    "invalid note: pitch={}, time={}, duration={}, velocity={}",
                    note.pitch, note.time, note.duration, note.velocity
                ),
            );
            return None;
        }

        if note
            .velocityDeviation
            .is_some_and(|deviation| deviation != 0.0)
        {
            self.warning(
                sourceId,
                DiagnosticCode::UnsupportedVelocityDeviation,
                "note velocity deviation is preserved only in the Live source model",
            );
        }

        Some(Note {
            pitch: note.pitch as u8,
            start: Beat(note.time),
            duration: Beat(note.duration),
            velocity: note.velocity,
            releaseVelocity: note.releaseVelocity,
            muted: note.enabled == Some(false),
            probability: note.probability,
        })
    }

    fn reportUnsupportedFeatures(&mut self, clip: &LiveMidiClip) {
        if clip.hasClipAutomation {
            self.warning(
                clip.id.clone(),
                DiagnosticCode::UnsupportedClipAutomation,
                "clip automation is not mapped yet",
            );
        }
        if clip.hasPerNoteExpression {
            self.warning(
                clip.id.clone(),
                DiagnosticCode::UnsupportedPerNoteExpression,
                "per-note expression is not mapped yet",
            );
        }
    }

    fn warning(&mut self, sourceId: String, code: DiagnosticCode, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Warning,
            code,
            sourceId,
            message: message.into(),
        });
    }

    fn error(&mut self, sourceId: String, code: DiagnosticCode, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            code,
            sourceId,
            message: message.into(),
        });
    }
}
