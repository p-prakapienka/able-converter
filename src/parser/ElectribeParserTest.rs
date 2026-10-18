use crate::model::electribe::{ElectribeBeat, ElectribeFileKind};

use super::electribe::{ElectribeParser, ElectribeReadError};

const HEADER_SIZE: usize = 0x100;
const PATTERN_SIZE: usize = 0x4000;
const GLOBAL_SIZE: usize = 0x10000;
const PART_BASE: usize = 0x800;
const PART_SIZE: usize = 0x330;

fn fileHeader() -> Vec<u8> {
    let mut header = vec![0_u8; 0x24];
    header[..4].copy_from_slice(b"KORG");
    header[0x10..0x19].copy_from_slice(b"e2sampler");
    header[0x20] = 1;
    header.resize(HEADER_SIZE, 0xFF);
    header
}

fn emptyPattern(name: &str) -> Vec<u8> {
    let mut pattern = vec![0_u8; PATTERN_SIZE];
    pattern[..4].copy_from_slice(b"PTST");
    pattern[0x3BFC..0x3C00].copy_from_slice(b"PTED");
    pattern[0x10..0x10 + name.len()].copy_from_slice(name.as_bytes());
    pattern[0x22..0x24].copy_from_slice(&1205_u16.to_le_bytes());
    pattern
}

fn stepOffset(part: usize, step: usize) -> usize {
    PART_BASE + part * PART_SIZE + 0x30 + step * 12
}

fn synthesizedPattern() -> Vec<u8> {
    let mut pattern = emptyPattern("Synthetic   ");
    pattern[0x24] = (-12_i8).to_le_bytes()[0];
    pattern[0x25] = 1;
    pattern[0x26] = 3;
    pattern[0x27] = 9;
    pattern[0x28] = 2;
    pattern[0x2A] = 27;
    pattern[0x3C] = 1;
    pattern[0x3D] = 5;
    pattern[0x44] = 1;
    pattern[0x3B00..0x3B02].copy_from_slice(&7_u16.to_le_bytes());
    pattern[0x3B02] = 2;

    let part = PART_BASE + PART_SIZE;
    pattern[part] = 6;
    pattern[part + 1] = 1;
    pattern[part + 8..part + 10].copy_from_slice(&419_u16.to_le_bytes());
    pattern[part + 0x0C] = 7;
    pattern[part + 0x19] = (-20_i8).to_le_bytes()[0];
    pattern[part + 0x26] = 0xFF;

    let step = stepOffset(1, 17);
    pattern[step..step + 12].copy_from_slice(&[1, 0xFF, 110, 1, 0, 61, 0, 0, 0, 0, 0, 0]);

    pattern[0x100] = 2;
    pattern[0x118] = 4;
    pattern[0x130..0x170].fill(64);
    pattern[0x101] = 17;
    pattern[0x119] = 0x0F;
    pattern
}

#[test]
fn parsesSinglePatternFile() {
    let mut bytes = fileHeader();
    bytes.extend(synthesizedPattern());

    let project = ElectribeParser::new(bytes.as_slice())
        .parse()
        .expect("parse synthetic pattern");

    assert_eq!(project.fileKind, ElectribeFileKind::Pattern);
    assert_eq!(project.product, "e2sampler");
    assert_eq!(project.patterns.len(), 1);
    let pattern = &project.patterns[0];
    assert_eq!(pattern.number, 1);
    assert_eq!(pattern.name, "Synthetic");
    assert_eq!(pattern.tempoTenths, 1205);
    assert_eq!(pattern.swing, -12);
    assert_eq!(pattern.lengthBars, 2);
    assert_eq!(pattern.beat, ElectribeBeat::SixteenthTriplet);
    assert_eq!(pattern.key, 9);
    assert_eq!(pattern.levelRaw, 27);
    assert!(pattern.masterEffect.on);
    assert_eq!(pattern.masterEffect.effectType, 5);
    assert!(pattern.alternate1314);
    assert!(!pattern.alternate1516);
    assert_eq!(pattern.chainTo, 7);
    assert_eq!(pattern.chainRepeat, 2);
    assert_eq!(pattern.parts.len(), 16);
    assert_eq!(pattern.parts[0].steps.len(), 64);

    let part = &pattern.parts[1];
    assert_eq!(part.number, 2);
    assert_eq!(part.lastStep, 6);
    assert!(part.mute);
    assert_eq!(part.oscillator, 419);
    assert_eq!(part.filterType, 7);
    assert_eq!(part.pan, -20);
    assert_eq!(part.unknown26, 0xFF);

    let step = &part.steps[17];
    assert_eq!(step.index, 17);
    assert!(step.triggered);
    assert_eq!(step.gate, 0xFF);
    assert_eq!(step.velocity, 110);
    assert_eq!(step.unknownFlag, 1);
    assert_eq!(step.noteSlots, [0, 61, 0, 0, 0, 0, 0, 0]);

    assert_eq!(pattern.motionSlots.len(), 2);
    assert_eq!(pattern.motionSlots[0].part, 2);
    assert_eq!(pattern.motionSlots[0].parameterId, 4);
    assert_eq!(pattern.motionSlots[0].values, vec![64; 64]);
    assert_eq!(pattern.motionSlots[1].part, 17);
}

#[test]
fn parsesAllPatternFile() {
    let mut global = vec![0xFF_u8; GLOBAL_SIZE];
    global[..0x100].fill(0);
    global[..4].copy_from_slice(b"GLST");
    global[0xFC..0x100].copy_from_slice(b"GLED");

    let mut bytes = fileHeader();
    bytes.extend(global);
    for index in 0..250 {
        bytes.extend(emptyPattern(&format!("Pattern {}", index + 1)));
    }

    let project = ElectribeParser::new(bytes.as_slice())
        .parse()
        .expect("parse synthetic all-pattern file");

    assert_eq!(project.fileKind, ElectribeFileKind::AllPatterns);
    assert_eq!(project.patterns.len(), 250);
    assert_eq!(project.patterns[0].name, "Pattern 1");
    assert_eq!(project.patterns[249].number, 250);
    assert_eq!(project.patterns[249].name, "Pattern 250");
}

#[test]
fn rejectsFilesWithoutKorgHeader() {
    let error = ElectribeParser::new(&b"not an electribe file"[..])
        .parse()
        .expect_err("reject foreign file");

    assert!(matches!(error, ElectribeReadError::MissingFileHeader));
}

#[test]
fn rejectsOtherElectribeProducts() {
    let mut bytes = fileHeader();
    bytes[0x10..0x20].fill(0);
    bytes[0x10..0x19].copy_from_slice(b"electribe");
    bytes.extend(emptyPattern("Synth"));

    let error = ElectribeParser::new(bytes.as_slice())
        .parse()
        .expect_err("reject synth file");

    assert!(
        matches!(error, ElectribeReadError::UnsupportedProduct(product) if product == "electribe")
    );
}

#[test]
fn rejectsTruncatedPattern() {
    let mut bytes = fileHeader();
    bytes.extend(&emptyPattern("Short")[..0x2000]);

    let error = ElectribeParser::new(bytes.as_slice())
        .parse()
        .expect_err("reject truncated pattern");

    assert!(matches!(
        error,
        ElectribeReadError::Truncated {
            expected: 0x4100,
            actual: 0x2100,
            ..
        }
    ));
}

#[test]
fn rejectsPatternWithoutEndMarker() {
    let mut pattern = emptyPattern("Broken");
    pattern[0x3BFC..0x3C00].fill(0);
    let mut bytes = fileHeader();
    bytes.extend(pattern);

    let error = ElectribeParser::new(bytes.as_slice())
        .parse()
        .expect_err("reject pattern without PTED");

    assert!(matches!(
        error,
        ElectribeReadError::InvalidPatternMarkers { number: 1 }
    ));
}

#[test]
fn rejectsUnknownBeatValue() {
    let mut pattern = emptyPattern("Odd beat");
    pattern[0x26] = 9;
    let mut bytes = fileHeader();
    bytes.extend(pattern);

    let error = ElectribeParser::new(bytes.as_slice())
        .parse()
        .expect_err("reject unknown beat");

    assert!(matches!(
        error,
        ElectribeReadError::InvalidPatternValue {
            field: "beat",
            value: 9,
            ..
        }
    ));
}
