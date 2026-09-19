use thiserror::Error;

#[derive(Debug, Error)]
pub enum CausticReadError {
    #[error("failed to read the Caustic song: {0}")]
    Io(#[from] std::io::Error),
    #[error("the file does not start with a RACK chunk")]
    MissingRackChunk,
    #[error("{structure} needs {needed} bytes at offset {offset} but only {available} remain")]
    UnexpectedEnd {
        structure: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("{structure} declares a length of {length} bytes but only {available} remain")]
    DeclaredLengthTooLarge {
        structure: &'static str,
        length: usize,
        available: usize,
    },
    #[error("{structure} contains a {field} value that is not valid ASCII")]
    InvalidAscii {
        structure: &'static str,
        field: &'static str,
    },
    #[error("machine slot {slot} declares identifier {machineId} which is not four characters")]
    InvalidMachineId { slot: usize, machineId: String },
}
