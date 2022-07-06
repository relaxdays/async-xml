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
#[from_xml(tag_name = "report")]
pub struct Report {
    #[from_xml(attribute)]
    pub id: Id,
    #[from_xml(child)]
    pub data: Data,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct Id(String);

#[derive(Debug, PartialEq, FromXml)]
pub struct Data(Value);

#[derive(Debug, PartialEq, FromXml)]
pub struct Value(String);
