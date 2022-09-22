//! Miscellaneous helper types

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
    B: AsyncBufRead + Unpin,
{
    type Output = Self;

    fn visit_tag(&mut self, name: &str) -> Result<(), Error> {
        tracing::trace!("XmlNode deserializing element <{}>", name);
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
        tracing::trace!("XmlNode done deserializing element <{}>", self.name);
        Ok(self)
    }
}

impl<B> FromXml<B> for XmlNode
where
    B: AsyncBufRead + Unpin,
{
    type Visitor = Self;
}

/// A generic visitor that will discard all errors thrown during build
///
/// This visitor can be wrapped around any existing visitor and its output type will be the wrapped visitor's output
/// type wrapped in an [`Option`]. Errors thrown during build will be discarded and a [`None`]-value will be returned.
pub struct DiscardErrorVisitor<V, B>
where
    B: AsyncBufRead + Unpin,
    V: Visitor<B>,
{
    inner_visitor: V,
    _phantom: core::marker::PhantomData<B>,
}

impl<V, B> Default for DiscardErrorVisitor<V, B>
where
    B: AsyncBufRead + Unpin,
    V: Visitor<B> + Default,
{
    fn default() -> Self {
        Self {
            inner_visitor: V::default(),
            _phantom: core::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<V, B> Visitor<B> for DiscardErrorVisitor<V, B>
where
    B: AsyncBufRead + Unpin,
    V: Visitor<B>,
{
    type Output = Option<V::Output>;

    fn start_name() -> Option<&'static str> {
        V::start_name()
    }

    fn visit_attribute(&mut self, name: &str, value: &str) -> Result<(), Error> {
        self.inner_visitor.visit_attribute(name, value)
    }

    async fn visit_child(
        &mut self,
        name: &str,
        reader: &mut crate::PeekingReader<B>,
    ) -> Result<(), Error> {
        self.inner_visitor.visit_child(name, reader).await
    }

    fn visit_text(&mut self, text: &str) -> Result<(), Error> {
        self.inner_visitor.visit_text(text)
    }

    fn build(self) -> Result<Self::Output, Error> {
        match self.inner_visitor.build() {
            Ok(t) => Ok(Some(t)),
            Err(e) => {
                tracing::trace!("discarding build error: {:?}", e);
                Ok(None)
            }
        }
    }
}
