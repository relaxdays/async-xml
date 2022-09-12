use crate::attr::{self, Container};
use crate::ctx::Ctx;
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_from_xml(input: &syn::DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let ctx = Ctx::new();
    let container = Container::from_attrs(&ctx, &input.attrs);
    ctx.check()?;

    match &container.from {
        attr::From::Default => {}
        attr::From::From(t) => {
            let name = &input.ident;
            return Ok(quote! {
                impl<B> ::async_xml::reader::FromXml<B> for #name
                where
                    B: ::tokio::io::AsyncBufRead + Unpin,
                {
                    type Visitor = ::async_xml::reader::FromVisitor<B, #name, #t>;
                }
            });
        }
        attr::From::TryFrom(t) => {
            let name = &input.ident;
            return Ok(quote! {
                impl<B> ::async_xml::reader::FromXml<B> for #name
                where
                    B: ::tokio::io::AsyncBufRead + Unpin,
                {
                    type Visitor = ::async_xml::reader::TryFromVisitor<B, #name, #t, <#name as ::core::convert::TryFrom<#t>>::Error>;
                }
            });
        }
        attr::From::FromStr => {
            let name = &input.ident;
            return Ok(quote! {
                impl ::async_xml::reader::XmlFromStr for #name {}
            });
        }
    }

    match &input.data {
        syn::Data::Struct(d) => crate::xml_struct::expand_struct(container, &*input, d),
        _ => Err(vec![syn::Error::new_spanned(
            input,
            "only struct types implemented",
        )]),
    }
}
