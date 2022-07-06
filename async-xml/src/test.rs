pub mod report;

use self::report::*;
use crate::from_str;

#[tokio::test]
async fn test_missing() {
    let xml = r#"<report id="a"></report>"#;
    let de: Report = from_str(xml).await.unwrap();
    assert_eq!(
        de,
        Report {
            id: "a".to_string(),
            data: None
        }
    );
}

#[tokio::test]
async fn test_empty() {
    let xml = r#"<report id="a"><data /></report>"#;
    let de: Report = from_str(xml).await.unwrap();
    assert_eq!(
        de,
        Report {
            id: "a".to_string(),
            data: None
        }
    );
}

#[tokio::test]
async fn test_full() {
    let xml = r#"<report id="a"><data>test</data></report>"#;
    let de: Report = from_str(xml).await.unwrap();
    assert_eq!(
        de,
        Report {
            id: "a".to_string(),
            data: Some(ReportData {
                data: "test".to_string()
            }),
        }
    );
}
