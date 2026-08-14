//! Application-level use cases shared by native and browser adapters.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use mapper_model::LiveSetInspection;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InspectError {
    #[error("failed to open {path}: {source}")]
    Open {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Live(#[from] mapper_live::LiveReadError),
}

pub fn inspect_als<R: Read>(source: R) -> Result<LiveSetInspection, InspectError> {
    Ok(mapper_live::inspect_als(source)?)
}

pub fn inspect_als_path(path: impl AsRef<Path>) -> Result<LiveSetInspection, InspectError> {
    let path = path.as_ref();
    let file = File::open(path).map_err(|source| InspectError::Open {
        path: path.to_owned(),
        source,
    })?;

    inspect_als(file)
}
