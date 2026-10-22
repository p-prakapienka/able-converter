use thiserror::Error;

#[derive(Debug, Error)]
pub enum DigitoneIiReadError {
    #[error("failed to read the Digitone II project: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to read the Digitone II archive: {0}")]
    Archive(#[from] zip::result::ZipError),
    #[error("failed to read manifest.json: {0}")]
    Manifest(#[from] serde_json::Error),
    #[error("the archive has no manifest.json entry")]
    MissingManifest,
    #[error("the archive has no payload entry named {name}")]
    MissingPayload { name: String },
    #[error("the payload is {length} bytes, which is too short for a container")]
    PayloadTooShort { length: usize },
    #[error("the payload does not start with the AC11D303 container magic")]
    BadMagic,
    #[error("the payload does not end with the AAA1DAAA footer magic")]
    BadFooter,
    #[error("the container pair at offset 4 is {found:02X?}, not 02 00 05 00")]
    UnexpectedPair { found: [u8; 4] },
    #[error("the container trailer-length byte is {found}, not 12")]
    UnexpectedTrailerLength { found: u8 },
    #[error("device byte {device} is not a Digitone II container")]
    NotDigitoneIi { device: u8 },
    #[error("content kind {contentKind} is not a project")]
    NotProject { contentKind: u32 },
    #[error("the payload length field is {stored} but the body is {computed} bytes")]
    LengthMismatch { stored: u32, computed: usize },
    #[error("the payload check field is {stored:#010X} but the body computes {computed:#010X}")]
    ChecksumMismatch { stored: u32, computed: u32 },
    #[error(
        "the container declares an image of {declared} bytes but decompression produced {actual}"
    )]
    UncompressedLengthMismatch { declared: u32, actual: usize },
    #[error("decoded image is {length} bytes, not a known Digitone II project size")]
    ImageSize { length: usize },
    #[error("LZ4 block chain: {detail}")]
    Lz4 { detail: String },
}
