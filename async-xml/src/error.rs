//! Module for the [`Error`](enum@Error) and [`Result`] types.

use thiserror::Error;

/// A [`Result`](std::result::Result) using [`Error`](enum@Error) as the error type
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for this crate
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to parse XML
    #[error("Failed to parse XML: {0}")]
    Xml(quick_xml::Error),
    /// Start element doesn't match the expected element
    #[error("Expected start element <{0}>, found <{1}>")]
    WrongStart(String, String),
    /// End element doesn't match the expected element
    #[error("Expected end element </{0}>, found </{1}>")]
    WrongEnd(String, String),
    /// Start element missing
    #[error("Missing start element")]
    MissingStart,
    /// Missing required attribute
    #[error("Missing attribute {0}")]
    MissingAttribute(String),
    /// Missing required child element
    #[error("Missing child <{0}>")]
    MissingChild(String),
    /// Missing required element text
    #[error("Missing element text")]
    MissingText,
    /// Encountered multiple child elements with the given name when only one was expected
    #[error("Found multiple child elements <{0}>, but only expected one")]
    DoubleChild(String),
    /// Encountered multiple text events when only one was expected
    #[error("Element contains multiple text events")]
    DoubleText,
    /// Encountered an unexpected attribute
    #[error("Found unexpected attribute {0}")]
    UnexpectedAttribute(String),
    /// Encountered an unexpected child element
    #[error("Found unexpected child element <{0}>")]
    UnexpectedChild(String),
    /// Encountered an unexpected text event
    #[error("Found unexpected text")]
    UnexpectedText,
    /// General deserialization error
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
