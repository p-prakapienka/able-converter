use super::checksum::Checksum;

#[test]
fn matchesTheZeroInitialCrc32OfTheAsciiCheckString() {
    assert_eq!(Checksum::crc32ZeroInit(b"123456789"), 0xD202_D277);
}

#[test]
fn complementsAnEmptyMessage() {
    assert_eq!(Checksum::crc32ZeroInit(b""), 0xFFFF_FFFF);
}
