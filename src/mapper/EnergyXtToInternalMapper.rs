//! Semantic mapping from energyXT source models to the canonical model.
//!
//! energyXT has a timeline but no Session view, so every part becomes an
//! Arrangement clip. `ArrangementToSessionMapper` lays those out as scenes.

use crate::model::energyxt::{
    DRUM_VALUE_CUTOFF, DRUM_VALUE_PAN, DRUM_VALUE_VOLUME, ENERGYXT_TICKS_PER_BEAT, EnergyXtPart,
    EnergyXtPartKind, EnergyXtProject, EnergyXtTrack, EnergyXtTrackKind,
};
use crate::model::internal::{
    Beat, BeatRange, ClipSource, DiagnosticCode, MappingResult, MidiClip, Note, Project, Track,
    TrackKind,
};

use super::energyxttointernalmappingcontext::EnergyXtToInternalMappingContext;

/// Drum lanes become notes from this key upwards, lane 0 on C1.
const DRUM_BASE_PITCH: u16 = 36;
const MAX_VELOCITY: f32 = 127.0;

/// Reusable stateless service that maps energyXT projects into the canonical model.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnergyXtToInternalMapper;

impl EnergyXtToInternalMapper {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn map(&self, source: &EnergyXtProject) -> MappingResult<Project> {
        let mut context = EnergyXtToInternalMappingContext::new();
        self.reportUnmappedRecords(&mut context, source);

        let mut tracks = Vec::new();
        let mut midiClips = Vec::new();
        for track in &source.tracks {
            tracks.push(self.mapTrack(&mut context, track));
            for (partIndex, part) in track.parts.iter().enumerate() {
                if let Some(clip) = self.mapPart(&mut context, source, track, part, partIndex) {
                    midiClips.push(clip);
                }
            }
        }

        context.finish(Project {
            tempo: source.tempo,
            tracks,
            scenes: Vec::new(),
            midiClips,
        })
    }

    fn reportUnmappedRecords(
        &self,
        context: &mut EnergyXtToInternalMappingContext,
        source: &EnergyXtProject,
    ) {
        let mut reported: Vec<&str> = Vec::new();
        for key in &source.unmappedRecords {
            if reported.contains(&key.as_str()) {
                continue;
            }
            reported.push(key);
            let count = source
                .unmappedRecords
                .iter()
                .filter(|other| *other == key)
                .count();
            context.warning(
                "project".to_owned(),
                DiagnosticCode::UnsupportedTimelineRecord,
                format!(
                    "{count} energyXT `{key}` record(s) are not mapped; the conversion assumes \
                     one tempo and 4/4"
                ),
            );
        }
    }

    fn trackId(&self, track: &EnergyXtTrack) -> String {
        format!("xt:track:{}", track.index)
    }

    fn mapTrack(
        &self,
        context: &mut EnergyXtToInternalMappingContext,
        track: &EnergyXtTrack,
    ) -> Track {
        let kind = match track.kind {
            EnergyXtTrackKind::Instrument | EnergyXtTrackKind::Drum => TrackKind::Midi,
            EnergyXtTrackKind::Audio => TrackKind::Audio,
            EnergyXtTrackKind::Folder => TrackKind::Group,
            EnergyXtTrackKind::Unknown(code) => {
                context.warning(
                    self.trackId(track),
                    DiagnosticCode::UnknownTrackKind,
                    format!(
                        "track kind {code} is unknown; the track and its {} part(s) are not mapped",
                        track.parts.len()
                    ),
                );
                TrackKind::Group
            }
        };

        // energyXT colours are not Live palette indices, so none is carried over.
        Track {
            id: self.trackId(track),
            kind,
            name: track.name.clone(),
            color: None,
        }
    }

    fn mapPart(
        &self,
        context: &mut EnergyXtToInternalMappingContext,
        source: &EnergyXtProject,
        track: &EnergyXtTrack,
        placed: &EnergyXtPart,
        partIndex: usize,
    ) -> Option<MidiClip> {
        let clipId = format!("{}:part:{partIndex}", self.trackId(track));
        if matches!(track.kind, EnergyXtTrackKind::Unknown(_)) {
            return None;
        }
        match placed.kind {
            EnergyXtPartKind::Midi | EnergyXtPartKind::Drum => {}
            EnergyXtPartKind::Audio => {
                context.warning(
                    clipId,
                    DiagnosticCode::UnsupportedAudioPart,
                    format!("audio part '{}' is not mapped", placed.name),
                );
                return None;
            }
            EnergyXtPartKind::Unknown(code) => {
                context.warning(
                    clipId,
                    DiagnosticCode::UnsupportedPartKind,
                    format!(
                        "part kind {code} of '{}' is unknown and not mapped",
                        placed.name
                    ),
                );
                return None;
            }
        }
        if placed.isMuted() {
            context.warning(
                clipId,
                DiagnosticCode::MutedPartOmitted,
                format!(
                    "muted part '{}' does not play in energyXT and is omitted",
                    placed.name
                ),
            );
            return None;
        }
        if placed.length <= 0 {
            context.error(
                clipId,
                DiagnosticCode::InvalidClipRange,
                format!("part length {} ticks is not positive", placed.length),
            );
            return None;
        }

        let events = match placed.poolIndex {
            None => placed,
            Some(index) => {
                let Some(pooled) = source.pool.get(index) else {
                    context.error(
                        clipId,
                        DiagnosticCode::InvalidPoolReference,
                        format!(
                            "part refers to pool entry {index}, but the pool has {}",
                            source.pool.len()
                        ),
                    );
                    return None;
                };
                pooled
            }
        };
        if events.hasAutomation {
            context.warning(
                clipId.clone(),
                DiagnosticCode::UnsupportedClipAutomation,
                "part automation lanes are not mapped",
            );
        }

        let mut notes = Vec::new();
        self.mapMidiEvents(context, &clipId, placed, events, &mut notes);
        self.mapDrumHits(context, &clipId, placed, events, &mut notes);
        notes.sort_by(|left, right| {
            left.start
                .0
                .total_cmp(&right.start.0)
                .then(left.pitch.cmp(&right.pitch))
        });

        let start = self.beats(i64::from(placed.start));
        let length = self.beats(i64::from(placed.length));
        Some(MidiClip {
            id: clipId,
            name: if placed.name.is_empty() {
                events.name.clone()
            } else {
                placed.name.clone()
            },
            source: ClipSource::Arrangement {
                trackId: self.trackId(track),
                timelineRange: BeatRange {
                    start: Beat(start),
                    end: Beat(start + length),
                },
            },
            contentRange: BeatRange {
                start: Beat(0.0),
                end: Beat(length),
            },
            loopSettings: None,
            disabled: false,
            notes,
        })
    }

    /// Note-ons inside the placed part become notes; other messages are reported.
    fn mapMidiEvents(
        &self,
        context: &mut EnergyXtToInternalMappingContext,
        clipId: &str,
        placed: &EnergyXtPart,
        events: &EnergyXtPart,
        notes: &mut Vec<Note>,
    ) {
        let mut unsupported = 0usize;
        for (index, event) in events.midiEvents.iter().enumerate() {
            let at = placed.eventTime(event.tick);
            if !placed.contains(at) {
                continue; // silent in energyXT as well
            }
            if !event.isNoteOn() || event.data2 == 0 {
                unsupported += 1;
                continue;
            }
            let length = event.length.unwrap_or(0);
            if length == 0 || event.data1 > 127 {
                context.error(
                    format!("{clipId}:event:{index}"),
                    DiagnosticCode::InvalidNote,
                    format!("invalid note: key={}, length={length}", event.data1),
                );
                continue;
            }
            notes.push(Note {
                pitch: event.data1,
                start: Beat(self.beats(at - i64::from(placed.start))),
                duration: Beat(self.beats(i64::from(length))),
                velocity: f32::from(event.data2.min(127)),
                releaseVelocity: None,
                muted: false,
                probability: None,
            });
        }
        if unsupported > 0 {
            context.warning(
                clipId.to_owned(),
                DiagnosticCode::UnsupportedMidiMessage,
                format!(
                    "{unsupported} MIDI message(s) other than notes (controllers, pitch bend, \
                     program changes, pressure) are not mapped"
                ),
            );
        }
    }

    /// Drum hits become notes on key 36 + lane, with the hit volume as velocity.
    fn mapDrumHits(
        &self,
        context: &mut EnergyXtToInternalMappingContext,
        clipId: &str,
        placed: &EnergyXtPart,
        events: &EnergyXtPart,
        notes: &mut Vec<Note>,
    ) {
        let mut unsupportedValues = 0usize;
        for (index, hit) in events.drumHits.iter().enumerate() {
            let at = placed.eventTime(hit.tick);
            if !placed.contains(at) {
                continue;
            }
            let pitch = DRUM_BASE_PITCH + u16::from(hit.lane);
            if pitch > 127 || hit.length == 0 {
                context.error(
                    format!("{clipId}:hit:{index}"),
                    DiagnosticCode::InvalidNote,
                    format!("invalid drum hit: lane={}, length={}", hit.lane, hit.length),
                );
                continue;
            }
            if hit.value(DRUM_VALUE_PAN).is_some() || hit.value(DRUM_VALUE_CUTOFF).is_some() {
                unsupportedValues += 1;
            }
            // A missing volume is at its default of 1, full velocity.
            let velocity = hit.value(DRUM_VALUE_VOLUME).map_or(MAX_VELOCITY, |volume| {
                (volume * MAX_VELOCITY).round().clamp(1.0, MAX_VELOCITY)
            });
            notes.push(Note {
                pitch: pitch as u8,
                start: Beat(self.beats(at - i64::from(placed.start))),
                duration: Beat(self.beats(i64::from(hit.length))),
                velocity,
                releaseVelocity: None,
                muted: false,
                probability: None,
            });
        }
        if unsupportedValues > 0 {
            context.warning(
                clipId.to_owned(),
                DiagnosticCode::UnsupportedDrumHitValue,
                format!("{unsupportedValues} drum hit(s) carry pan or cutoff values, which are not mapped"),
            );
        }
    }

    fn beats(&self, ticks: i64) -> f64 {
        ticks as f64 / ENERGYXT_TICKS_PER_BEAT
    }
}
