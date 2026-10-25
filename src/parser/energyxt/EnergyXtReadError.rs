use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnergyXtReadError {
    #[error("failed to read the energyXT file: {0}")]
    Io(#[from] std::io::Error),
    #[error("the file is not an energyXT project (missing the XTF header)")]
    NotAProject,
    #[error("{context} ends early at byte {offset}")]
    Truncated {
        context: &'static str,
        offset: usize,
    },
    #[error("invalid record key at byte {offset}")]
    InvalidKey { offset: usize },
    #[error("invalid {record} record: {reason}")]
    InvalidRecord {
        record: &'static str,
        reason: String,
    },
    #[error("{record} record appears before the record it belongs to")]
    UnexpectedRecord { record: &'static str },
    #[error("the project has no sequencer module")]
    MissingSequencer,
}
