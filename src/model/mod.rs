//! Platform-neutral models shared by parsers, mappers, exporters, and user interfaces.

#[path = "DigitoneIi.rs"]
pub mod digitoneii;

#[path = "Internal.rs"]
pub mod internal;

#[path = "Live.rs"]
pub mod live;

#[path = "Note.rs"]
pub mod note;

#[cfg(test)]
#[path = "DigitoneIiTest.rs"]
mod digitoneiitest;

#[cfg(test)]
#[path = "InternalTest.rs"]
mod internaltest;

#[cfg(test)]
#[path = "NoteTest.rs"]
mod notetest;
