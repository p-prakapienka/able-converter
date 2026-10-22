use std::io::{Cursor, Write};

use zip::CompressionMethod;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::exporter::digitoneii::DigitoneIiExporter;
use crate::model::digitoneii::{
    CONTAINER_HEADER_LEN, CONTAINER_MAGIC, CONTAINER_PAIR, CONTENT_KIND_PROJECT,
    DEVICE_DIGITONE_II, DigitoneIiMachine, DigitoneIiManifest, DigitoneIiNoteLength,
    DigitoneIiProject, DigitoneIiTrigKind, DigitoneIiTrigPayload, IMAGE_SIZE_OS110, KIT_BASE,
    OBJECT_MAGIC, TAIL_BASE, TRIG_SLOT_COUNT, TRIG_SLOT_SIZE, noteLengthSixteenths,
};
use crate::parser::digitoneii::DigitoneIiParser;

use super::digitoneii::DigitoneIiReadError;

#[test]
fn roundTripsASyntheticProjectAndPreservesUnknownBytes() {
    let project = syntheticProject();
    let archive = DigitoneIiExporter::new()
        .exportProject(&project)
        .expect("archive");
    let parsed = DigitoneIiParser::new(Cursor::new(archive))
        .parse()
        .expect("parse archive");

    assert_eq!(parsed.image(), project.image());
    assert_eq!(parsed.projectName(), "MORNING");
    assert_eq!(parsed.identityToken(), 0x1122_3344);
    assert_eq!(parsed.projectObjectVersion(), Some(3));
    assert_eq!(parsed.manifest.payloadName, "MORNING");
    assert_eq!(parsed.manifest.firmwareVersion, "1.10E");
    assert!(parsed.container().compressed);

    let pattern = parsed.pattern(0).expect("pattern");
    assert!(pattern.recognised);
    assert_eq!(pattern.label, "A1");
    assert_eq!(pattern.trigs.len(), 1);
    assert_eq!(pattern.trigs[0].note, 60);
    assert_eq!(pattern.trigs[0].step, 1);
    assert_eq!(pattern.steps.len(), 1);
    assert_eq!(pattern.steps[0].kind, DigitoneIiTrigKind::Note);
    assert_eq!(pattern.steps[0].flags, 0x0191);
    assert_eq!(
        noteLengthSixteenths(pattern.trigs[0].noteLength),
        DigitoneIiNoteLength::Sixteenths(12)
    );

    let kit = parsed.kit(0).expect("kit");
    assert_eq!(kit.name, "KIT 1");
    assert_eq!(kit.trackLevels[0], 100);
    assert_eq!(kit.sounds[0].name, "PRESET 1");
    assert_eq!(kit.sounds[0].machine, Some(DigitoneIiMachine::FmTone));
    assert_eq!(kit.midiTrackMask, Some(0x0008));
    assert_eq!(parsed.image()[TAIL_BASE], 0x5A);

    let inspection = parsed.inspect();
    assert_eq!(inspection.patternsWithTrigs, 1);
    assert_eq!(inspection.trigCount, 1);
    assert_eq!(inspection.namedKitCount, 1);

    let mut renamed = parsed;
    renamed.setProjectName("EVENING");
    let again = DigitoneIiParser::new(Cursor::new(
        DigitoneIiExporter::new()
            .exportProject(&renamed)
            .expect("renamed archive"),
    ))
    .parse()
    .expect("parse renamed");
    assert_eq!(again.projectName(), "EVENING");
    assert_eq!(again.image()[TAIL_BASE], 0x5A);
    assert_eq!(again.pattern(0).expect("pattern").trigs[0].note, 60);
}

#[test]
fn readsADeflatedArchive() {
    let project = syntheticProject();
    let payload = DigitoneIiExporter::new()
        .exportPayload(&project)
        .expect("payload");
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    archive
        .start_file("manifest.json", options)
        .expect("manifest");
    archive
        .write_all(project.manifest.bytes())
        .expect("manifest bytes");
    archive
        .start_file("MORNING", options)
        .expect("payload entry");
    archive.write_all(&payload).expect("payload bytes");
    let bytes = archive.finish().expect("zip").into_inner();

    let parsed = DigitoneIiParser::new(Cursor::new(bytes))
        .parse()
        .expect("deflated project");
    assert_eq!(parsed.image(), project.image());
    assert!(parsed.container().compressed);
}

#[test]
fn rejectsAPayloadWhoseCheckFieldDoesNotMatch() {
    let project = syntheticProject();
    let mut payload = DigitoneIiExporter::new()
        .exportPayload(&project)
        .expect("payload");
    payload[CONTAINER_HEADER_LEN + 8] ^= 0x01;
    let bytes = zipWith(project.manifest.bytes(), &payload);
    let error = DigitoneIiParser::new(Cursor::new(bytes))
        .parse()
        .expect_err("bad check");
    assert!(matches!(
        error,
        DigitoneIiReadError::ChecksumMismatch { .. }
    ));
}

fn syntheticProject() -> DigitoneIiProject {
    let mut image = vec![0u8; IMAGE_SIZE_OS110];
    image[0..4].copy_from_slice(&OBJECT_MAGIC);
    image[4..8].copy_from_slice(&3u32.to_be_bytes());
    image[0x18..0x1C].copy_from_slice(&0x1122_3344u32.to_be_bytes());
    let pattern = 0x200;
    image[pattern..pattern + 4].copy_from_slice(&3u32.to_be_bytes());
    let trigs = pattern + 0x4A34;
    image[trigs..trigs + TRIG_SLOT_COUNT * TRIG_SLOT_SIZE].fill(0xFF);
    writeObject(&mut image, KIT_BASE, 3);
    writeObject(&mut image, KIT_BASE + 60, 2);
    image[KIT_BASE + 10_260..KIT_BASE + 10_262].copy_from_slice(&0x0008u16.to_be_bytes());
    image[TAIL_BASE] = 0x5A;

    let manifestJson = br#"{"FormatVersion":"1.0","ProductType":[],"Payload":"MORNING","FileType":"Project","FirmwareVersion":"1.10E"}"#;
    let manifest = DigitoneIiManifest::new(
        "1.0".to_owned(),
        Vec::new(),
        "MORNING".to_owned(),
        "Project".to_owned(),
        "1.10E".to_owned(),
        manifestJson.to_vec(),
    );
    let mut project = DigitoneIiProject::fromParts(manifest, header(), image);
    project.setProjectName("MORNING");
    project
        .insertNoteTrig(
            0,
            0,
            1,
            DigitoneIiTrigPayload {
                note: 60,
                velocity: 100,
                noteLength: 10,
                microTiming: 0,
            },
        )
        .expect("note trig");
    project.setKitName(0, "KIT 1").expect("kit name");
    project.setTrackLevel(0, 0, 100).expect("level");
    project.setSoundName(0, 0, "PRESET 1").expect("sound name");
    project
}

fn writeObject(image: &mut [u8], offset: usize, version: u32) {
    image[offset..offset + 4].copy_from_slice(&OBJECT_MAGIC);
    image[offset + 4..offset + 8].copy_from_slice(&version.to_be_bytes());
}

fn header() -> [u8; CONTAINER_HEADER_LEN] {
    let mut header = [0u8; CONTAINER_HEADER_LEN];
    header[0..4].copy_from_slice(&CONTAINER_MAGIC);
    header[4..8].copy_from_slice(&CONTAINER_PAIR);
    header[0x08] = DEVICE_DIGITONE_II;
    header[0x09..0x0D].copy_from_slice(b"0050");
    header[0x0D..0x11].copy_from_slice(&CONTENT_KIND_PROJECT.to_be_bytes());
    header[0x11..0x15].copy_from_slice(&3u32.to_be_bytes());
    header[0x19..0x1D]
        .copy_from_slice(&u32::try_from(IMAGE_SIZE_OS110).expect("size").to_be_bytes());
    header[0x1E] = 12;
    header
}

fn zipWith(manifest: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    archive
        .start_file("manifest.json", options)
        .expect("manifest");
    archive.write_all(manifest).expect("manifest bytes");
    archive.start_file("MORNING", options).expect("payload");
    archive.write_all(payload).expect("payload bytes");
    archive.finish().expect("zip").into_inner()
}
