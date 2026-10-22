#[path = "../DigitoneIiParser.rs"]
mod digitoneiiparser;

pub use digitoneiiparser::{DigitoneIiParser, DigitoneIiReadError};

#[path = "Checksum.rs"]
pub(crate) mod checksum;

#[path = "ContainerReader.rs"]
mod containerreader;

#[path = "DigitoneIiReadError.rs"]
mod digitoneiireaderror;

#[path = "Lz4BlockDecoder.rs"]
pub(crate) mod lz4blockdecoder;

#[path = "ProjectArchiveReader.rs"]
mod projectarchivereader;

#[cfg(test)]
#[path = "Lz4BlockDecoderTest.rs"]
mod lz4blockdecodertest;

#[cfg(test)]
#[path = "ChecksumTest.rs"]
mod checksumtest;
