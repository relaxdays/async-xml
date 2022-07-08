use async_xml::from_str;
use async_xml_derive::FromXml;

#[tokio::main]
async fn main() {
    let report: Report = from_str(r#"<report><ids>2 4 6 7</ids></report>"#)
        .await
        .unwrap();
    println!("deserialized: {:?}", report);
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(rename = "report")]
pub struct Report {
    #[async_xml(child, rename = "ids", from = "async_xml::XmlVec<u32>")]
    pub data: Vec<u32>,
}
