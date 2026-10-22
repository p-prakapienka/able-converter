use crate::exporter::digitoneii::lz4blockencoder::Lz4BlockEncoder;

use super::lz4blockdecoder::Lz4BlockDecoder;

#[test]
fn roundTripsLiteralsAndARepeatedRun() {
    let mut image = b"hello ".repeat(20);
    image.extend(std::iter::repeat_n(0u8, 100));
    let decoded = Lz4BlockDecoder
        .decode(&Lz4BlockEncoder.encode(&image))
        .expect("valid chain");
    assert_eq!(decoded, image);
}

#[test]
fn roundTripsAMatchThatCrossesTheBlockBoundary() {
    let mut image = vec![0x11u8; 32_768];
    image.extend(std::iter::repeat_n(0x11u8, 64));
    image.extend_from_slice(&[0x22, 0x33, 0x44, 0x55]);
    let encoded = Lz4BlockEncoder.encode(&image);
    assert!(
        encoded.len() > 8,
        "the chain should contain more than a size and a terminator"
    );
    let decoded = Lz4BlockDecoder.decode(&encoded).expect("valid chain");
    assert_eq!(decoded, image);
}

#[test]
fn rejectsAChainWithoutATerminator() {
    let error = Lz4BlockDecoder
        .decode(&[0x00, 0x00, 0x00, 0x01, 0x00])
        .expect_err("truncated");
    let message = error.to_string();
    assert!(
        message.contains("terminating") || message.contains("remain") || message.contains("block"),
        "{message}"
    );
}
