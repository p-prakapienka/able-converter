use crate::model::caustic::CausticTransport;

use super::bytecursor::ByteCursor;
use super::causticreaderror::CausticReadError;

/// Offset of the tempo value inside the `OUTP` payload.
///
/// Provisional: taken from existing third-party decoders and not yet confirmed
/// against `OutputPanel::Serialize` in the engine binary.
const TEMPO_OFFSET: usize = 82;

/// Bytes required to read both transport values.
const TRANSPORT_LENGTH: usize = TEMPO_OFFSET + 5;

/// Reads the transport values carried by the `OUTP` payload.
#[derive(Default)]
pub(super) struct TransportParser;

impl TransportParser {
    pub(super) fn new() -> Self {
        Self
    }

    /// Read tempo and bar length, leaving both unset when the payload is too
    /// short rather than guessing at a different layout.
    pub(super) fn parse(&self, payload: &[u8]) -> Result<CausticTransport, CausticReadError> {
        if payload.len() < TRANSPORT_LENGTH {
            return Ok(CausticTransport::default());
        }

        let mut cursor = ByteCursor::new(payload);
        cursor.readBytes("output panel", TEMPO_OFFSET)?;
        let tempo = cursor.readF32("output panel")?;
        let beatsPerBar = cursor.readBytes("output panel", 1)?[0];

        Ok(CausticTransport {
            tempo: Some(tempo),
            beatsPerBar: Some(beatsPerBar),
        })
    }
}
