use crate as async_xml;
use crate::from_str;
use async_xml_derive::FromXml;
use std::str::FromStr;

#[tokio::test]
async fn test() {
    let xml = r#"<report id="a">text</report>"#;
    let de: Report = from_str(xml).await.unwrap();
    let expected = Report {
        id: Id("a".into()),
        data: "text".into(),
    };
    assert_eq!(de, expected);
}

#[tokio::test]
#[should_panic]
async fn test_invalid() {
    let xml = r#"<report id="">text</report>"#;
    let _: Report = from_str(xml).await.unwrap();
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(tag_name = "report")]
pub struct Report {
    #[async_xml(attribute)]
    pub id: Id,
    #[async_xml(value)]
    pub data: String,
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(use_from_str)]
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
