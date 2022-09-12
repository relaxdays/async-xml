//! A crate for deserializing XML data asynchronously based on [`quick_xml`]

#![warn(missing_docs)]

pub mod error;
pub mod reader;
pub mod util;

#[cfg(test)]
mod test;

pub use self::error::{Error, Result};
pub use self::reader::{PeekingReader, Visitor};
pub use self::util::XmlVec;

#[cfg(feature = "derive")]
pub use async_xml_derive::FromXml;

/// Shortcut for deserializing data from a [`str`] containing XML
pub async fn from_str<'r, T: reader::FromXml<&'r [u8]>>(str: &'r str) -> Result<T> {
    let mut reader = PeekingReader::from_str(str);
    reader.deserialize().await
}
