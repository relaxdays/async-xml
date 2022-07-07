use async_xml::from_str;
use async_xml_derive::FromXml;

#[tokio::main]
async fn main() {
    let xml = r#"<data><empty /></data>"#;
    let de: DataTuple = from_str(xml).await.unwrap();
    println!("deserialized: {:?}", de);
}

#[derive(Debug, PartialEq, FromXml)]
pub struct DataTuple {
    #[async_xml(child)]
    empty: Option<EmptyTuple>,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct EmptyTuple();
