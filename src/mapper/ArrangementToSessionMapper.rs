//! Lays out canonical Arrangement clips as Session scenes and clip slots.
//!
//! 1. A clip longer than a slot allows is cut where other tracks change content,
//!    or every eight bars when they do not.
//! 2. The timeline is split into scene ranges at positions where no clip on any
//!    track is in the middle of playing. A range longer than a slot allows is
//!    split anyway, at the last clip edge that keeps it short enough.
//! 3. In each range a track plays one clip: the clip itself when the range holds
//!    that clip once or repeated back to back (it then loops), otherwise the
//!    track's content in the range rendered as a new clip.
//! 4. Ranges whose tracks play the same clips share one scene.
//!
//! See `docs/implementation-plan.md`.

use crate::model::internal::{
    Beat, BeatRange, ClipLoop, ClipSource, DiagnosticCode, MappingResult, MidiClip, Note, Project,
    Scene,
};
use crate::model::note::NOTE_CLIP_BEAT_LIMIT;

use super::arrangementtosessionlayoutsection::LayoutSection;
use super::arrangementtosessionlayoutunit::LayoutUnit;
use super::arrangementtosessionmappingcontext::ArrangementToSessionMappingContext;

const DEFAULT_BEATS_PER_BAR: f64 = 4.0;
const DEFAULT_FALLBACK_CUT_BARS: f64 = 8.0;

/// Reusable service that turns an Arrangement-only project into scenes.
#[derive(Debug, Clone, Copy)]
pub struct ArrangementToSessionMapper {
    clipBeatLimit: f64,
    beatsPerBar: f64,
    fallbackCutBars: f64,
}

impl Default for ArrangementToSessionMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl ArrangementToSessionMapper {
    /// Slots hold up to Note's clip length; long clips fall back to 8-bar pieces
    /// of 4/4.
    #[must_use]
    pub fn new() -> Self {
        Self {
            clipBeatLimit: NOTE_CLIP_BEAT_LIMIT,
            beatsPerBar: DEFAULT_BEATS_PER_BAR,
            fallbackCutBars: DEFAULT_FALLBACK_CUT_BARS,
        }
    }

    /// Bars per piece when a long clip has no other track's edges to follow.
    #[must_use]
    pub fn withFallbackCutBars(self, bars: f64) -> Self {
        Self {
            fallbackCutBars: bars,
            ..self
        }
    }

    #[must_use]
    pub fn map(&self, source: &Project) -> MappingResult<Project> {
        let mut context = ArrangementToSessionMappingContext::new();
        let hasArrangement = source
            .midiClips
            .iter()
            .any(|clip| matches!(clip.source, ClipSource::Arrangement { .. }));
        let hasSession = source
            .midiClips
            .iter()
            .any(|clip| matches!(clip.source, ClipSource::Session { .. }));
        if !hasArrangement {
            return context.finish(source.clone());
        }
        if hasSession {
            context.error(
                "project".to_owned(),
                DiagnosticCode::MixedSessionAndArrangement,
                "Session and Arrangement clips are never combined; select one of them first",
            );
            return context.finish(source.clone());
        }

        let wholeClips = self.collectUnits(&mut context, source);
        let changeEdges: Vec<Vec<f64>> = wholeClips
            .iter()
            .map(|units| self.changeEdges(units))
            .collect();
        let units: Vec<Vec<LayoutUnit>> = wholeClips
            .iter()
            .enumerate()
            .map(|(trackPosition, trackUnits)| {
                let otherEdges = self.otherTracksEdges(&changeEdges, trackPosition);
                trackUnits
                    .iter()
                    .flat_map(|unit| self.cutIfTooLong(&mut context, unit, &otherEdges))
                    .collect()
            })
            .collect();

        let sections = self.sections(&units);
        let mut slotUnits: Vec<Vec<LayoutUnit>> = vec![Vec::new(); units.len()];
        let mut scenes = Vec::new();
        let mut sceneKeys: Vec<Vec<Option<usize>>> = Vec::new();
        let mut sceneSlots: Vec<Vec<Option<usize>>> = Vec::new();

        for section in &sections {
            let mut keys = Vec::with_capacity(units.len());
            let mut slots = Vec::with_capacity(units.len());
            for (trackPosition, trackUnits) in units.iter().enumerate() {
                let trackId = &source.tracks[trackPosition].id;
                match self.slotContent(&mut context, trackId, trackUnits, section) {
                    Some(unit) => {
                        let index = self.shareClip(&mut slotUnits[trackPosition], unit);
                        keys.push(Some(index));
                        slots.push(Some(index));
                    }
                    None => {
                        keys.push(None);
                        slots.push(None);
                    }
                }
            }
            if sceneKeys.contains(&keys) {
                continue;
            }
            let index = scenes.len();
            scenes.push(Scene {
                id: format!("arrangement:scene:{index}"),
                index,
                name: format!("Bar {}", self.barNumber(section.start)),
                color: None,
                tempoOverride: None,
            });
            sceneKeys.push(keys);
            sceneSlots.push(slots);
        }

        let mut midiClips = Vec::new();
        for (sceneIndex, slots) in sceneSlots.iter().enumerate() {
            for (trackPosition, slot) in slots.iter().enumerate() {
                if let Some(clipIndex) = slot {
                    let trackId = source.tracks[trackPosition].id.clone();
                    midiClips.push(self.sessionClip(
                        &slotUnits[trackPosition][*clipIndex],
                        trackId,
                        sceneIndex,
                    ));
                }
            }
        }

        context.finish(Project {
            tempo: source.tempo,
            tracks: source.tracks.clone(),
            scenes,
            midiClips,
        })
    }

    /// One unit per Arrangement clip, grouped by track in track order and sorted.
    fn collectUnits(
        &self,
        context: &mut ArrangementToSessionMappingContext,
        source: &Project,
    ) -> Vec<Vec<LayoutUnit>> {
        let mut units: Vec<Vec<LayoutUnit>> = vec![Vec::new(); source.tracks.len()];
        for clip in &source.midiClips {
            let ClipSource::Arrangement {
                trackId,
                timelineRange,
            } = &clip.source
            else {
                continue;
            };
            let Some(trackPosition) = source.tracks.iter().position(|track| &track.id == trackId)
            else {
                context.error(
                    clip.id.clone(),
                    DiagnosticCode::UnknownTrackSelection,
                    format!("clip refers to unknown track {trackId}"),
                );
                continue;
            };
            if !timelineRange.isValid() || timelineRange.end.0 <= timelineRange.start.0 {
                context.error(
                    clip.id.clone(),
                    DiagnosticCode::InvalidClipRange,
                    "Arrangement clip has an empty or invalid timeline range",
                );
                continue;
            }
            if clip
                .loopSettings
                .is_some_and(|loopSettings| loopSettings.enabled)
            {
                context.warning(
                    clip.id.clone(),
                    DiagnosticCode::UnsupportedArrangementLoop,
                    "Arrangement clip loops are not expanded; the clip plays its content once",
                );
            }
            units[trackPosition].push(self.unitFromClip(clip, timelineRange));
        }
        for trackUnits in &mut units {
            trackUnits.sort_by(|left, right| left.start.total_cmp(&right.start));
        }
        units
    }

    fn unitFromClip(&self, clip: &MidiClip, timelineRange: &BeatRange) -> LayoutUnit {
        let contentStart = clip.contentRange.start.0;
        let length = timelineRange.end.0 - timelineRange.start.0;
        let notes = clip
            .notes
            .iter()
            .filter(|note| note.start.0 >= contentStart && note.start.0 < contentStart + length)
            .map(|note| Note {
                start: Beat(note.start.0 - contentStart),
                ..note.clone()
            })
            .collect();
        LayoutUnit {
            sourceClipId: clip.id.clone(),
            name: clip.name.clone(),
            start: timelineRange.start.0,
            end: timelineRange.end.0,
            notes: self.sortedNotes(notes),
        }
    }

    fn sortedNotes(&self, mut notes: Vec<Note>) -> Vec<Note> {
        notes.sort_by(|left, right| {
            left.start
                .0
                .total_cmp(&right.start.0)
                .then(left.pitch.cmp(&right.pitch))
                .then(left.duration.0.total_cmp(&right.duration.0))
        });
        notes
    }

    /// Where a track's content changes: the edges of each run of back-to-back
    /// units with identical content. A clip repeated bar after bar has edges only
    /// where the run starts and ends.
    fn changeEdges(&self, units: &[LayoutUnit]) -> Vec<f64> {
        let mut edges = Vec::new();
        let mut previous: Option<&LayoutUnit> = None;
        for unit in units {
            let continuesRun = previous.is_some_and(|before| {
                before.end == unit.start && before.contentKey() == unit.contentKey()
            });
            if !continuesRun {
                if let Some(before) = previous {
                    edges.push(before.end);
                }
                edges.push(unit.start);
            }
            previous = Some(unit);
        }
        if let Some(last) = previous {
            edges.push(last.end);
        }
        edges
    }

    fn otherTracksEdges(&self, changeEdges: &[Vec<f64>], trackPosition: usize) -> Vec<f64> {
        let mut edges: Vec<f64> = changeEdges
            .iter()
            .enumerate()
            .filter(|(position, _)| *position != trackPosition)
            .flat_map(|(_, trackEdges)| trackEdges.iter().copied())
            .collect();
        edges.sort_by(f64::total_cmp);
        edges.dedup();
        edges
    }

    /// Keeps a unit that fits a slot. A longer one is cut at the other tracks'
    /// change edges inside it, or every `fallbackCutBars` when there are none;
    /// any piece still too long is cut every `fallbackCutBars` as well.
    fn cutIfTooLong(
        &self,
        context: &mut ArrangementToSessionMappingContext,
        unit: &LayoutUnit,
        otherEdges: &[f64],
    ) -> Vec<LayoutUnit> {
        if unit.length() <= self.clipBeatLimit {
            return vec![unit.clone()];
        }
        let fallback = self.fallbackCutBars * self.beatsPerBar;
        let mut cuts: Vec<f64> = otherEdges
            .iter()
            .copied()
            .filter(|edge| *edge > unit.start && *edge < unit.end)
            .collect();
        if cuts.is_empty() {
            cuts = self.everyStep(unit.start, unit.end, fallback);
        }
        let mut bounds = vec![unit.start];
        bounds.extend(cuts);
        bounds.push(unit.end);
        let mut pieces = Vec::new();
        for pair in bounds.windows(2) {
            if pair[1] - pair[0] > self.clipBeatLimit {
                let mut inner = vec![pair[0]];
                inner.extend(self.everyStep(pair[0], pair[1], fallback));
                inner.push(pair[1]);
                pieces.extend(inner.windows(2).map(|range| (range[0], range[1])));
            } else {
                pieces.push((pair[0], pair[1]));
            }
        }

        let count = pieces.len();
        let mut shortened = 0usize;
        let mut result = Vec::with_capacity(count);
        for (index, (start, end)) in pieces.into_iter().enumerate() {
            let (notes, cutNotes) = self.notesWithin(std::slice::from_ref(unit), start, end);
            shortened += cutNotes;
            result.push(LayoutUnit {
                sourceClipId: unit.sourceClipId.clone(),
                name: format!("{} {}/{count}", unit.name, index + 1),
                start,
                end,
                notes,
            });
        }
        context.warning(
            unit.sourceClipId.clone(),
            DiagnosticCode::NotesShortenedAtCut,
            format!(
                "'{}' is {} beats long and was cut into {count} clips; {shortened} note(s) \
                 sounding across a cut were shortened",
                unit.name,
                unit.length()
            ),
        );
        result
    }

    fn everyStep(&self, start: f64, end: f64, step: f64) -> Vec<f64> {
        let mut points = Vec::new();
        let mut at = start + step;
        while at < end {
            points.push(at);
            at += step;
        }
        points
    }

    /// The notes of `units` that start in `[start, end)`, relative to `start`,
    /// with those ringing past `end` shortened. Returns how many were shortened.
    fn notesWithin(&self, units: &[LayoutUnit], start: f64, end: f64) -> (Vec<Note>, usize) {
        let mut shortened = 0usize;
        let mut notes = Vec::new();
        for unit in units {
            for note in &unit.notes {
                let at = unit.start + note.start.0;
                if at < start || at >= end {
                    continue;
                }
                let mut copy = note.clone();
                copy.start = Beat(at - start);
                if copy.duration.0 > end - at {
                    copy.duration = Beat(end - at);
                    shortened += 1;
                }
                notes.push(copy);
            }
        }
        (self.sortedNotes(notes), shortened)
    }

    /// Splits the timeline into scene ranges. A range ends where no unit is in
    /// the middle of playing. If that would make it longer than a slot allows,
    /// it ends at the last unit edge that fits, or at the limit itself.
    /// Stretches where every track is silent are skipped.
    fn sections(&self, units: &[Vec<LayoutUnit>]) -> Vec<LayoutSection> {
        let mut edges: Vec<f64> = units
            .iter()
            .flatten()
            .flat_map(|unit| [unit.start, unit.end])
            .collect();
        edges.sort_by(f64::total_cmp);
        edges.dedup();
        let Some(&last) = edges.last() else {
            return Vec::new();
        };
        let isClean = |edge: f64| {
            !units
                .iter()
                .flatten()
                .any(|unit| unit.start < edge && edge < unit.end)
        };

        let mut sections = Vec::new();
        let mut start = edges[0];
        while start < last {
            let limit = start + self.clipBeatLimit;
            let nextClean = edges
                .iter()
                .copied()
                .find(|edge| *edge > start && isClean(*edge))
                .unwrap_or(last);
            let end = if nextClean <= limit {
                nextClean
            } else {
                edges
                    .iter()
                    .copied()
                    .rev()
                    .find(|edge| *edge > start && *edge <= limit)
                    .unwrap_or(limit)
            };
            let audible = units
                .iter()
                .flatten()
                .any(|unit| unit.start < end && unit.end > start);
            if audible {
                sections.push(LayoutSection { start, end });
            }
            start = end;
        }
        sections
    }

    /// What one track plays in a scene range: nothing, one of its units when the
    /// range holds it once or repeated back to back, or the range's content
    /// rendered as a new unit.
    fn slotContent(
        &self,
        context: &mut ArrangementToSessionMappingContext,
        trackId: &str,
        units: &[LayoutUnit],
        section: &LayoutSection,
    ) -> Option<LayoutUnit> {
        let overlapping: Vec<LayoutUnit> = units
            .iter()
            .filter(|unit| unit.start < section.end && unit.end > section.start)
            .cloned()
            .collect();
        let first = overlapping.first()?;

        let firstKey = first.contentKey();
        let repeatsExactly = first.start == section.start
            && overlapping
                .last()
                .is_some_and(|last| last.end == section.end)
            && overlapping
                .windows(2)
                .all(|pair| pair[0].end == pair[1].start)
            && overlapping.iter().all(|unit| unit.contentKey() == firstKey);
        if repeatsExactly {
            return Some(first.clone());
        }

        let (notes, shortened) = self.notesWithin(&overlapping, section.start, section.end);
        if shortened > 0 {
            context.warning(
                format!("{trackId}:bar:{}", self.barNumber(section.end)),
                DiagnosticCode::NotesShortenedAtCut,
                format!(
                    "{shortened} note(s) sounding across the scene boundary at bar {} were \
                     shortened",
                    self.barNumber(section.end)
                ),
            );
        }
        let mut names: Vec<&str> = Vec::new();
        for unit in &overlapping {
            if !names.contains(&unit.name.as_str()) {
                names.push(&unit.name);
            }
        }
        Some(LayoutUnit {
            sourceClipId: first.sourceClipId.clone(),
            name: names.join(" + "),
            start: section.start,
            end: section.end,
            notes,
        })
    }

    /// Adds `unit` to the track's clips unless one with the same content exists.
    /// Returns the clip's index.
    fn shareClip(&self, clips: &mut Vec<LayoutUnit>, unit: LayoutUnit) -> usize {
        let key = unit.contentKey();
        if let Some(index) = clips.iter().position(|clip| clip.contentKey() == key) {
            return index;
        }
        clips.push(unit);
        clips.len() - 1
    }

    fn barNumber(&self, beat: f64) -> usize {
        (beat / self.beatsPerBar).floor().max(0.0) as usize + 1
    }

    fn sessionClip(&self, unit: &LayoutUnit, trackId: String, sceneIndex: usize) -> MidiClip {
        let region = BeatRange {
            start: Beat(0.0),
            end: Beat(unit.length()),
        };
        MidiClip {
            id: format!("{trackId}:scene:{sceneIndex}"),
            name: unit.name.clone(),
            source: ClipSource::Session {
                trackId,
                sceneIndex,
            },
            contentRange: region,
            loopSettings: Some(ClipLoop {
                region,
                startRelative: Beat(0.0),
                enabled: true,
            }),
            disabled: false,
            notes: unit.notes.clone(),
        }
    }
}
