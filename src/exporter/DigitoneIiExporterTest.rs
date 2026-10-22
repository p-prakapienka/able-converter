use super::digitoneii::{DigitoneIiExporter, DigitoneIiWriteError};
use crate::model::digitoneii::{CONTAINER_HEADER_LEN, DigitoneIiManifest, DigitoneIiProject};

#[test]
fn refusesToPackAProjectThatHasNoManifest() {
    let project = DigitoneIiProject::fromParts(
        DigitoneIiManifest::new(
            String::new(),
            Vec::new(),
            String::new(),
            String::new(),
            String::new(),
            Vec::new(),
        ),
        [0; CONTAINER_HEADER_LEN],
        Vec::new(),
    );

    let error = DigitoneIiExporter::new()
        .exportProject(&project)
        .expect_err("raw container");
    assert!(matches!(error, DigitoneIiWriteError::MissingPayloadName));
}

#[test]
fn refusesAnImageThatIsNotADigitoneIiProject() {
    let manifest = br#"{"FormatVersion":"1.0","ProductType":[],"Payload":"MORNING","FileType":"Project","FirmwareVersion":"1.10E"}"#;
    let project = DigitoneIiProject::fromParts(
        DigitoneIiManifest::new(
            "1.0".to_owned(),
            Vec::new(),
            "MORNING".to_owned(),
            "Project".to_owned(),
            "1.10E".to_owned(),
            manifest.to_vec(),
        ),
        [0; CONTAINER_HEADER_LEN],
        vec![0; 32],
    );

    let error = DigitoneIiExporter::new()
        .exportPayload(&project)
        .expect_err("short image");
    assert!(matches!(
        error,
        DigitoneIiWriteError::ImageSize { length: 32 }
    ));
}
