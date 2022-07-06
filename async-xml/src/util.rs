use crate::reader::XmlFromStr;
use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};

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
