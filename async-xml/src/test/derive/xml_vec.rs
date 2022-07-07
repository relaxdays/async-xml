use crate as async_xml;
use crate::{from_str, XmlVec};
use async_xml_derive::FromXml;

#[tokio::test]
async fn test_xml_vec() {
    let xml = r#"<report><ids>2 4 6 7</ids></report>"#;
    let de: Report = from_str(xml).await.unwrap();
    let expected = Report {
        data: vec![2, 4, 6, 7].into(),
    };
    assert_eq!(de, expected);
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(tag_name = "report")]
pub struct Report {
    #[async_xml(child, rename = "ids")]
    pub data: XmlVec<u32>,
}
