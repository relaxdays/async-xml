use crate::symbol::*;

#[derive(Copy, Clone, PartialEq)]
pub enum TypePathType {
    Any,
    Vec,
    Option,
    XmlNode,
}

pub fn get_type_path_type(ty: &syn::Type) -> TypePathType {
    if let syn::Type::Path(path) = ty {
        if path.path.segments.len() == 1 && path.path.leading_colon.is_none() {
            let segment = &path.path.segments[0];
            if segment.ident == VEC {
                return TypePathType::Vec;
            } else if segment.ident == OPTION {
                return TypePathType::Option;
            } else if segment.ident == XML_NODE {
                // TODO: make this more stable for paths like async_xml::util::XmlNode
                return TypePathType::XmlNode;
            }
        }
    }
    TypePathType::Any
}

pub fn get_generic_arg(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(path) = ty {
        if path.path.segments.len() == 1 && path.path.leading_colon.is_none() {
            let segment = &path.path.segments[0];
            if segment.ident == VEC || segment.ident == OPTION {
                match &segment.arguments {
                    syn::PathArguments::AngleBracketed(a) => {
                        if let syn::GenericArgument::Type(t) = &a.args[0] {
                            return t.clone();
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    panic!("not a vector type!");
}
