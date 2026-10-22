use super::digitoneii::{
    DigitoneIiMachine, DigitoneIiNoteLength, conditionLabel, noteLengthSixteenths, patternLabel,
};

#[test]
fn namesPatternsTheWayTheDeviceDoes() {
    assert_eq!(patternLabel(0), "A1");
    assert_eq!(patternLabel(15), "A16");
    assert_eq!(patternLabel(16), "B1");
    assert_eq!(patternLabel(97), "G2");
}

#[test]
fn decodesTheVerifiedNoteLengthBands() {
    assert_eq!(noteLengthSixteenths(0), DigitoneIiNoteLength::Sixteenths(2));
    assert_eq!(
        noteLengthSixteenths(29),
        DigitoneIiNoteLength::Sixteenths(31)
    );
    assert_eq!(
        noteLengthSixteenths(30),
        DigitoneIiNoteLength::Sixteenths(32)
    );
    assert_eq!(
        noteLengthSixteenths(45),
        DigitoneIiNoteLength::Sixteenths(62)
    );
    assert_eq!(
        noteLengthSixteenths(110),
        DigitoneIiNoteLength::Sixteenths(1024)
    );
    assert_eq!(
        noteLengthSixteenths(126),
        DigitoneIiNoteLength::Sixteenths(2048)
    );
    assert_eq!(noteLengthSixteenths(127), DigitoneIiNoteLength::Infinite);
    assert_eq!(noteLengthSixteenths(255), DigitoneIiNoteLength::Inherit);
    assert_eq!(
        noteLengthSixteenths(128),
        DigitoneIiNoteLength::Unrecognised(128)
    );
}

#[test]
fn labelsOnlyConditionCodesThatWereMeasured() {
    assert_eq!(conditionLabel(0x08), Some("1:2"));
    assert_eq!(conditionLabel(0x09), Some("2:2"));
    assert_eq!(conditionLabel(0x12), Some("2:4"));
    assert_eq!(conditionLabel(0x4B), Some("not 8:8"));
    assert_eq!(conditionLabel(0x2E), None);
}

#[test]
fn doesNotNameAnUnrecognisedMachineByte() {
    assert_eq!(DigitoneIiMachine::fromByte(4), DigitoneIiMachine::Midi);
    assert_eq!(
        DigitoneIiMachine::fromByte(5),
        DigitoneIiMachine::Unrecognised(5)
    );
}
