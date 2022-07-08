use async_xml::from_str;
use async_xml_derive::FromXml;

#[tokio::test]
async fn test() {
    let xml = r#"<report id="b"><data>text</data></report>"#;
    let de: Report = from_str(xml).await.unwrap();
    let expected = Report {
        id: "b".into(),
        data: Some(ReportData {
            data: "text".into(),
        }),
    };
    assert_eq!(de, expected);
}

#[tokio::test]
async fn test_missing() {
    let xml = r#"<report id="b"></report>"#;
    let de: Report = from_str(xml).await.unwrap();
    let expected = Report {
        id: "b".into(),
        data: None,
    };
    assert_eq!(de, expected);
}

#[tokio::test]
async fn test_empty() {
    let xml = r#"<report id="b"><data /></report>"#;
    let de: Report = from_str(xml).await.unwrap();
    let expected = Report {
        id: "b".into(),
        data: None,
    };
    assert_eq!(de, expected);
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(rename = "report")]
pub struct Report {
    #[async_xml(attribute)]
    pub id: String,
    #[async_xml(child)]
    pub data: Option<ReportData>,
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(rename = "data")]
pub struct ReportData {
    #[async_xml(value)]
    pub data: String,
}
