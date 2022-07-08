use proc_macro2::Ident;
use syn::Path;

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

pub const ASYNC_XML: Symbol = Symbol("async_xml");
pub const ATTRIBUTE: Symbol = Symbol("attribute");
pub const VALUE: Symbol = Symbol("value");
pub const CHILD: Symbol = Symbol("child");
pub const REMAINS: Symbol = Symbol("remains");
pub const DEFAULT: Symbol = Symbol("default");
pub const RENAME: Symbol = Symbol("rename");
pub const VEC: Symbol = Symbol("Vec");
pub const OPTION: Symbol = Symbol("Option");
pub const XML_NODE: Symbol = Symbol("XmlNode");
pub const FROM_STR: Symbol = Symbol("from_str");
pub const FROM: Symbol = Symbol("from");
pub const TRY_FROM: Symbol = Symbol("try_from");
pub const ALLOW_UNKNOWN_CHILDREN: Symbol = Symbol("allow_unknown_children");
pub const ALLOW_UNKNOWN_ATTRIBUTES: Symbol = Symbol("allow_unknown_attributes");
pub const ALLOW_UNKNOWN_TEXT: Symbol = Symbol("allow_unknown_text");
pub const ALLOW_UNKNOWN: Symbol = Symbol("allow_unknown");
pub const FLATTEN: Symbol = Symbol("flatten");

impl PartialEq<Symbol> for Path {
    fn eq(&self, other: &Symbol) -> bool {
        self.is_ident(other.0)
    }
}

impl PartialEq<Symbol> for Ident {
    fn eq(&self, other: &Symbol) -> bool {
        self == other.0
    }
}
