use async_xml::from_str;
use async_xml_derive::FromXml;

#[tokio::main]
async fn main() {
    let report: Report =
        from_str(r#"<report id="b"><data id="a">test</data><data id="3"></data></report>"#)
            .await
            .unwrap();
    println!("deserialized: {:?}", report);
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(rename = "report")]
pub struct Report {
    #[async_xml(attribute)]
    pub id: String,
    #[async_xml(child)]
    pub data: Vec<ReportData>,
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(rename = "data")]
pub struct ReportData {
    #[async_xml(attribute)]
    pub id: String,
    #[async_xml(value, default)]
    pub data: String,
}
