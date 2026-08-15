use std::io::Write;

use flate2::{Compression, write::GzEncoder};

use super::live::LiveParser;

fn gzip(xml: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(xml.as_bytes()).expect("write fixture");
    encoder.finish().expect("finish fixture")
}

#[test]
fn parserInspectsTracksClipsVersionAndTempo() {
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

    let result = LiveParser::new(gzip(xml).as_slice())
        .inspect()
        .expect("inspect fixture");

    assert_eq!(result.format.major.as_deref(), Some("5"));
    assert_eq!(result.format.minor.as_deref(), Some("12.0_123"));
    assert_eq!(result.format.creator.as_deref(), Some("Ableton Live 12.1"));
    assert_eq!(result.tempo, Some(123.5));
    assert_eq!(result.tracks.midi, 1);
    assert_eq!(result.tracks.audio, 1);
    assert_eq!(result.tracks.group, 1);
    assert_eq!(result.tracks.returnTracks, 1);
    assert_eq!(result.tracks.main, 1);
    assert_eq!(result.clips.sessionMidi, 1);
    assert_eq!(result.clips.arrangementMidi, 1);
    assert_eq!(result.clips.arrangementAudio, 1);
}

#[test]
fn rejectsNonAbletonXml() {
    let error = LiveParser::new(gzip("<not-ableton />").as_slice())
        .inspect()
        .expect_err("reject fixture");
    assert!(error.to_string().contains("Ableton root"));
}

#[test]
fn parserReadsTrackNamesAndExplicitSceneMetadata() {
    let xml = r#"
        <Ableton MajorVersion="5" MinorVersion="12.0_123" Creator="Ableton Live 12">
          <LiveSet>
            <Tracks>
              <MidiTrack Id="42">
                <Name>
                  <EffectiveName Value="1-MIDI" />
                  <UserName Value="Bass" />
                </Name>
                <Color Value="10" />
                <DeviceChain><Instrument><Name Value="Must not replace track name" /></Instrument></DeviceChain>
              </MidiTrack>
              <AudioTrack Id="43">
                <Name>
                  <EffectiveName Value="2-Audio" />
                  <UserName Value="" />
                </Name>
                <Color Value="11" />
              </AudioTrack>
            </Tracks>
            <Scenes>
              <Scene Id="9">
                <Name Value="Intro" />
                <Color Value="5" />
                <Tempo Value="128" />
                <IsTempoEnabled Value="true" />
                <TimeSignatureId Value="201" />
                <IsTimeSignatureEnabled Value="true" />
              </Scene>
              <Scene Id="12">
                <Name Value="Verse" />
                <ColorIndex Value="6" />
                <Tempo Value="120" />
                <TempoEnabled Value="false" />
                <TimeSignatureId Value="202" />
                <TimeSignatureEnabled Value="false" />
              </Scene>
            </Scenes>
          </LiveSet>
        </Ableton>
    "#;

    let project = LiveParser::new(gzip(xml).as_slice())
        .parse()
        .expect("parse metadata fixture");

    assert_eq!(project.tracks.len(), 2);
    assert_eq!(project.tracks[0].id, "42");
    assert_eq!(project.tracks[0].effectiveName, "1-MIDI");
    assert_eq!(project.tracks[0].userName, "Bass");
    assert_eq!(project.tracks[0].color, Some(10));
    assert_eq!(project.tracks[1].effectiveName, "2-Audio");
    assert_eq!(project.tracks[1].userName, "");

    assert_eq!(project.scenes.len(), 2);
    assert_eq!(project.scenes[0].id, "9");
    assert_eq!(project.scenes[0].index, 0);
    assert_eq!(project.scenes[0].name, "Intro");
    assert_eq!(project.scenes[0].color, Some(5));
    assert_eq!(project.scenes[0].tempo, Some(128.0));
    assert!(project.scenes[0].tempoEnabled);
    assert_eq!(project.scenes[0].timeSignatureId, Some(201));
    assert!(project.scenes[0].timeSignatureEnabled);
    assert_eq!(project.scenes[1].index, 1);
    assert_eq!(project.scenes[1].color, Some(6));
    assert!(!project.scenes[1].tempoEnabled);
    assert!(!project.scenes[1].timeSignatureEnabled);
}

#[test]
fn parserReadsSessionClipNotesAndLoop() {
    let xml = r#"
        <Ableton MajorVersion="5" MinorVersion="11.0_11300" Creator="Ableton Live 11">
          <LiveSet>
            <Tracks>
              <MidiTrack Id="42">
                <DeviceChain><MainSequencer>
                  <ClipSlotList>
                    <ClipSlot Id="3">
                      <ClipSlot><Value>
                        <MidiClip Id="7" Time="0">
                          <CurrentStart Value="0" />
                          <CurrentEnd Value="4" />
                          <Loop>
                            <LoopStart Value="0" />
                            <LoopEnd Value="4" />
                            <StartRelative Value="0" />
                            <LoopOn Value="true" />
                          </Loop>
                          <Name Value="Bass &amp; Lead" />
                          <Disabled Value="false" />
                          <Envelopes><Envelopes><AutomationEnvelope /></Envelopes></Envelopes>
                          <Notes>
                            <KeyTracks>
                              <KeyTrack Id="0">
                                <Notes>
                                  <MidiNoteEvent Time="1.5" Duration="0.5" Velocity="75.5"
                                      VelocityDeviation="2" OffVelocity="64" Probability="0.75"
                                      IsEnabled="false" NoteId="23" />
                                </Notes>
                                <MidiKey Value="43" />
                              </KeyTrack>
                              <KeyTrack Id="1">
                                <Notes>
                                  <MidiNoteEvent Time="0" Duration="0.25" Velocity="100" NoteId="24" />
                                </Notes>
                                <MidiKey Value="60" />
                              </KeyTrack>
                            </KeyTracks>
                            <PerNoteEventStore><EventLists><PerNoteEvent /></EventLists></PerNoteEventStore>
                          </Notes>
                        </MidiClip>
                      </Value></ClipSlot>
                    </ClipSlot>
                  </ClipSlotList>
                  <ClipTimeable><ArrangerAutomation><Events>
                    <MidiClip Id="8" Time="4" />
                  </Events></ArrangerAutomation></ClipTimeable>
                </MainSequencer></DeviceChain>
              </MidiTrack>
            </Tracks>
          </LiveSet>
        </Ableton>
    "#;

    let project = LiveParser::new(gzip(xml).as_slice())
        .parse()
        .expect("parse fixture");

    assert_eq!(project.inspection.clips.sessionMidi, 1);
    assert_eq!(project.inspection.clips.arrangementMidi, 1);
    assert_eq!(project.sessionMidiClips.len(), 1);
    let clip = &project.sessionMidiClips[0];
    assert_eq!(clip.id, "7");
    assert_eq!(clip.trackId, "42");
    assert_eq!(clip.sceneIndex, 3);
    assert_eq!(clip.name, "Bass & Lead");
    assert_eq!(clip.currentStart, 0.0);
    assert_eq!(clip.currentEnd, 4.0);
    assert!(clip.loopSettings.expect("loop").enabled);
    assert!(clip.hasClipAutomation);
    assert!(clip.hasPerNoteExpression);
    assert_eq!(clip.notes.len(), 2);
    assert_eq!(clip.notes[0].pitch, 43);
    assert_eq!(clip.notes[0].velocity, 75.5);
    assert_eq!(clip.notes[0].probability, Some(0.75));
    assert_eq!(clip.notes[0].enabled, Some(false));
    assert_eq!(clip.notes[1].pitch, 60);
    assert_eq!(clip.notes[1].enabled, None);
}

#[test]
fn parserRejectsIncompleteSessionClip() {
    let xml = r#"
        <Ableton Creator="Ableton Live 12">
          <LiveSet><Tracks><MidiTrack Id="42"><DeviceChain><MainSequencer>
            <ClipSlotList><ClipSlot Id="0"><ClipSlot><Value>
              <MidiClip Id="7"><CurrentStart Value="0" /></MidiClip>
            </Value></ClipSlot></ClipSlot></ClipSlotList>
          </MainSequencer></DeviceChain></MidiTrack></Tracks></LiveSet>
        </Ableton>
    "#;

    let error = LiveParser::new(gzip(xml).as_slice())
        .parse()
        .expect_err("reject incomplete clip");

    assert!(error.to_string().contains("CurrentEnd"));
}
