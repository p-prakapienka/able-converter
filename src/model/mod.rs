//! Platform-neutral models shared by parsers, mappers, exporters, and user interfaces.

#[path = "Internal.rs"]
pub mod internal;

#[path = "Live.rs"]
pub mod live;

#[path = "Note.rs"]
pub mod note;

#[cfg(test)]
#[path = "InternalTest.rs"]
mod internaltest;

#[cfg(test)]
#[path = "NoteTest.rs"]
mod notetest;
