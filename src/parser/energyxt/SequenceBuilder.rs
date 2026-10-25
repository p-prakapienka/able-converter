use crate::model::energyxt::{
    EnergyXtDrumHit, EnergyXtMidiEvent, EnergyXtPart, EnergyXtPartKind, EnergyXtProject,
    EnergyXtTrack, EnergyXtTrackKind,
};

use super::bytecursor::ByteCursor;
use super::energyxtreaderror::EnergyXtReadError;
use super::xtrecord::XtRecord;

const DEFAULT_PART_COLOR: u32 = 0xFFFF_FFFF;

/// Sequencer records that place things on the timeline but are not mapped.
const UNMAPPED_TIMELINE_KEYS: [&str; 6] = [
    "tempo", "timesig", "marker", "amarker", "pattern", "apattern",
];

/// Where the most recent part record went, so its event records can follow it.
#[derive(Debug, Clone, Copy)]
enum CurrentPart {
    None,
    Pool(usize),
    Placed(usize, usize),
}

/// Builds tracks and parts from the sequencer module's flat child records:
/// track settings first, then the part pool, then each track's placed parts
/// after a `ptrack` marker.
pub(super) struct SequenceBuilder {
    tracks: Vec<EnergyXtTrack>,
    pool: Vec<EnergyXtPart>,
    unmappedRecords: Vec<String>,
    placedTracks: usize,
    current: CurrentPart,
}

impl SequenceBuilder {
    pub(super) fn new() -> Self {
        Self {
            tracks: Vec::new(),
            pool: Vec::new(),
            unmappedRecords: Vec::new(),
            placedTracks: 0,
            current: CurrentPart::None,
        }
    }

    pub(super) fn onRecord(&mut self, record: XtRecord<'_>) -> Result<(), EnergyXtReadError> {
        match record.key {
            "track" => self.onTrack(record.payload),
            "trackCol" => self.onTrackColor(record.payload),
            "ptrack" => self.onPlacedTrackMarker(),
            "part" => self.onPart(record.payload),
            "partCol" => self.onPartColor(record.payload),
            "mevents" => self.onMidiEvents(record.payload),
            "devents" => self.onDrumHits(record.payload),
            "achs" | "aevents2" | "aevents" => {
                self.currentPart("audio events")?.hasAudioEvents = true;
                Ok(())
            }
            "amevents" => {
                self.currentPart("amevents")?.hasAutomation = true;
                Ok(())
            }
            key if UNMAPPED_TIMELINE_KEYS.contains(&key) => {
                self.unmappedRecords.push(key.to_owned());
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub(super) fn finish(self, tempo: Option<f64>) -> EnergyXtProject {
        EnergyXtProject {
            tempo,
            tracks: self.tracks,
            pool: self.pool,
            unmappedRecords: self.unmappedRecords,
        }
    }

    fn onTrack(&mut self, payload: &[u8]) -> Result<(), EnergyXtReadError> {
        let mut cursor = ByteCursor::new(payload, "track");
        let kind = EnergyXtTrackKind::fromCode(cursor.u32()?);
        let _flags = cursor.u32()?;
        let name = cursor.string()?;
        let _height = cursor.i32()?;
        let _expandedHeight = cursor.i32()?;
        let _editorScroll = cursor.i32()?;
        let parent = cursor.i32()?;
        let _overlayControllers = cursor.u8()?;
        let midiChannel = cursor.u8()? as i8;
        self.tracks.push(EnergyXtTrack {
            index: self.tracks.len(),
            kind,
            name,
            color: None,
            parent: usize::try_from(parent).ok(),
            midiChannel,
            parts: Vec::new(),
        });
        Ok(())
    }

    fn onTrackColor(&mut self, payload: &[u8]) -> Result<(), EnergyXtReadError> {
        let mut cursor = ByteCursor::new(payload, "trackCol");
        let color = cursor.u32()?;
        let track = self
            .tracks
            .last_mut()
            .ok_or(EnergyXtReadError::UnexpectedRecord { record: "trackCol" })?;
        track.color = Some(color);
        Ok(())
    }

    fn onPlacedTrackMarker(&mut self) -> Result<(), EnergyXtReadError> {
        if self.placedTracks >= self.tracks.len() {
            return Err(EnergyXtReadError::InvalidRecord {
                record: "ptrack",
                reason: format!("more markers than the {} tracks", self.tracks.len()),
            });
        }
        self.placedTracks += 1;
        self.current = CurrentPart::None;
        Ok(())
    }

    fn onPart(&mut self, payload: &[u8]) -> Result<(), EnergyXtReadError> {
        let mut cursor = ByteCursor::new(payload, "part");
        let _version = cursor.u8()?;
        let kind = EnergyXtPartKind::fromCode(cursor.u8()?);
        let _placed = cursor.u8()?;
        let poolIndex = usize::try_from(cursor.i32()?).ok();
        let start = cursor.i32()?;
        let offset = cursor.i32()?;
        let length = cursor.i32()?;
        let flags = cursor.i32()?;
        let name = cursor.string()?;
        cursor.expectEnd()?;

        let part = EnergyXtPart {
            kind,
            poolIndex,
            start,
            offset,
            length,
            flags,
            name,
            color: None,
            midiEvents: Vec::new(),
            drumHits: Vec::new(),
            hasAudioEvents: false,
            hasAutomation: false,
        };
        if self.placedTracks == 0 {
            self.pool.push(part);
            self.current = CurrentPart::Pool(self.pool.len() - 1);
        } else {
            let trackIndex = self.placedTracks - 1;
            let parts = &mut self.tracks[trackIndex].parts;
            parts.push(part);
            self.current = CurrentPart::Placed(trackIndex, parts.len() - 1);
        }
        Ok(())
    }

    fn onPartColor(&mut self, payload: &[u8]) -> Result<(), EnergyXtReadError> {
        let mut cursor = ByteCursor::new(payload, "partCol");
        let color = cursor.u32()?;
        self.currentPart("partCol")?.color = (color != DEFAULT_PART_COLOR).then_some(color);
        Ok(())
    }

    fn onMidiEvents(&mut self, payload: &[u8]) -> Result<(), EnergyXtReadError> {
        let mut cursor = ByteCursor::new(payload, "mevents");
        let count = cursor.u32()?;
        let mut events = Vec::new();
        let mut tick = 0u32;
        for _ in 0..count {
            tick = tick.saturating_add(cursor.u32()?);
            let _flag = cursor.u8()?;
            let message = cursor.u32()?;
            let [status, data1, data2, _] = message.to_le_bytes();
            let mut event = EnergyXtMidiEvent {
                tick,
                status,
                data1,
                data2,
                length: None,
            };
            if event.isNoteOn() {
                event.length = Some(cursor.u32()?);
            }
            events.push(event);
        }
        cursor.expectEnd()?;
        self.currentPart("mevents")?.midiEvents = events;
        Ok(())
    }

    fn onDrumHits(&mut self, payload: &[u8]) -> Result<(), EnergyXtReadError> {
        let mut cursor = ByteCursor::new(payload, "devents");
        let count = cursor.u32()?;
        let mut hits = Vec::new();
        let mut tick = 0u32;
        for _ in 0..count {
            tick = tick.saturating_add(cursor.u32()?);
            let lane = cursor.u8()?;
            let _reserved = cursor.u8()?;
            let length = cursor.u32()?;
            let valueCount = cursor.u8()?;
            let mut values = Vec::new();
            for _ in 0..valueCount {
                let id = cursor.u8()?;
                values.push((id, cursor.f32()?));
            }
            hits.push(EnergyXtDrumHit {
                tick,
                lane,
                length,
                values,
            });
        }
        cursor.expectEnd()?;
        self.currentPart("devents")?.drumHits = hits;
        Ok(())
    }

    fn currentPart(
        &mut self,
        record: &'static str,
    ) -> Result<&mut EnergyXtPart, EnergyXtReadError> {
        match self.current {
            CurrentPart::Pool(index) => Ok(&mut self.pool[index]),
            CurrentPart::Placed(track, index) => Ok(&mut self.tracks[track].parts[index]),
            CurrentPart::None => Err(EnergyXtReadError::UnexpectedRecord { record }),
        }
    }
}
