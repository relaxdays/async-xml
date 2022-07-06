use crate::util::{XmlAttribute, XmlNode};

use super::*;

#[tokio::test]
async fn test_children() {
    let xml = r#"
<test>
    text
    <required>this is important!</required>
    <useless>
        <child attribute="something"></child>
        <child>some random text whatever</child>
    </useless>
    <whatever />
</test>"#;
    let de: Test = from_str(xml).await.unwrap();
    let expected = Test {
        text: "text".into(),
        required: "this is important!".into(),
        remaining: XmlNode {
            name: "test".into(),
            children: vec![
                XmlNode {
                    name: "useless".into(),
                    children: vec![
                        XmlNode {
                            name: "child".into(),
                            attributes: vec![XmlAttribute {
                                name: "attribute".into(),
                                value: "something".into(),
                            }],
                            ..Default::default()
                        },
                        XmlNode {
                            name: "child".into(),
                            text: Some("some random text whatever".into()),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
                XmlNode {
                    name: "whatever".into(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
    };
    assert_eq!(de, expected);
}

#[derive(Debug, PartialEq, FromXml)]
pub struct Test {
    #[from_xml(value)]
    text: String,
    #[from_xml(child)]
    required: String,
    #[from_xml(remains)]
    remaining: XmlNode,
}
