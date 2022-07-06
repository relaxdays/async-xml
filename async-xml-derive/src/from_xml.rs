use crate::attr::Container;
use crate::ctx::Ctx;
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_from_xml(input: &syn::DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let ctx = Ctx::new();
    let container = Container::from_attrs(&ctx, &input.attrs);
    ctx.check()?;

    if container.use_from_str {
        let name = &input.ident;
        return Ok(quote! {
            impl async_xml::reader::XmlFromStr for #name {}
        });
    }

    match &input.data {
        syn::Data::Struct(d) => crate::xml_struct::expand_struct(container, &*input, d),
        _ => Err(vec![syn::Error::new_spanned(
            input,
            "only struct types implemented",
        )]),
    }
}
