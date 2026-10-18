#[path = "../ElectribeParser.rs"]
mod electribeparser;

pub use electribeparser::{ElectribeParser, ElectribeReadError};

#[path = "ByteBlock.rs"]
mod byteblock;

#[path = "ElectribeReadError.rs"]
mod electribereaderror;

#[path = "FileHeaderReader.rs"]
mod fileheaderreader;

#[path = "MotionSlotReader.rs"]
mod motionslotreader;

#[path = "PartReader.rs"]
mod partreader;

#[path = "PatternReader.rs"]
mod patternreader;

#[path = "StepReader.rs"]
mod stepreader;
