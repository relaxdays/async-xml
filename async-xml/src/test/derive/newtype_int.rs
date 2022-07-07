use crate as async_xml;
use crate::from_str;
use async_xml_derive::FromXml;

#[tokio::test]
async fn test() {
    let xml = r#"<report id="5"></report>"#;
    let de: Report = from_str(xml).await.unwrap();
    let expected = Report { id: Id(5) };
    assert_eq!(de, expected);
}

#[tokio::test]
#[should_panic]
async fn test_invalid() {
    let xml = r#"<report id="-5"></report>"#;
    let _: Report = from_str(xml).await.unwrap();
}

#[tokio::test]
#[should_panic]
async fn test_missing() {
    let xml = r#"<report id=""></report>"#;
    let _: Report = from_str(xml).await.unwrap();
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(tag_name = "report")]
pub struct Report {
    #[async_xml(attribute)]
    pub id: Id,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct Id(u32);
