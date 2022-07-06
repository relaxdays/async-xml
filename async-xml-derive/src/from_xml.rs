use crate::attr::{Container, FieldSource};
use crate::ctx::Ctx;
use crate::field::FieldData;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt};

pub fn expand_from_xml(input: &mut syn::DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let ctx = Ctx::new();
    let container = Container::from_attrs(&ctx, &input.attrs);
    ctx.check()?;

    let struct_data = match &input.data {
        syn::Data::Struct(d) => d,
        _ => panic!("only struct implemented"),
    };

    let name = &input.ident;
    let visitor_name = Ident::new(&format!("__{}Visitor", name), Span::call_site());

    let visitor_tag_name = if let Some(tag_name) = &container.tag_name {
        quote!(Some(#tag_name))
    } else {
        quote!(None)
    };

    let ctx = Ctx::new();
    let fields = struct_data
        .fields
        .iter()
        .flat_map(|f| FieldData::from_field(&ctx, f).ok())
        .collect::<Vec<_>>();
    if fields
        .iter()
        .filter(|f| f.attrs.source == FieldSource::Value)
        .count()
        > 1
    {
        let mut errs = fields
            .iter()
            .filter(|f| f.attrs.source == FieldSource::Value)
            .map(|f| syn::Error::new_spanned(&f.inner, "multiple fields sourcing from text"))
            .collect::<Vec<_>>();
        let mut err = errs.remove(0);
        for e in errs {
            err.combine(e);
        }
        ctx.syn_error(err);
    }
    ctx.check()?;
    let mut visitor_fields = TokenStream::new();
    visitor_fields.append_all(fields.iter().map(|f| f.visitor_struct_field_def()));
    let mut visitor_default = TokenStream::new();
    visitor_default.append_all(fields.iter().map(|f| f.visitor_struct_default_field()));
    let mut visitor_build_fields = TokenStream::new();
    visitor_build_fields.append_all(fields.iter().map(|f| f.visitor_build_field()));

    let mut visitor_visit_attr_match = TokenStream::new();
    let mut visitor_visit_child_match = TokenStream::new();
    let mut visitor_visit_value = TokenStream::new();
    for field in &fields {
        field.visitor_visit(
            &mut visitor_visit_attr_match,
            &mut visitor_visit_child_match,
            &mut visitor_visit_value,
        );
    }
    if visitor_visit_value.is_empty() {
        visitor_visit_value = quote! { Err(async_xml::Error::UnexpectedText) };
    }

    let mut visitor_build = TokenStream::new();
    visitor_build.append_all(fields.iter().map(|f| f.visitor_build()));

    let expanded = quote! {
        pub struct #visitor_name {
            #visitor_fields
        }
        impl Default for #visitor_name {
            fn default() -> Self {
                Self { #visitor_default }
            }
        }
        #[async_trait::async_trait]
        impl<B: tokio::io::AsyncBufRead + Send + Unpin> async_xml::Visitor<B> for #visitor_name {
            type Output = #name;

            fn start_name() -> Option<&'static str> {
                #visitor_tag_name
            }

            fn visit_attribute(&mut self, name: &str, value: &str) -> Result<(), async_xml::Error> {
                match name {
                    #visitor_visit_attr_match
                    _ => {
                        return Err(async_xml::Error::UnexpectedAttribute(name.into()));
                    }
                }
                #[allow(unreachable_code)]
                Ok(())
            }

            fn visit_text(&mut self, text: &str) -> Result<(), async_xml::Error> {
                #visitor_visit_value
            }

            async fn visit_child(
                &mut self,
                name: &str,
                reader: &mut async_xml::reader::PeekingReader<B>,
            ) -> Result<(), async_xml::Error> {
                match name {
                    #visitor_visit_child_match
                    _ => {
                        return Err(async_xml::Error::UnexpectedChild(name.into()));
                    }
                }
                #[allow(unreachable_code)]
                Ok(())
            }

            fn build(self) -> Result<Self::Output, async_xml::Error> {
                #visitor_build

                Ok(Self::Output {
                    #visitor_build_fields
                })
            }
        }
        impl<B: tokio::io::AsyncBufRead + Send + Unpin> async_xml::reader::FromXml<B> for #name {
            type Visitor = #visitor_name;
        }
    };

    Ok(expanded)
}
