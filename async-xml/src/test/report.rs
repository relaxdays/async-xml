use crate::{from_str, reader::FromXml, Error, PeekingReader, Result, Visitor};
use tokio::io::AsyncBufRead;

#[derive(Debug, PartialEq)]
pub struct Report {
    pub id: String,
    pub data: Option<ReportData>,
}

#[derive(Debug, PartialEq)]
pub struct ReportData {
    pub data: String,
}

#[derive(Default)]
pub struct ReportVisitor {
    id: Option<String>,
    data: Option<ReportData>,
}

#[async_trait::async_trait(?Send)]
impl<B: AsyncBufRead + Unpin> Visitor<B> for ReportVisitor {
    type Output = Report;

    fn visit_attribute(&mut self, name: &str, value: &str) -> Result<()> {
        match name {
            "id" => {
                self.id.replace(value.into());
            }
            _ => return Err(Error::UnexpectedAttribute(name.into())),
        }
        Ok(())
    }

    async fn visit_child(&mut self, name: &str, reader: &mut PeekingReader<B>) -> Result<()> {
        match name {
            "data" => {
                if self.data.is_some() {
                    return Err(Error::DoubleChild(name.into()));
                }
                self.data = reader.deserialize().await?;
            }
            _ => return Err(Error::UnexpectedChild(name.into())),
        }
        Ok(())
    }

    fn build(self) -> Result<Self::Output> {
        let id = if let Some(id) = self.id {
            id
        } else {
            return Err(Error::MissingAttribute("id".into()));
        };
        Ok(Report {
            id,
            data: self.data,
        })
    }
}

impl<B: AsyncBufRead + Unpin> FromXml<B> for Report {
    type Visitor = ReportVisitor;
}

#[derive(Default)]
pub struct ReportDataVisitor {
    data: Option<String>,
}

impl<B: AsyncBufRead + Unpin> Visitor<B> for ReportDataVisitor {
    type Output = ReportData;

    fn visit_text(&mut self, text: &str) -> Result<()> {
        if self.data.replace(text.into()).is_some() {
            Err(Error::DoubleText)
        } else {
            Ok(())
        }
    }

    fn build(self) -> Result<Self::Output> {
        let data = if let Some(data) = self.data {
            data
        } else {
            return Err(Error::MissingText);
        };
        Ok(ReportData { data })
    }
}

impl<B: AsyncBufRead + Unpin> FromXml<B> for ReportData {
    type Visitor = ReportDataVisitor;
}

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
