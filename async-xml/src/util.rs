use crate::{
    reader::{FromXml, XmlFromStr},
    Error, Visitor,
};
use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};
use tokio::io::AsyncBufRead;

/// A vector expecting space-separated items.
#[derive(Debug, Clone, PartialEq)]
pub struct XmlVec<T> {
    items: Vec<T>,
}

impl<T, E> FromStr for XmlVec<T>
where
    T: FromStr<Err = E>,
{
    type Err = E;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut items = Vec::new();
        for split in s.split(' ') {
            let item = T::from_str(split)?;
            items.push(item);
        }
        Ok(Self { items })
    }
}

impl<T> XmlFromStr for XmlVec<T> where T: FromStr {}

impl<T> Deref for XmlVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T> DerefMut for XmlVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl<T> From<XmlVec<T>> for Vec<T> {
    fn from(v: XmlVec<T>) -> Self {
        v.items
    }
}

impl<T> From<Vec<T>> for XmlVec<T> {
    fn from(v: Vec<T>) -> Self {
        Self { items: v }
    }
}

/// An XML node that isn't deserialized into a more specific type.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct XmlNode {
    /// Tag name of the node.
    pub name: String,
    /// Attributes of the node.
    pub attributes: Vec<XmlAttribute>,
    /// Text content of the node.
    pub text: Option<String>,
    /// Child nodes.
    pub children: Vec<XmlNode>,
}

/// An attribute of an [`XmlNode`].
#[derive(Debug, Clone, PartialEq)]
pub struct XmlAttribute {
    /// Attribute name.
    pub name: String,
    /// Attribute value.
    pub value: String,
}

#[async_trait::async_trait(?Send)]
impl<B> Visitor<B> for XmlNode
where
    B: AsyncBufRead + Send + Unpin,
{
    type Output = Self;

    fn visit_tag(&mut self, name: &str) -> Result<(), Error> {
        self.name = name.to_string();
        Ok(())
    }

    fn visit_attribute(&mut self, name: &str, value: &str) -> Result<(), Error> {
        self.attributes.push(XmlAttribute {
            name: name.into(),
            value: value.into(),
        });
        Ok(())
    }

    fn visit_text(&mut self, text: &str) -> Result<(), Error> {
        self.text = Some(text.into());
        Ok(())
    }

    async fn visit_child(
        &mut self,
        _name: &str,
        reader: &mut crate::PeekingReader<B>,
    ) -> Result<(), Error> {
        self.children.push(reader.deserialize().await?);
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Error> {
        Ok(self)
    }
}

impl<B> FromXml<B> for XmlNode
where
    B: AsyncBufRead + Send + Unpin,
{
    type Visitor = Self;
}
