//! Reads the two-entry ZIP that Transfer writes for a `.dn2prj` project.

use std::io::{Cursor, Read};

use serde::Deserialize;
use zip::ZipArchive;

use crate::model::digitoneii::DigitoneIiManifest;

use super::digitoneiireaderror::DigitoneIiReadError;

#[derive(Debug, Deserialize)]
struct ManifestDocument {
    #[serde(rename = "FormatVersion")]
    formatVersion: String,
    #[serde(rename = "ProductType")]
    productType: Vec<String>,
    #[serde(rename = "Payload")]
    payload: String,
    #[serde(rename = "FileType")]
    fileType: String,
    #[serde(rename = "FirmwareVersion")]
    firmwareVersion: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct ProjectArchiveReader;

impl ProjectArchiveReader {
    pub(crate) fn read(
        self,
        bytes: &[u8],
    ) -> Result<(DigitoneIiManifest, Vec<u8>), DigitoneIiReadError> {
        let mut archive = ZipArchive::new(Cursor::new(bytes))?;
        let mut manifestBytes = None;
        let mut entries = Vec::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;
            if entry.is_dir() {
                continue;
            }
            let name = entry.name().to_owned();
            let mut data = Vec::new();
            entry.read_to_end(&mut data)?;
            if name == "manifest.json" {
                manifestBytes = Some(data);
            } else {
                entries.push((name, data));
            }
        }
        let Some(raw) = manifestBytes else {
            return Err(DigitoneIiReadError::MissingManifest);
        };
        let document: ManifestDocument = serde_json::from_slice(&raw)?;
        let Some(position) = entries
            .iter()
            .position(|(name, _)| name == &document.payload)
        else {
            return Err(DigitoneIiReadError::MissingPayload {
                name: document.payload,
            });
        };
        let payload = entries.swap_remove(position).1;
        Ok((
            DigitoneIiManifest::new(
                document.formatVersion,
                document.productType,
                document.payload,
                document.fileType,
                document.firmwareVersion,
                raw,
            ),
            payload,
        ))
    }
}
