use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod attr;
mod ctx;
mod field;
mod from_xml;
mod path;
mod respan;
mod symbol;

#[proc_macro_derive(FromXml, attributes(from_xml))]
pub fn derive_from_xml(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    from_xml::expand_from_xml(&mut input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}
