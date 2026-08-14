//! Platform-neutral models shared by parsers, mappers, exporters, and user interfaces.

#[path = "Internal.rs"]
pub mod internal;

#[path = "Live.rs"]
pub mod live;

#[cfg(test)]
#[path = "InternalTest.rs"]
mod internal_test;
