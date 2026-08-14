//! Streaming inspection of gzip-compressed Ableton Live Set (`.als`) files.

use std::io::{BufReader, Read};

use flate2::read::GzDecoder;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use thiserror::Error;

use crate::model::{ClipCounts, LiveFormatVersion, LiveSetInspection, TrackCounts};

#[derive(Debug, Error)]
pub enum LiveReadError {
    #[error("failed to read Ableton Live XML: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("an XML attribute could not be decoded as UTF-8: {0}")]
    InvalidAttribute(#[from] std::str::Utf8Error),
    #[error("the file does not contain an Ableton root element")]
    MissingAbletonRoot,
}

/// Inspect an `.als` stream without materialising the decompressed XML on disk.
///
/// The source must start with a GZIP member. Decompression failures are surfaced
/// through the XML reader as [`LiveReadError::Xml`].
pub fn inspect_als<R: Read>(source: R) -> Result<LiveSetInspection, LiveReadError> {
    let decoder = GzDecoder::new(BufReader::new(source));
    inspect_xml(BufReader::new(decoder))
}

fn inspect_xml<R: std::io::BufRead>(source: R) -> Result<LiveSetInspection, LiveReadError> {
    let mut reader = Reader::from_reader(source);
    reader.config_mut().trim_text(true);

    let mut buffer = Vec::new();
    let mut format = None;
    let mut tempo = None;
    let mut tracks = TrackCounts::default();
    let mut clips = ClipCounts::default();
    let mut session_depth = 0_usize;
    let mut arrangement_depth = 0_usize;
    let mut tempo_depth = 0_usize;

    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                let name = element.name();
                let name = name.as_ref();
                on_element(
                    name,
                    &element,
                    &mut format,
                    &mut tempo,
                    &mut tracks,
                    &mut clips,
                    session_depth,
                    arrangement_depth,
                    tempo_depth,
                )?;

                match name {
                    b"ClipSlotList" => session_depth += 1,
                    b"ArrangerAutomation" => arrangement_depth += 1,
                    b"Tempo" => tempo_depth += 1,
                    _ => {}
                }
            }
            Event::Empty(element) => {
                let name = element.name();
                on_element(
                    name.as_ref(),
                    &element,
                    &mut format,
                    &mut tempo,
                    &mut tracks,
                    &mut clips,
                    session_depth,
                    arrangement_depth,
                    tempo_depth,
                )?;
            }
            Event::End(element) => match element.name().as_ref() {
                b"ClipSlotList" => session_depth = session_depth.saturating_sub(1),
                b"ArrangerAutomation" => {
                    arrangement_depth = arrangement_depth.saturating_sub(1);
                }
                b"Tempo" => tempo_depth = tempo_depth.saturating_sub(1),
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }

        buffer.clear();
    }

    Ok(LiveSetInspection {
        format: format.ok_or(LiveReadError::MissingAbletonRoot)?,
        tempo,
        tracks,
        clips,
    })
}

#[allow(clippy::too_many_arguments)]
fn on_element(
    name: &[u8],
    element: &BytesStart<'_>,
    format: &mut Option<LiveFormatVersion>,
    tempo: &mut Option<f64>,
    tracks: &mut TrackCounts,
    clips: &mut ClipCounts,
    session_depth: usize,
    arrangement_depth: usize,
    tempo_depth: usize,
) -> Result<(), LiveReadError> {
    match name {
        b"Ableton" if format.is_none() => {
            *format = Some(LiveFormatVersion {
                major: attribute(element, b"MajorVersion")?,
                minor: attribute(element, b"MinorVersion")?,
                creator: attribute(element, b"Creator")?,
                revision: attribute(element, b"Revision")?,
            });
        }
        b"MidiTrack" => tracks.midi += 1,
        b"AudioTrack" => tracks.audio += 1,
        b"GroupTrack" => tracks.group += 1,
        b"ReturnTrack" => tracks.return_tracks += 1,
        b"MasterTrack" | b"MainTrack" => tracks.main += 1,
        b"MidiClip" => classify_clip(
            &mut clips.session_midi,
            &mut clips.arrangement_midi,
            &mut clips.unclassified_midi,
            session_depth,
            arrangement_depth,
        ),
        b"AudioClip" => classify_clip(
            &mut clips.session_audio,
            &mut clips.arrangement_audio,
            &mut clips.unclassified_audio,
            session_depth,
            arrangement_depth,
        ),
        b"Manual" if tempo_depth > 0 && tempo.is_none() => {
            *tempo = attribute(element, b"Value")?.and_then(|value| value.parse().ok());
        }
        _ => {}
    }

    Ok(())
}

fn classify_clip(
    session: &mut usize,
    arrangement: &mut usize,
    unclassified: &mut usize,
    session_depth: usize,
    arrangement_depth: usize,
) {
    if session_depth > 0 {
        *session += 1;
    } else if arrangement_depth > 0 {
        *arrangement += 1;
    } else {
        *unclassified += 1;
    }
}

fn attribute(element: &BytesStart<'_>, key: &[u8]) -> Result<Option<String>, LiveReadError> {
    for attribute in element.attributes().with_checks(false).flatten() {
        if attribute.key.as_ref() == key {
            return Ok(Some(
                std::str::from_utf8(attribute.value.as_ref())?.to_owned(),
            ));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::{Compression, write::GzEncoder};

    use super::inspect_als;

    fn gzip(xml: &str) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(xml.as_bytes()).expect("write fixture");
        encoder.finish().expect("finish fixture")
    }

    #[test]
    fn inspects_tracks_clips_version_and_tempo() {
        let xml = r#"
            <Ableton MajorVersion="5" MinorVersion="12.0_123"
                     Creator="Ableton Live 12.1" Revision="abc123">
              <LiveSet>
                <Tracks>
                  <MidiTrack>
                    <DeviceChain><MainSequencer>
                      <ClipSlotList><ClipSlot><Value><MidiClip /></Value></ClipSlot></ClipSlotList>
                      <ArrangerAutomation><Events><MidiClip /></Events></ArrangerAutomation>
                    </MainSequencer></DeviceChain>
                  </MidiTrack>
                  <AudioTrack>
                    <DeviceChain><MainSequencer>
                      <ArrangerAutomation><Events><AudioClip /></Events></ArrangerAutomation>
                    </MainSequencer></DeviceChain>
                  </AudioTrack>
                  <GroupTrack />
                  <ReturnTrack />
                </Tracks>
                <MasterTrack><DeviceChain><Mixer><Tempo><Manual Value="123.5" /></Tempo></Mixer></DeviceChain></MasterTrack>
              </LiveSet>
            </Ableton>
        "#;

        let result = inspect_als(gzip(xml).as_slice()).expect("inspect fixture");

        assert_eq!(result.format.major.as_deref(), Some("5"));
        assert_eq!(result.format.minor.as_deref(), Some("12.0_123"));
        assert_eq!(result.format.creator.as_deref(), Some("Ableton Live 12.1"));
        assert_eq!(result.tempo, Some(123.5));
        assert_eq!(result.tracks.midi, 1);
        assert_eq!(result.tracks.audio, 1);
        assert_eq!(result.tracks.group, 1);
        assert_eq!(result.tracks.return_tracks, 1);
        assert_eq!(result.tracks.main, 1);
        assert_eq!(result.clips.session_midi, 1);
        assert_eq!(result.clips.arrangement_midi, 1);
        assert_eq!(result.clips.arrangement_audio, 1);
    }

    #[test]
    fn rejects_non_ableton_xml() {
        let error = inspect_als(gzip("<not-ableton />").as_slice()).expect_err("reject fixture");
        assert!(error.to_string().contains("Ableton root"));
    }
}
