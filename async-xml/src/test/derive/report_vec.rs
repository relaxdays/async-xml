use crate as async_xml;
use crate::from_str;
use async_xml_derive::FromXml;

#[tokio::test]
async fn test() {
    let xml = r#"<report id="b"><data id="a">test</data><data id="3"></data></report>"#;
    let de: Report = from_str(xml).await.unwrap();
    let expected = Report {
        id: "b".into(),
        data: vec![
            ReportData {
                id: "a".into(),
                data: "test".into(),
            },
            ReportData {
                id: "3".into(),
                data: "".into(),
            },
        ],
    };
    assert_eq!(de, expected);
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(tag_name = "report")]
pub struct Report {
    #[async_xml(attribute)]
    pub id: String,
    #[async_xml(child)]
    pub data: Vec<ReportData>,
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(tag_name = "data")]
pub struct ReportData {
    #[async_xml(attribute)]
    pub id: String,
    #[async_xml(value, default)]
    pub data: String,
}
