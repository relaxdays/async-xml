use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to parse XML: {0}")]
    Xml(quick_xml::Error),
    #[error("Expected start element <{0}>, found <{1}>")]
    WrongStart(String, String),
    #[error("Expected end element </{0}>, found </{1}>")]
    WrongEnd(String, String),
    #[error("Missing start element")]
    MissingStart,
    #[error("Missing attribute {0}")]
    MissingAttribute(String),
    #[error("Missing child <{0}>")]
    MissingChild(String),
    #[error("Missing element text")]
    MissingText,
    #[error("Found multiple child elements <{0}>, but only expected one")]
    DoubleChild(String),
    #[error("Element contains multiple text events")]
    DoubleText,
    #[error("Found unexpected attribute {0}")]
    UnexpectedAttribute(String),
    #[error("Found unexpected child element <{0}>")]
    UnexpectedChild(String),
    #[error("Found unexpected text")]
    UnexpectedText,
    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

impl<T> From<T> for Error
where
    T: Into<quick_xml::Error>,
{
    fn from(e: T) -> Self {
        Self::Xml(e.into())
    }
}
