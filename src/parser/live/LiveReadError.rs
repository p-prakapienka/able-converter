use thiserror::Error;

#[derive(Debug, Error)]
pub enum LiveReadError {
    #[error("failed to read Ableton Live XML: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("an XML attribute could not be decoded as UTF-8: {0}")]
    InvalidAttribute(#[from] std::str::Utf8Error),
    #[error("an XML attribute contains an invalid escape sequence: {0}")]
    InvalidEscape(#[from] quick_xml::escape::EscapeError),
    #[error("invalid {element} value: {value}")]
    InvalidValue {
        element: &'static str,
        value: String,
    },
    #[error("{element} is missing required {attribute} attribute")]
    MissingAttribute {
        element: &'static str,
        attribute: &'static str,
    },
    #[error("{parent} is missing required {element} element")]
    MissingElement {
        parent: &'static str,
        element: &'static str,
    },
    #[error("unexpected nested {0} element")]
    UnexpectedStructure(&'static str),
    #[error("the file does not contain an Ableton root element")]
    MissingAbletonRoot,
}
