use super::{FromXml, PeekingReader, Visitor};
use crate::Error;
use std::str::FromStr;
use tokio::io::AsyncBufRead;

impl<B, T> FromXml<B> for Option<T>
where
    B: AsyncBufRead + Send + Unpin,
    T: FromXml<B>,
{
    type Visitor = OptionalVisitor<T, B>;
}

pub struct OptionalVisitor<T, B>
where
    B: AsyncBufRead + Send + Unpin,
    T: FromXml<B>,
{
    empty: bool,
    inner_visitor: T::Visitor,
}

impl<T, B> Default for OptionalVisitor<T, B>
where
    B: AsyncBufRead + Send + Unpin,
    T: FromXml<B>,
{
    fn default() -> Self {
        Self {
            empty: true,
            inner_visitor: T::Visitor::default(),
        }
    }
}

#[async_trait::async_trait]
impl<B, T> Visitor<B> for OptionalVisitor<T, B>
where
    B: AsyncBufRead + Send + Unpin,
    T: FromXml<B>,
{
    type Output = Option<T>;

    fn start_name() -> Option<&'static str> {
        T::Visitor::start_name()
    }

    fn visit_attribute(&mut self, name: &str, value: &str) -> Result<(), Error> {
        self.empty = false;
        self.inner_visitor.visit_attribute(name, value)
    }

    async fn visit_child(
        &mut self,
        name: &str,
        reader: &mut PeekingReader<B>,
    ) -> Result<(), Error> {
        self.empty = false;
        self.inner_visitor.visit_child(name, reader).await
    }

    fn visit_text(&mut self, text: &str) -> Result<(), Error> {
        self.empty = false;
        self.inner_visitor.visit_text(text)
    }

    fn build(self) -> Result<Self::Output, Error> {
        if self.empty {
            return Ok(None);
        }
        let result = self.inner_visitor.build()?;
        Ok(Some(result))
    }
}

/// Marker trait for auto-implementing the [`FromXml`] trait based on the [`FromStr`]-implementation of a type.
pub trait XmlFromStr {}

impl<B, T, E> FromXml<B> for T
where
    B: AsyncBufRead + Send + Unpin,
    T: XmlFromStr + FromStr<Err = E> + Send,
    E: std::fmt::Display,
{
    type Visitor = FromStringVisitor<T>;
}

pub struct FromStringVisitor<T>
where
    T: FromStr,
{
    data: Option<T>,
}

impl<T, E> Default for FromStringVisitor<T>
where
    T: XmlFromStr + FromStr<Err = E> + Send,
    E: std::fmt::Display,
{
    fn default() -> Self {
        Self { data: None }
    }
}

#[async_trait::async_trait]
impl<B, T, E> Visitor<B> for FromStringVisitor<T>
where
    B: AsyncBufRead + Send + Unpin,
    T: XmlFromStr + FromStr<Err = E> + Send,
    E: std::fmt::Display,
{
    type Output = T;

    fn start_name() -> Option<&'static str> {
        None
    }

    fn visit_text(&mut self, text: &str) -> Result<(), Error> {
        if self.data.is_some() {
            return Err(Error::DoubleText);
        }
        match T::from_str(text) {
            Ok(t) => self.data = Some(t),
            Err(e) => return Err(Error::Deserialization(e.to_string())),
        }
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Error> {
        let data = if let Some(data) = self.data {
            data
        } else {
            return Err(Error::MissingText);
        };
        Ok(data)
    }
}

impl XmlFromStr for String {}
impl XmlFromStr for i8 {}
impl XmlFromStr for i16 {}
impl XmlFromStr for i32 {}
impl XmlFromStr for i64 {}
impl XmlFromStr for u8 {}
impl XmlFromStr for u16 {}
impl XmlFromStr for u32 {}
impl XmlFromStr for u64 {}
impl XmlFromStr for u128 {}
impl XmlFromStr for bool {}
impl XmlFromStr for char {}
impl XmlFromStr for &'static str {}

impl<T: XmlFromStr> XmlFromStr for &'static T {}
