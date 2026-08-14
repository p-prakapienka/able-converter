use std::io::Write;

use flate2::{Compression, write::GzEncoder};

use super::live::inspect_als;

fn gzip(xml: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(xml.as_bytes()).expect("write fixture");
    encoder.finish().expect("finish fixture")
}

#[test]
fn inspect_als_reads_tracks_clips_version_and_tempo() {
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
