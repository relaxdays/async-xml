use super::*;

#[tokio::test]
async fn test_children() {
    let xml = r#"
<test>
    text
    <useless>
        <child attribute="something"></child>
        <child>some random text whatever</child>
    </useless>
    <whatever />
</test>"#;
    let de: TestChildren = from_str(xml).await.unwrap();
    let expected = TestChildren {
        text: "text".into(),
    };
    assert_eq!(de, expected);
}

#[tokio::test]
#[should_panic]
async fn test_children_disallowed() {
    let xml = r#"
<test>
    text
    <useless>
        <child attribute="something"></child>
        <child>some random text whatever</child>
    </useless>
    <whatever />
</test>"#;
    let _: Test = from_str(xml).await.unwrap();
}

#[tokio::test]
async fn test_attributes() {
    let xml = r#"
<test attribute="something" attribute2="something-else">
    text
</test>"#;
    let de: TestAttributes = from_str(xml).await.unwrap();
    let expected = TestAttributes {
        text: "text".into(),
    };
    assert_eq!(de, expected);
}

#[tokio::test]
#[should_panic]
async fn test_attributes_disallowed() {
    let xml = r#"
<test attribute="something" attribute2="something-else">
    text
</test>"#;
    let _: Test = from_str(xml).await.unwrap();
}

#[derive(Debug, PartialEq, FromXml)]
pub struct Test {
    #[async_xml(value)]
    text: String,
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(allow_unknown_children)]
pub struct TestChildren {
    #[async_xml(value)]
    text: String,
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(allow_unknown_attributes)]
pub struct TestAttributes {
    #[async_xml(value)]
    text: String,
}
