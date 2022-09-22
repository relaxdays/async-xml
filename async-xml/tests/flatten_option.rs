use async_xml::from_str;
use async_xml_derive::FromXml;

#[tokio::test]
async fn test_present() {
    let xml =
        r#"<response status="ok"><status_text>success</status_text><data>text</data></response>"#;
    let de: Response = from_str(xml).await.unwrap();
    let expected = Response {
        status: Some(ResponseStatus {
            status: "ok".into(),
            status_text: "success".into(),
        }),
        data: ResponseData {
            text: "text".into(),
        },
    };
    assert_eq!(de, expected);
}

#[tokio::test]
async fn test_partial_present_1() {
    let xml = r#"<response status="ok"><data>text</data></response>"#;
    let de: Response = from_str(xml).await.unwrap();
    let expected = Response {
        status: None,
        data: ResponseData {
            text: "text".into(),
        },
    };
    assert_eq!(de, expected);
}

#[tokio::test]
async fn test_partial_present_2() {
    let xml = r#"<response><status_text>success</status_text><data>text</data></response>"#;
    let de: Response = from_str(xml).await.unwrap();
    let expected = Response {
        status: None,
        data: ResponseData {
            text: "text".into(),
        },
    };
    assert_eq!(de, expected);
}

#[tokio::test]
async fn test_not_present() {
    let xml = r#"<response><data>text</data></response>"#;
    let de: Response = from_str(xml).await.unwrap();
    let expected = Response {
        status: None,
        data: ResponseData {
            text: "text".into(),
        },
    };
    assert_eq!(de, expected);
}

#[derive(Debug, PartialEq, FromXml)]
pub struct ResponseStatus {
    #[async_xml(attribute)]
    status: String,
    #[async_xml(child)]
    status_text: String,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct Response {
    #[async_xml(flatten)]
    status: Option<ResponseStatus>,
    #[async_xml(child)]
    data: ResponseData,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct ResponseData {
    #[async_xml(value)]
    text: String,
}
