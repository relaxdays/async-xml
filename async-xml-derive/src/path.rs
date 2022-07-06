use crate::symbol::*;

#[derive(Copy, Clone, PartialEq)]
pub enum TypePathType {
    Any,
    Vec,
    Option,
}

pub fn get_type_path_type(ty: &syn::Type) -> TypePathType {
    if let syn::Type::Path(path) = ty {
        if path.path.segments.len() == 1 && path.path.leading_colon.is_none() {
            let segment = &path.path.segments[0];
            if segment.ident == VEC {
                return TypePathType::Vec;
            } else if segment.ident == OPTION {
                return TypePathType::Option;
            }
        }
    }
    TypePathType::Any
}
