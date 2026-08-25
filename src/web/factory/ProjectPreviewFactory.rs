use crate::model::internal::{
    ClipSource, Diagnostic, DiagnosticSeverity, MappingResult, MidiClip, Note, Project, TrackKind,
};
use crate::model::note::{NoteMidiClip, NoteProject};
use crate::web::model::preview::{
    CompatibilityReport, PreviewClip, PreviewDiagnostic, PreviewNote, PreviewNoteState,
    PreviewScene, PreviewTrack, ProjectPreview, ProjectPreviewSummary,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct ProjectPreviewFactory;

impl ProjectPreviewFactory {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn create(
        &self,
        internalResult: &MappingResult<Project>,
        noteResult: &MappingResult<NoteProject>,
    ) -> ProjectPreview {
        let sourceTracks = self.buildSourceTracks(&internalResult.value);
        let targetTracks = self.buildTargetTracks(&internalResult.value, &noteResult.value);
        let sourceClipCount = sourceTracks.iter().map(|track| track.clips.len()).sum();
        let targetClipCount = targetTracks.iter().map(|track| track.clips.len()).sum();
        let diagnostics = internalResult
            .diagnostics
            .iter()
            .chain(&noteResult.diagnostics)
            .map(|diagnostic| self.mapDiagnostic(diagnostic))
            .collect::<Vec<_>>();
        let warningCount = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == "warning")
            .count();
        let errorCount = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == "error")
            .count();

        ProjectPreview {
            summary: ProjectPreviewSummary {
                tempo: internalResult.value.tempo,
                sourceTrackCount: sourceTracks.len(),
                targetTrackCount: targetTracks.len(),
                sceneCount: internalResult.value.scenes.len(),
                sourceClipCount,
                targetClipCount,
            },
            scenes: internalResult
                .value
                .scenes
                .iter()
                .map(|scene| PreviewScene {
                    index: scene.index,
                    name: scene.name.clone(),
                    color: scene.color,
                })
                .collect(),
            sourceTracks,
            targetTracks,
            compatibility: CompatibilityReport {
                warningCount,
                errorCount,
                diagnostics,
            },
        }
    }

    fn buildSourceTracks(&self, source: &Project) -> Vec<PreviewTrack> {
        source
            .tracks
            .iter()
            .filter(|track| track.kind == TrackKind::Midi)
            .map(|track| PreviewTrack {
                id: track.id.clone(),
                name: track.name.clone(),
                color: track.color,
                clips: source
                    .midiClips
                    .iter()
                    .filter(|clip| self.belongsToTrack(clip, &track.id))
                    .map(|clip| self.mapSourceClip(clip))
                    .collect(),
            })
            .collect()
    }

    fn buildTargetTracks(&self, source: &Project, target: &NoteProject) -> Vec<PreviewTrack> {
        target
            .tracks
            .iter()
            .enumerate()
            .map(|(trackPosition, track)| {
                let sourceTrack = source
                    .tracks
                    .iter()
                    .filter(|candidate| candidate.kind == TrackKind::Midi)
                    .nth(trackPosition);
                let trackId = sourceTrack.map_or_else(
                    || format!("target-track:{trackPosition}"),
                    |item| item.id.clone(),
                );

                PreviewTrack {
                    id: trackId.clone(),
                    name: track.name.clone(),
                    color: Some(track.color),
                    clips: track
                        .clipSlots
                        .iter()
                        .enumerate()
                        .filter_map(|(scenePosition, slot)| {
                            let clip = slot.clip.as_ref()?;
                            let sceneIndex = source
                                .scenes
                                .get(scenePosition)
                                .map_or(scenePosition, |scene| scene.index);
                            let sourceClip = source.midiClips.iter().find(|candidate| {
                                self.belongsToSlot(candidate, &trackId, sceneIndex)
                            });
                            Some(self.mapTargetClip(
                                clip,
                                &trackId,
                                sceneIndex,
                                sourceClip.map(|item| item.id.as_str()),
                                trackPosition,
                                scenePosition,
                            ))
                        })
                        .collect(),
                }
            })
            .collect()
    }

    fn mapSourceClip(&self, clip: &MidiClip) -> PreviewClip {
        let (trackId, sceneIndex) = match &clip.source {
            ClipSource::Session {
                trackId,
                sceneIndex,
            } => (trackId.clone(), *sceneIndex),
            ClipSource::Arrangement { trackId, .. } => (trackId.clone(), 0),
        };
        let loopSettings = clip
            .loopSettings
            .unwrap_or(crate::model::internal::ClipLoop {
                region: clip.contentRange,
                startRelative: crate::model::internal::Beat(0.0),
                enabled: false,
            });

        PreviewClip {
            id: clip.id.clone(),
            sourceId: Some(clip.id.clone()),
            name: clip.name.clone(),
            trackId,
            sceneIndex,
            start: clip.contentRange.start.0,
            end: clip.contentRange.end.0,
            loopStart: loopSettings.region.start.0,
            loopEnd: loopSettings.region.end.0,
            loopEnabled: loopSettings.enabled,
            enabled: !clip.disabled,
            notes: clip
                .notes
                .iter()
                .map(|note| self.mapSourceNote(note))
                .collect(),
        }
    }

    fn mapTargetClip(
        &self,
        clip: &NoteMidiClip,
        trackId: &str,
        sceneIndex: usize,
        sourceId: Option<&str>,
        trackPosition: usize,
        scenePosition: usize,
    ) -> PreviewClip {
        PreviewClip {
            id: format!("target:{trackPosition}:{scenePosition}"),
            sourceId: sourceId.map(str::to_owned),
            name: clip.name.clone(),
            trackId: trackId.to_owned(),
            sceneIndex,
            start: clip.region.start,
            end: clip.region.end,
            loopStart: clip.region.r#loop.start,
            loopEnd: clip.region.r#loop.end,
            loopEnabled: clip.region.r#loop.isEnabled,
            enabled: clip.isEnabled,
            notes: clip
                .notes
                .iter()
                .map(|note| PreviewNote {
                    pitch: note.noteNumber,
                    start: note.startTime,
                    duration: note.duration,
                    velocity: note.velocity,
                    offVelocity: Some(note.offVelocity),
                    probability: None,
                    state: PreviewNoteState::Mapped,
                })
                .collect(),
        }
    }

    fn mapSourceNote(&self, note: &Note) -> PreviewNote {
        let state = if note.muted {
            PreviewNoteState::Omitted
        } else if note
            .probability
            .is_some_and(|probability| (probability - 1.0).abs() > f32::EPSILON)
        {
            PreviewNoteState::Lossy
        } else {
            PreviewNoteState::Source
        };

        PreviewNote {
            pitch: note.pitch,
            start: note.start.0,
            duration: note.duration.0,
            velocity: note.velocity,
            offVelocity: note.releaseVelocity,
            probability: note.probability,
            state,
        }
    }

    fn belongsToTrack(&self, clip: &MidiClip, expectedTrackId: &str) -> bool {
        match &clip.source {
            ClipSource::Session { trackId, .. } | ClipSource::Arrangement { trackId, .. } => {
                trackId == expectedTrackId
            }
        }
    }

    fn belongsToSlot(
        &self,
        clip: &MidiClip,
        expectedTrackId: &str,
        expectedSceneIndex: usize,
    ) -> bool {
        matches!(
            &clip.source,
            ClipSource::Session {
                trackId,
                sceneIndex,
            } if trackId == expectedTrackId && *sceneIndex == expectedSceneIndex
        )
    }

    fn mapDiagnostic(&self, diagnostic: &Diagnostic) -> PreviewDiagnostic {
        PreviewDiagnostic {
            severity: match diagnostic.severity {
                DiagnosticSeverity::Warning => "warning".to_owned(),
                DiagnosticSeverity::Error => "error".to_owned(),
            },
            code: format!("{:?}", diagnostic.code),
            sourceId: diagnostic.sourceId.clone(),
            message: diagnostic.message.clone(),
        }
    }
}
