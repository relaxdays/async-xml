use async_xml::from_str;
use async_xml_derive::FromXml;

#[tokio::main]
async fn main() {
    let report: Report = from_str(r#"<report id="b"><data>text</data></report>"#)
        .await
        .unwrap();
    println!("deserialized: {:?}", report);
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(tag_name = "report")]
pub struct Report {
    #[async_xml(attribute)]
    pub id: String,
    #[async_xml(child)]
    pub data: Option<ReportData>,
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(tag_name = "data")]
pub struct ReportData {
    #[async_xml(value)]
    pub data: String,
}
