//! Semantic mapping from Ableton Live source models to the canonical model.

use crate::model::internal::{
    Beat, BeatRange, ClipLoop, ClipSource, DiagnosticCode, MappingResult, MidiClip, Note, Project,
    Scene, Track, TrackKind,
};
use crate::model::live::{
    LiveMidiClip, LiveMidiNote, LiveProject, LiveScene, LiveTrack, LiveTrackKind,
};

use super::livetointernalmappingcontext::LiveToInternalMappingContext;

/// Reusable stateless service that maps Ableton Live projects into the canonical model.
#[derive(Debug, Default, Clone, Copy)]
pub struct LiveToInternalMapper;

impl LiveToInternalMapper {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn map(&self, source: &LiveProject) -> MappingResult<Project> {
        let mut context = LiveToInternalMappingContext::new();
        let tracks = source
            .tracks
            .iter()
            .map(|track| self.mapTrack(track))
            .collect();
        let scenes = source
            .scenes
            .iter()
            .map(|scene| self.mapScene(&mut context, scene))
            .collect();
        let mut midiClips = Vec::new();

        for clip in &source.sessionMidiClips {
            self.reportUnsupportedFeatures(&mut context, clip);
            if let Some(mappedClip) = self.mapClip(&mut context, clip) {
                midiClips.push(mappedClip);
            }
        }

        context.finish(Project {
            tempo: source.inspection.tempo,
            tracks,
            scenes,
            midiClips,
        })
    }

    fn mapTrack(&self, track: &LiveTrack) -> Track {
        let name = if track.userName.trim().is_empty() {
            track.effectiveName.clone()
        } else {
            track.userName.clone()
        };

        Track {
            id: track.id.clone(),
            kind: match track.kind {
                LiveTrackKind::Midi => TrackKind::Midi,
                LiveTrackKind::Audio => TrackKind::Audio,
                LiveTrackKind::Group => TrackKind::Group,
                LiveTrackKind::Return => TrackKind::Return,
                LiveTrackKind::Main => TrackKind::Main,
            },
            name,
            color: track.color,
        }
    }

    fn mapScene(&self, context: &mut LiveToInternalMappingContext, scene: &LiveScene) -> Scene {
        let tempoOverride = if scene.tempoEnabled {
            match scene.tempo {
                Some(tempo) if tempo.is_finite() && tempo > 0.0 => Some(tempo),
                _ => {
                    context.error(
                        scene.id.clone(),
                        DiagnosticCode::InvalidSceneTempo,
                        "enabled scene tempo must be a finite positive number",
                    );
                    None
                }
            }
        } else {
            None
        };

        if scene.timeSignatureEnabled {
            let sourceValue = scene
                .timeSignatureId
                .map_or_else(|| "missing".to_owned(), |value| value.to_string());
            context.warning(
                scene.id.clone(),
                DiagnosticCode::UnsupportedSceneTimeSignature,
                format!(
                    "scene time-signature identifier {sourceValue} is preserved only in the Live source model"
                ),
            );
        }

        Scene {
            id: scene.id.clone(),
            index: scene.index,
            name: scene.name.clone(),
            color: scene.color,
            tempoOverride,
        }
    }

    fn mapClip(
        &self,
        context: &mut LiveToInternalMappingContext,
        clip: &LiveMidiClip,
    ) -> Option<MidiClip> {
        let contentRange = self.mapRange(
            context,
            clip.currentStart,
            clip.currentEnd,
            clip,
            DiagnosticCode::InvalidClipRange,
            "clip content range",
        )?;

        let loopSettings = match clip.loopSettings {
            Some(sourceLoop) => {
                let region = self.mapRange(
                    context,
                    sourceLoop.start,
                    sourceLoop.end,
                    clip,
                    DiagnosticCode::InvalidLoopRange,
                    "clip loop range",
                )?;
                if !sourceLoop.startRelative.is_finite() {
                    context.error(
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
            .filter_map(|(index, note)| self.mapNote(context, clip, note, index))
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
        &self,
        context: &mut LiveToInternalMappingContext,
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
            context.error(
                clip.id.clone(),
                code,
                format!("{description} is invalid: {start}..{end}"),
            );
            None
        }
    }

    fn mapNote(
        &self,
        context: &mut LiveToInternalMappingContext,
        clip: &LiveMidiClip,
        note: &LiveMidiNote,
        index: usize,
    ) -> Option<Note> {
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
            context.error(
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
            context.warning(
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

    fn reportUnsupportedFeatures(
        &self,
        context: &mut LiveToInternalMappingContext,
        clip: &LiveMidiClip,
    ) {
        if clip.hasClipAutomation {
            context.warning(
                clip.id.clone(),
                DiagnosticCode::UnsupportedClipAutomation,
                "clip automation is not mapped yet",
            );
        }
        if clip.hasPerNoteExpression {
            context.warning(
                clip.id.clone(),
                DiagnosticCode::UnsupportedPerNoteExpression,
                "per-note expression is not mapped yet",
            );
        }
    }
}
