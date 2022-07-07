use crate as async_xml;
use crate::from_str;
use async_xml_derive::FromXml;

#[tokio::test]
async fn test() {
    let xml = r#"<report id="b"><data>text</data></report>"#;
    let de: Report = from_str(xml).await.unwrap();
    let expected = Report {
        id: Id("b".into()),
        data: Data(Value("text".into())),
    };
    assert_eq!(de, expected);
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(tag_name = "report")]
pub struct Report {
    #[async_xml(attribute)]
    pub id: Id,
    #[async_xml(child)]
    pub data: Data,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct Id(String);

#[derive(Debug, PartialEq, FromXml)]
pub struct Data(Value);

#[derive(Debug, PartialEq, FromXml)]
pub struct Value(String);
