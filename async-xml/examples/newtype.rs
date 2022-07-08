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
#[async_xml(rename = "report")]
pub struct Report {
    #[async_xml(attribute)]
    pub id: Id,
    #[async_xml(child)]
    pub data: Data,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct Id(String);

#[derive(Debug, PartialEq, FromXml)]
pub struct Data(String);
