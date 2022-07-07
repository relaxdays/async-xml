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
    #[from_xml(attribute)]
    status: String,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct Response {
    #[from_xml(flatten)]
    status: ResponseStatus,
    #[from_xml(child)]
    data: ResponseData,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct ResponseData {
    #[from_xml(value)]
    text: String,
}
