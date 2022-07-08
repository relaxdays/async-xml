use async_xml::from_str;
use async_xml_derive::FromXml;

#[tokio::test]
async fn test() {
    let xml = r#"<a></a>"#;
    let de: A = from_str(xml).await.unwrap();
    let expected = A {};
    assert_eq!(de, expected);
}

#[derive(Debug, PartialEq, FromXml)]
struct A {}
