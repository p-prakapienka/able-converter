//! Semantic mapping from the canonical project model to Ableton Note Set data.

use std::collections::BTreeMap;

use serde_json::json;

use crate::model::internal::{
    ClipSource, DiagnosticCode, MappingResult, MidiClip, Note, Project, Scene, Track, TrackKind,
};
use crate::model::note::{
    NOTE_CLIP_BEAT_LIMIT, NOTE_SCENE_LIMIT, NOTE_SCHEMA_URI, NOTE_TRACK_LIMIT, NoteClipRegion,
    NoteClipSlot, NoteDevice, NoteGroove, NoteGrooveEvent, NoteGrooveLoop, NoteLoop,
    NoteMasterMixer, NoteMasterTrack, NoteMetadata, NoteMidiClip, NoteMidiNote, NoteMixer,
    NoteProject, NoteRepeatArpeggio, NoteScene, NoteTimeSignature, NoteTrack,
};

use super::internaltonotemappingcontext::InternalToNoteMappingContext;

const DEFAULT_TEMPO: f64 = 120.0;
const DEFAULT_COLOR: i32 = 0;
const ANALOG_DRIFT_PRESET_URI: &str =
    "ableton:/packs/abl-core-library/Track%20Presets/Templates/Analog%20Drift.json";

/// Reusable stateless service that maps canonical Session grids into Ableton Note data.
#[derive(Debug, Default, Clone, Copy)]
pub struct InternalToNoteMapper;

impl InternalToNoteMapper {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn map(&self, source: &Project) -> MappingResult<NoteProject> {
        let selectedTrackIds = source
            .tracks
            .iter()
            .filter(|track| track.kind == TrackKind::Midi)
            .map(|track| track.id.clone())
            .collect();
        let selectedSceneIndices = source.scenes.iter().map(|scene| scene.index).collect();

        self.mapWithContext(
            source,
            InternalToNoteMappingContext::new(selectedTrackIds, selectedSceneIndices),
        )
    }

    #[must_use]
    pub fn mapSelected(
        &self,
        source: &Project,
        selectedTrackIds: Vec<String>,
        selectedSceneIndices: Vec<usize>,
    ) -> MappingResult<NoteProject> {
        self.mapWithContext(
            source,
            InternalToNoteMappingContext::new(selectedTrackIds, selectedSceneIndices),
        )
    }

    fn mapWithContext(
        &self,
        source: &Project,
        mut context: InternalToNoteMappingContext,
    ) -> MappingResult<NoteProject> {
        let tracks = self.resolveTracks(source, &mut context);
        let scenes = self.resolveScenes(source, &mut context);
        let tempo = self.mapTempo(source, &mut context);
        let noteScenes = scenes.iter().map(|scene| self.mapScene(scene)).collect();
        let noteTracks = tracks
            .iter()
            .enumerate()
            .map(|(index, track)| self.mapTrack(source, &mut context, track, &scenes, index))
            .collect();

        context.finish(NoteProject {
            schema: NOTE_SCHEMA_URI.to_owned(),
            stepEditorResolution: "1/16".to_owned(),
            tempo,
            globalGrooveAmount: 0.0,
            timeSignature: NoteTimeSignature { upper: 4, lower: 4 },
            rootNote: 0,
            scale: "major".to_owned(),
            melodicLayout: "chromatic".to_owned(),
            tracks: noteTracks,
            returnTracks: Vec::new(),
            masterTrack: self.createMasterTrack(),
            scenes: noteScenes,
            grooves: vec![self.createSwingGroove()],
            metadata: NoteMetadata {
                usedFeatures: Vec::new(),
            },
        })
    }

    fn resolveTracks(
        &self,
        source: &Project,
        context: &mut InternalToNoteMappingContext,
    ) -> Vec<Track> {
        let selectedTrackCount = context.selectedTrackIds().len();
        if selectedTrackCount > NOTE_TRACK_LIMIT {
            context.error(
                "project",
                DiagnosticCode::NoteTrackLimitExceeded,
                format!(
                    "{} selected tracks exceed Note's {NOTE_TRACK_LIMIT}-track limit",
                    selectedTrackCount
                ),
            );
        }

        let selectedIds = context
            .selectedTrackIds()
            .iter()
            .take(NOTE_TRACK_LIMIT)
            .cloned()
            .collect::<Vec<_>>();
        let mut tracks = Vec::new();

        for trackId in selectedIds {
            let Some(track) = source
                .tracks
                .iter()
                .find(|track| track.id == trackId)
                .cloned()
            else {
                context.error(
                    trackId,
                    DiagnosticCode::UnknownTrackSelection,
                    "selected track does not exist",
                );
                continue;
            };

            if track.kind != TrackKind::Midi {
                context.error(
                    track.id.clone(),
                    DiagnosticCode::UnsupportedTrackKind,
                    "only MIDI tracks can be written by the minimal Note mapper",
                );
                continue;
            }
            tracks.push(track);
        }

        tracks
    }

    fn resolveScenes(
        &self,
        source: &Project,
        context: &mut InternalToNoteMappingContext,
    ) -> Vec<Scene> {
        let selectedSceneCount = context.selectedSceneIndices().len();
        if selectedSceneCount > NOTE_SCENE_LIMIT {
            context.error(
                "project",
                DiagnosticCode::NoteSceneLimitExceeded,
                format!(
                    "{} selected scenes exceed Note's {NOTE_SCENE_LIMIT}-scene limit",
                    selectedSceneCount
                ),
            );
        }

        let selectedIndices = context
            .selectedSceneIndices()
            .iter()
            .take(NOTE_SCENE_LIMIT)
            .copied()
            .collect::<Vec<_>>();
        let mut scenes = Vec::new();

        for sceneIndex in selectedIndices {
            let Some(scene) = source
                .scenes
                .iter()
                .find(|scene| scene.index == sceneIndex)
                .cloned()
            else {
                context.error(
                    format!("scene:{sceneIndex}"),
                    DiagnosticCode::UnknownSceneSelection,
                    "selected scene does not exist",
                );
                continue;
            };

            if scene.tempoOverride.is_some() {
                context.warning(
                    scene.id.clone(),
                    DiagnosticCode::UnsupportedSceneTempoOverride,
                    "scene tempo overrides are not represented by the minimal Note writer",
                );
            }
            scenes.push(scene);
        }

        scenes
    }

    fn mapTempo(&self, source: &Project, context: &mut InternalToNoteMappingContext) -> f64 {
        match source.tempo {
            Some(tempo) if tempo.is_finite() && tempo > 0.0 => tempo,
            _ => {
                context.warning(
                    "project",
                    DiagnosticCode::MissingProjectTempo,
                    format!("project tempo is missing or invalid; using {DEFAULT_TEMPO} BPM"),
                );
                DEFAULT_TEMPO
            }
        }
    }

    fn mapScene(&self, scene: &Scene) -> NoteScene {
        NoteScene {
            name: scene.name.clone(),
            color: scene.color,
        }
    }

    fn mapTrack(
        &self,
        source: &Project,
        context: &mut InternalToNoteMappingContext,
        track: &Track,
        scenes: &[Scene],
        trackIndex: usize,
    ) -> NoteTrack {
        let clipSlots = scenes
            .iter()
            .enumerate()
            .map(|(scenePosition, scene)| NoteClipSlot {
                hasStop: true,
                clip: self.mapSlot(
                    source,
                    context,
                    track,
                    scene,
                    trackIndex == 0 && scenePosition == 0,
                ),
            })
            .collect();

        NoteTrack {
            kind: "midi".to_owned(),
            name: track.name.clone(),
            color: track.color.unwrap_or(DEFAULT_COLOR),
            isSelected: trackIndex == 0,
            clipSlots,
            isArmed: false,
            isNoteRepeatOn: false,
            noteRepeatRate: "1/16".to_owned(),
            noteRepeatArpeggio: NoteRepeatArpeggio {
                style: "chordRepeat".to_owned(),
            },
            uiOctaveIndex: 4,
            midiInputMode: "auto".to_owned(),
            midiOutputEndpoint: None,
            devices: vec![self.createAnalogDriftDevice()],
            mixer: self.createTrackMixer(),
        }
    }

    fn mapSlot(
        &self,
        source: &Project,
        context: &mut InternalToNoteMappingContext,
        track: &Track,
        scene: &Scene,
        isPlaying: bool,
    ) -> Option<NoteMidiClip> {
        let (firstClip, hasDuplicate) = {
            let mut matchingClips = source.midiClips.iter().filter(|clip| {
                matches!(
                    &clip.source,
                    ClipSource::Session {
                        trackId,
                        sceneIndex
                    } if trackId == &track.id && *sceneIndex == scene.index
                )
            });
            (
                matchingClips.next().cloned(),
                matchingClips.next().is_some(),
            )
        };

        if hasDuplicate {
            context.error(
                format!("{}:{}", track.id, scene.index),
                DiagnosticCode::DuplicateSessionSlot,
                "multiple clips target the same Note clip slot",
            );
        }

        firstClip.as_ref().and_then(|clip| {
            self.mapClip(
                context,
                clip,
                track.color.unwrap_or(DEFAULT_COLOR),
                isPlaying,
            )
        })
    }

    fn mapClip(
        &self,
        context: &mut InternalToNoteMappingContext,
        clip: &MidiClip,
        color: i32,
        isPlaying: bool,
    ) -> Option<NoteMidiClip> {
        let clipLength = clip.contentRange.end.0 - clip.contentRange.start.0;
        if clipLength > NOTE_CLIP_BEAT_LIMIT {
            context.error(
                clip.id.clone(),
                DiagnosticCode::NoteClipLengthExceeded,
                format!(
                    "clip length {clipLength} beats exceeds the current \
                     {NOTE_CLIP_BEAT_LIMIT}-beat Note limit"
                ),
            );
            return None;
        }

        let loopSettings = clip.loopSettings.map_or(
            NoteLoop {
                start: clip.contentRange.start.0,
                end: clip.contentRange.end.0,
                isEnabled: false,
            },
            |loopSettings| {
                if loopSettings.startRelative.0.abs() > f64::EPSILON {
                    context.warning(
                        clip.id.clone(),
                        DiagnosticCode::UnsupportedLoopStartRelative,
                        "non-zero Live loop start-relative is not represented in Note",
                    );
                }
                NoteLoop {
                    start: loopSettings.region.start.0,
                    end: loopSettings.region.end.0,
                    isEnabled: loopSettings.enabled,
                }
            },
        );
        let notes = clip
            .notes
            .iter()
            .enumerate()
            .filter_map(|(index, note)| self.mapNote(context, clip, note, index))
            .collect();

        Some(NoteMidiClip {
            isPlaying,
            name: clip.name.clone(),
            color,
            isEnabled: !clip.disabled,
            region: NoteClipRegion {
                start: clip.contentRange.start.0,
                end: clip.contentRange.end.0,
                r#loop: loopSettings,
            },
            grooveId: 1,
            stepEditorScrollPosition: 0.0,
            notes,
            envelopes: Vec::new(),
        })
    }

    fn mapNote(
        &self,
        context: &mut InternalToNoteMappingContext,
        clip: &MidiClip,
        note: &Note,
        index: usize,
    ) -> Option<NoteMidiNote> {
        let sourceId = format!("{}:note:{index}", clip.id);
        if note.muted {
            context.warning(
                sourceId,
                DiagnosticCode::UnsupportedMutedNote,
                "muted note is omitted because the verified Note subset has no note mute field",
            );
            return None;
        }
        if note
            .probability
            .is_some_and(|probability| (probability - 1.0).abs() > f32::EPSILON)
        {
            context.warning(
                sourceId,
                DiagnosticCode::UnsupportedNoteProbability,
                "note probability is not represented by the minimal Note writer",
            );
        }

        Some(NoteMidiNote {
            noteNumber: note.pitch,
            startTime: note.start.0,
            duration: note.duration.0,
            velocity: note.velocity,
            offVelocity: note.releaseVelocity.unwrap_or(0.0),
        })
    }

    fn createAnalogDriftDevice(&self) -> NoteDevice {
        let parameters = BTreeMap::from([
            ("Enabled".to_owned(), json!(true)),
            ("Macro0".to_owned(), json!(0.0)),
            ("Macro1".to_owned(), json!(0.0)),
            ("Macro2".to_owned(), json!(0.0)),
            ("Macro3".to_owned(), json!(0.0)),
            ("Macro4".to_owned(), json!(0.0)),
            ("Macro5".to_owned(), json!(0.0)),
            ("Macro6".to_owned(), json!(0.0)),
            ("Macro7".to_owned(), json!(0.0)),
        ]);

        NoteDevice {
            presetUri: Some(ANALOG_DRIFT_PRESET_URI.to_owned()),
            kind: "instrumentRack".to_owned(),
            name: "Analog Drift".to_owned(),
            lockId: 1001,
            lockSeal: -1_741_001_056,
            parameters,
            chains: Vec::new(),
        }
    }

    fn createTrackMixer(&self) -> NoteMixer {
        NoteMixer {
            pan: 0.0,
            soloCue: false,
            speakerOn: true,
            volume: 0.0,
            sends: Vec::new(),
        }
    }

    fn createMasterTrack(&self) -> NoteMasterTrack {
        NoteMasterTrack {
            color: 23,
            isSelected: false,
            devices: Vec::new(),
            mixer: NoteMasterMixer {
                pan: 0.0,
                volume: 0.0,
            },
        }
    }

    fn createSwingGroove(&self) -> NoteGroove {
        let eventTimes = [
            0.0,
            1.0 / 3.0,
            0.5,
            5.0 / 6.0,
            1.0,
            4.0 / 3.0,
            1.5,
            11.0 / 6.0,
            2.0,
            7.0 / 3.0,
            2.5,
            17.0 / 6.0,
            3.0,
            10.0 / 3.0,
            3.5,
            23.0 / 6.0,
        ];

        NoteGroove {
            id: 1,
            name: "Swing 16ths".to_owned(),
            base: "1/16".to_owned(),
            r#loop: NoteGrooveLoop {
                start: 0.0,
                end: 4.0,
            },
            events: eventTimes
                .into_iter()
                .map(|time| NoteGrooveEvent { time })
                .collect(),
        }
    }
}
