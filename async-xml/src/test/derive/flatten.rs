use super::*;

#[tokio::test]
async fn test() {
    let xml = r#"<response status="ok"><data>text</data></response>"#;
    let de: Response = from_str(xml).await.unwrap();
    let expected = Response {
        status: ResponseStatus {
            status: "ok".into(),
        },
        data: ResponseData {
            text: "text".into(),
        },
    };
    assert_eq!(de, expected);
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
