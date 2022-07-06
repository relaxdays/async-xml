use proc_macro2::Ident;
use syn::Path;

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

pub const FROM_XML: Symbol = Symbol("from_xml");
pub const TAG_NAME: Symbol = Symbol("tag_name");
pub const ATTRIBUTE: Symbol = Symbol("attribute");
pub const VALUE: Symbol = Symbol("value");
pub const CHILD: Symbol = Symbol("child");
pub const DEFAULT: Symbol = Symbol("default");
pub const RENAME: Symbol = Symbol("rename");
pub const VEC: Symbol = Symbol("Vec");
pub const OPTION: Symbol = Symbol("Option");

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
