use thiserror::Error;

#[derive(Debug, Error)]
pub enum DigitoneIiWriteError {
    #[error("failed to write the Digitone II archive: {0}")]
    Archive(#[from] zip::result::ZipError),
    #[error("failed to write the Digitone II project: {0}")]
    Io(#[from] std::io::Error),
    #[error("the project has no payload name, so it cannot be packed as a .dn2prj archive")]
    MissingPayloadName,
    #[error("the project image is {length} bytes, not a known Digitone II project size")]
    ImageSize { length: usize },
}
