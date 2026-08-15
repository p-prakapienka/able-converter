use std::str::FromStr;

use quick_xml::events::BytesStart;

use super::livereaderror::LiveReadError;

pub(super) struct LiveElement<'element, 'data> {
    element: &'element BytesStart<'data>,
}

impl<'element, 'data> LiveElement<'element, 'data> {
    pub(super) fn new(element: &'element BytesStart<'data>) -> Self {
        Self { element }
    }

    pub(super) fn attribute(&self, key: &[u8]) -> Result<Option<String>, LiveReadError> {
        for attribute in self.element.attributes().with_checks(false).flatten() {
            if attribute.key.as_ref() == key {
                let value = std::str::from_utf8(attribute.value.as_ref())?;
                return Ok(Some(quick_xml::escape::unescape(value)?.into_owned()));
            }
        }

        Ok(None)
    }

    pub(super) fn requiredAttribute(
        &self,
        key: &[u8],
        elementName: &'static str,
        attributeName: &'static str,
    ) -> Result<String, LiveReadError> {
        self.attribute(key)?.ok_or(LiveReadError::MissingAttribute {
            element: elementName,
            attribute: attributeName,
        })
    }

    pub(super) fn requiredNumberAttribute<T: FromStr>(
        &self,
        key: &[u8],
        elementName: &'static str,
    ) -> Result<T, LiveReadError> {
        let value = self.requiredAttribute(key, elementName, "numeric value")?;
        self.parseNumber(elementName, value)
    }

    pub(super) fn optionalNumberAttribute<T: FromStr>(
        &self,
        key: &[u8],
        elementName: &'static str,
    ) -> Result<Option<T>, LiveReadError> {
        self.attribute(key)?
            .map(|value| self.parseNumber(elementName, value))
            .transpose()
    }

    pub(super) fn requiredNumberValue<T: FromStr>(
        &self,
        elementName: &'static str,
    ) -> Result<T, LiveReadError> {
        self.requiredNumberAttribute(b"Value", elementName)
    }

    pub(super) fn optionalNumberValue<T: FromStr>(
        &self,
        elementName: &'static str,
    ) -> Result<Option<T>, LiveReadError> {
        self.optionalNumberAttribute(b"Value", elementName)
    }

    pub(super) fn parseNumber<T: FromStr>(
        &self,
        elementName: &'static str,
        value: String,
    ) -> Result<T, LiveReadError> {
        value.parse().map_err(|_| LiveReadError::InvalidValue {
            element: elementName,
            value,
        })
    }

    pub(super) fn requiredBooleanValue(
        &self,
        elementName: &'static str,
    ) -> Result<bool, LiveReadError> {
        let value = self.requiredAttribute(b"Value", elementName, "Value")?;
        self.parseBoolean(elementName, value)
    }

    pub(super) fn optionalBooleanAttribute(
        &self,
        key: &[u8],
        elementName: &'static str,
    ) -> Result<Option<bool>, LiveReadError> {
        self.attribute(key)?
            .map(|value| self.parseBoolean(elementName, value))
            .transpose()
    }

    fn parseBoolean(
        &self,
        elementName: &'static str,
        value: String,
    ) -> Result<bool, LiveReadError> {
        match value.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(LiveReadError::InvalidValue {
                element: elementName,
                value,
            }),
        }
    }
}
