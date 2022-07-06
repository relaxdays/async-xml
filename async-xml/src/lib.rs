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

pub async fn from_str<'r, T: reader::FromXml<&'r [u8]>>(str: &'r str) -> Result<T> {
    let mut reader = PeekingReader::from_str(str);
    reader.deserialize().await
}
