use super::*;

#[tokio::test]
async fn test_zst_struct() {
    let xml = r#"<data><empty /></data>"#;
    let de: DataStruct = from_str(xml).await.unwrap();
    let expected = DataStruct {
        empty: Some(EmptyStruct {}),
    };
    assert_eq!(de, expected);
}

#[tokio::test]
async fn test_zst_struct_missing() {
    let xml = r#"<data></data>"#;
    let de: DataStruct = from_str(xml).await.unwrap();
    let expected = DataStruct { empty: None };
    assert_eq!(de, expected);
}

#[tokio::test]
async fn test_zst_struct_required() {
    let xml = r#"<data><empty /></data>"#;
    let de: DataStructReq = from_str(xml).await.unwrap();
    let expected = DataStructReq {
        empty: EmptyStruct {},
    };
    assert_eq!(de, expected);
}

#[tokio::test]
#[should_panic]
async fn test_zst_struct_required_missing() {
    let xml = r#"<data></data>"#;
    let _: DataStructReq = from_str(xml).await.unwrap();
}

#[tokio::test]
async fn test_zst_tuple() {
    let xml = r#"<data><empty /></data>"#;
    let de: DataTuple = from_str(xml).await.unwrap();
    let expected = DataTuple {
        empty: Some(EmptyTuple {}),
    };
    assert_eq!(de, expected);
}

#[tokio::test]
async fn test_zst_tuple_missing() {
    let xml = r#"<data></data>"#;
    let de: DataTuple = from_str(xml).await.unwrap();
    let expected = DataTuple { empty: None };
    assert_eq!(de, expected);
}

#[derive(Debug, PartialEq, FromXml)]
pub struct DataStruct {
    #[async_xml(child)]
    empty: Option<EmptyStruct>,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct DataStructReq {
    #[async_xml(child)]
    empty: EmptyStruct,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct EmptyStruct {}

#[derive(Debug, PartialEq, FromXml)]
pub struct DataTuple {
    #[async_xml(child)]
    empty: Option<EmptyTuple>,
}

#[derive(Debug, PartialEq, FromXml)]
pub struct EmptyTuple();
