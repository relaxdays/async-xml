use crate as async_xml;
use crate::from_str;
use async_xml_derive::FromXml;

#[tokio::test]
async fn test_xml_vec_derive() {
    let xml = r#"<report><ids>2 4 6 7</ids></report>"#;
    let de: Report = from_str(xml).await.unwrap();
    let expected = Report {
        data: vec![2, 4, 6, 7].into(),
    };
    assert_eq!(de, expected);
}

#[derive(Debug, PartialEq, FromXml)]
#[from_xml(tag_name = "report")]
pub struct Report {
    #[from_xml(child, rename = "ids", from = "async_xml::XmlVec<u32>")]
    pub data: Vec<u32>,
}