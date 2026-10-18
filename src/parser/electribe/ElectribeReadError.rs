use thiserror::Error;

#[derive(Debug, Error)]
pub enum ElectribeReadError {
    #[error("failed to read electribe data: {0}")]
    Io(#[from] std::io::Error),
    #[error("the file does not start with a KORG electribe file header")]
    MissingFileHeader,
    #[error("unsupported electribe product {0:?}; only electribe sampler files are supported")]
    UnsupportedProduct(String),
    #[error("unsupported electribe content marker {0:?}; expected a pattern or all-pattern file")]
    UnsupportedContent(String),
    #[error("{kind} is truncated: expected at least {expected} bytes, found {actual}")]
    Truncated {
        kind: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("pattern {number} is missing its PTST/PTED markers")]
    InvalidPatternMarkers { number: usize },
    #[error("the all-pattern global block is missing its GLST/GLED markers")]
    InvalidGlobalBlock,
    #[error("pattern {number} has invalid {field} value {value}")]
    InvalidPatternValue {
        number: usize,
        field: &'static str,
        value: u32,
    },
}
