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
#[from_xml(tag_name = "report")]
pub struct Report {
    #[from_xml(attribute)]
    pub id: String,
    #[from_xml(child)]
    pub data: Option<ReportData>,
}

#[derive(Debug, PartialEq, FromXml)]
#[from_xml(tag_name = "data")]
pub struct ReportData {
    #[from_xml(value)]
    pub data: String,
}
