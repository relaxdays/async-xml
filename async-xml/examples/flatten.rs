use async_xml::from_str;
use async_xml_derive::FromXml;

#[tokio::main]
async fn main() {
    let xml = r#"<response status="ok"><data>text</data></response>"#;
    let de: Response = from_str(xml).await.unwrap();
    println!("deserialized: {:?}", de);
}

#[derive(Debug, PartialEq, FromXml)]
pub struct ResponseStatus {
    #[async_xml(attribute)]
    status: String,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct Response {
    #[async_xml(flatten)]
    status: ResponseStatus,
    #[async_xml(child)]
    data: ResponseData,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct ResponseData {
    #[async_xml(value)]
    text: String,
}
