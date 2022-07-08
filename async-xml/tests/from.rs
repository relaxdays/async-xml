use async_xml::{from_str, util::XmlNode};
use async_xml_derive::FromXml;

#[tokio::test]
async fn test_from() {
    let xml = r#"<report id="3" />"#;
    let de: Report = from_str(xml).await.unwrap();
    let expected = Report { id: 3 };
    assert_eq!(de, expected);
}

#[tokio::test]
async fn test_try_from() {
    let xml = r#"<report id="3" />"#;
    let de: ReportTry = from_str(xml).await.unwrap();
    let expected = ReportTry { id: 3 };
    assert_eq!(de, expected);
}

#[tokio::test]
#[should_panic]
async fn test_try_from_panic() {
    let xml = r#"<report id="5" />"#;
    let _: ReportTry = from_str(xml).await.unwrap();
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(rename = "report", from = "async_xml::util::XmlNode")]
pub struct Report {
    pub id: u32,
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(rename = "report", try_from = "async_xml::util::XmlNode")]
pub struct ReportTry {
    pub id: u32,
}

impl From<XmlNode> for Report {
    fn from(n: XmlNode) -> Self {
        let id = n
            .attributes
            .into_iter()
            .find(|a| a.name == "id")
            .unwrap()
            .value;
        let id = id.parse().unwrap();
        Self { id }
    }
}

impl TryFrom<XmlNode> for ReportTry {
    type Error = String;

    fn try_from(n: XmlNode) -> Result<Self, Self::Error> {
        let id = n
            .attributes
            .into_iter()
            .find(|a| a.name == "id")
            .unwrap()
            .value;
        let id = id.parse().unwrap();
        // just test this with a defined error message
        if id == 5 {
            return Err("invalid num: 5".into());
        }
        Ok(Self { id })
    }
}
