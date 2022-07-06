use async_xml::from_str;
use async_xml_derive::FromXml;

#[tokio::main]
async fn main() {
    let report: Report = from_str(r#"<report id="5"></report>"#).await.unwrap();
    println!("deserialized: {:?}", report);
}

#[derive(Debug, PartialEq, FromXml)]
#[from_xml(tag_name = "report")]
pub struct Report {
    #[from_xml(attribute)]
    pub id: Id,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct Id(u32);
