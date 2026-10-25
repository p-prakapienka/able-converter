use crate::model::internal::Note;

/// A stretch of one track's timeline that becomes one Session clip: a whole
/// Arrangement clip, or one piece of a clip too long for a slot.
#[derive(Debug, Clone)]
pub(super) struct LayoutUnit {
    pub(super) sourceClipId: String,
    pub(super) name: String,
    /// Timeline position in beats.
    pub(super) start: f64,
    pub(super) end: f64,
    /// Notes relative to `start`, sorted.
    pub(super) notes: Vec<Note>,
}

impl LayoutUnit {
    pub(super) fn length(&self) -> f64 {
        self.end - self.start
    }

    /// Identifies units with the same length and notes, which can share a clip.
    pub(super) fn contentKey(&self) -> Vec<u64> {
        let mut key = vec![self.length().to_bits()];
        for note in &self.notes {
            key.extend([
                u64::from(note.pitch),
                note.start.0.to_bits(),
                note.duration.0.to_bits(),
                u64::from(note.velocity.to_bits()),
                note.releaseVelocity
                    .map_or(u64::MAX, |value| u64::from(value.to_bits())),
                u64::from(note.muted),
                note.probability
                    .map_or(u64::MAX, |value| u64::from(value.to_bits())),
            ]);
        }
        key
    }
}
