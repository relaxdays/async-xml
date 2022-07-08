use std::str::FromStr;

use async_xml::from_str;
use async_xml_derive::FromXml;

#[tokio::main]
async fn main() {
    let report: Report = from_str(r#"<report id="a">text</report>"#).await.unwrap();
    println!("deserialized: {:?}", report);
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(rename = "report")]
pub struct Report {
    #[async_xml(attribute)]
    pub id: Id,
    #[async_xml(value)]
    pub data: String,
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(from_str)]
pub struct Id(String);

impl FromStr for Id {
    type Err = IdFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 0 {
            Err(IdFromStrError::Invalid)
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IdFromStrError {
    #[error("Invalid")]
    Invalid,
}
