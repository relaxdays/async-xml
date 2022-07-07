use crate as async_xml;
use crate::from_str;
use async_xml_derive::FromXml;

pub mod discard;
pub mod flatten;
pub mod newtype;
pub mod newtype_custom;
pub mod newtype_int;
pub mod report_option;
pub mod report_vec;
pub mod store_any;
pub mod xml_vec;
pub mod xml_vec_derive;
pub mod zst;
