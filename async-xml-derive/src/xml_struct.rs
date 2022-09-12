use crate::{attr::FieldSource, ctx::Ctx, field::FieldData};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt};

#[derive(Copy, Clone, PartialEq)]
pub enum StructType {
    /// A normal, braced struct with named fields
    Normal,
    /// A tuple struct with exactly one field
    Newtype,
    /// A tuple struct
    Tuple,
}

pub struct StructContainer<'a> {
    attr: crate::attr::Container,
    /// Name of the input struct
    name: Ident,
    /// Name of the generated visitor
    visitor_name: Ident,
    /// Field data
    fields: Vec<FieldData<'a>>,
    /// Value for expected tag name
    tag_name: TokenStream,
    struct_type: StructType,
}

impl<'a> StructContainer<'a> {
    pub fn new(
        container: crate::attr::Container,
        input: &syn::DeriveInput,
        data: &'a syn::DataStruct,
    ) -> Result<Self, Vec<syn::Error>> {
        let name = input.ident.clone();
        let visitor_name = Ident::new(&format!("__{}Visitor", name), Span::call_site());

        let tag_name = if let Some(tag_name) = &container.tag_name {
            quote!(Some(#tag_name))
        } else {
            quote!(None)
        };

        let ctx = Ctx::new();
        let mut fields = data
            .fields
            .iter()
            .enumerate()
            .flat_map(|(i, f)| FieldData::from_field(&ctx, f, i).ok())
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
        let remain_count = fields
            .iter()
            .filter(|f| f.attrs.source == FieldSource::Remains)
            .count();
        if remain_count > 1 {
            let mut errs = fields
                .iter()
                .filter(|f| f.attrs.source == FieldSource::Remains)
                .map(|f| syn::Error::new_spanned(&f.inner, "multiple fields sourcing from remains"))
                .collect::<Vec<_>>();
            let mut err = errs.remove(0);
            for e in errs {
                err.combine(e);
            }
            ctx.syn_error(err);
        }
        ctx.check()?;
        let struct_type = match &data.fields {
            syn::Fields::Named(_) => StructType::Normal,
            syn::Fields::Unnamed(f) => {
                if f.unnamed.len() == 1 {
                    StructType::Newtype
                } else {
                    StructType::Tuple
                }
            }
            syn::Fields::Unit => unreachable!("Unit variant invalid for structs"),
        };
        // make sure the remains field is last
        if remain_count == 1 {
            let idx = fields
                .iter()
                .enumerate()
                .find(|(_, f)| f.attrs.source == FieldSource::Remains)
                .unwrap()
                .0;
            let remains_field = fields.remove(idx);
            fields.push(remains_field);
        }

        Ok(Self {
            attr: container,
            name,
            visitor_name,
            fields,
            tag_name,
            struct_type,
        })
    }
}

pub fn expand_struct(
    container: crate::attr::Container,
    input: &syn::DeriveInput,
    data: &syn::DataStruct,
) -> Result<TokenStream, Vec<syn::Error>> {
    let container = StructContainer::new(container, input, data)?;

    let visitor_name = &container.visitor_name;
    let name = &container.name;
    let tag_name = &container.tag_name;
    let vis = &input.vis;

    let mut visitor_fields = TokenStream::new();
    visitor_fields.append_all(
        container
            .fields
            .iter()
            .map(|f| f.visitor_struct_field_def()),
    );
    let mut visitor_default = TokenStream::new();
    visitor_default.append_all(
        container
            .fields
            .iter()
            .map(|f| f.visitor_struct_default_field()),
    );
    let mut visitor_build_fields = TokenStream::new();
    visitor_build_fields.append_all(
        container
            .fields
            .iter()
            .map(|f| f.visitor_build_field(&container.struct_type)),
    );

    let mut visitor_visit_attr_match = TokenStream::new();
    let mut visitor_visit_attr_match_any = TokenStream::new();
    let mut visitor_visit_child_match = TokenStream::new();
    let mut visitor_visit_child_match_any = TokenStream::new();
    let mut visitor_visit_value = TokenStream::new();
    let mut visitor_visit_tag = TokenStream::new();
    for field in &container.fields {
        field.visitor_visit(
            &mut visitor_visit_attr_match,
            &mut visitor_visit_attr_match_any,
            &mut visitor_visit_child_match,
            &mut visitor_visit_child_match_any,
            &mut visitor_visit_value,
            &mut visitor_visit_tag,
        );
    }
    if container.attr.allow_unknown_text {
        visitor_visit_value.append_all(quote! {
            #[allow(unreachable_code)]
            Ok(())
        });
    } else {
        visitor_visit_value.append_all(quote! {
            #[allow(unreachable_code)]
            Err(::async_xml::Error::UnexpectedText)
        });
    }

    let mut visitor_build = TokenStream::new();
    visitor_build.append_all(container.fields.iter().map(|f| f.visitor_build()));

    let visitor_doc = format!(
        "A [`Visitor`](::async_xml::Visitor) for deserializing a [`{}`]",
        name
    );
    let visitor = quote! {
        #[doc = #visitor_doc]
        #vis struct #visitor_name<B> where B: tokio::io::AsyncBufRead + Unpin {
            #visitor_fields
            _phantom: core::marker::PhantomData<B>,
        }
        impl<B: ::tokio::io::AsyncBufRead + Unpin> Default for #visitor_name<B> {
            fn default() -> Self {
                Self {
                    #visitor_default
                    _phantom: core::marker::PhantomData,
                }
            }
        }
    };
    let mut visitor_impl: syn::ItemImpl = syn::parse2(quote! {
        #[async_trait::async_trait(?Send)]
        impl<B: ::tokio::io::AsyncBufRead + Unpin> ::async_xml::Visitor<B> for #visitor_name<B> {
            type Output = #name;
        }
    })
    .unwrap();

    let visitor_fn_build = match container.struct_type {
        StructType::Normal => {
            quote! {
                fn build(self) -> ::core::result::Result<#name, ::async_xml::Error> {
                    #visitor_build

                    Ok(#name {
                        #visitor_build_fields
                    })
                }
            }
        }
        StructType::Newtype | StructType::Tuple => {
            quote! {
                fn build(self) -> ::core::result::Result<#name, ::async_xml::Error> {
                    #visitor_build

                    Ok(#name(
                        #visitor_build_fields
                    ))
                }
            }
        }
    };
    visitor_impl
        .items
        .push(syn::parse2(visitor_fn_build).unwrap());
    visitor_impl.items.push(
        syn::parse2(quote! {
            fn start_name() -> Option<&'static str> {
                #tag_name
            }
        })
        .unwrap(),
    );
    visitor_impl.items.push(
        syn::parse2(quote! {
            fn visit_tag(&mut self, name: &str) -> ::core::result::Result<(), ::async_xml::Error> {
                #visitor_visit_tag
                Ok(())
            }
        })
        .unwrap(),
    );
    let unknown_attr = if container.attr.allow_unknown_attributes {
        TokenStream::new()
    } else {
        quote! { return Err(::async_xml::Error::UnexpectedAttribute(name.into())); }
    };
    visitor_impl.items.push(
        syn::parse2(quote! {
            fn visit_attribute(&mut self, name: &str, value: &str) -> ::core::result::Result<(), ::async_xml::Error> {
                match name {
                    #visitor_visit_attr_match
                    _ => {
                        #visitor_visit_attr_match_any
                        #unknown_attr
                    }
                }
                #[allow(unreachable_code)]
                Ok(())
            }
        })
        .unwrap(),
    );
    visitor_impl.items.push(
        syn::parse2(quote! {
            fn visit_text(&mut self, text: &str) -> ::core::result::Result<(), ::async_xml::Error> {

                #[allow(unreachable_code)]
                {
                    #visitor_visit_value
                }
            }
        })
        .unwrap(),
    );
    let unknown_child = if container.attr.allow_unknown_children {
        quote! { reader.skip_element().await?; }
    } else {
        quote! { return Err(::async_xml::Error::UnexpectedChild(name.into())); }
    };
    visitor_impl.items.push(
        syn::parse2(quote! {
            async fn visit_child(
                &mut self,
                name: &str,
                reader: &mut ::async_xml::reader::PeekingReader<B>,
            ) -> ::core::result::Result<(), ::async_xml::Error> {
                match name {
                    #visitor_visit_child_match
                    _ => {
                        #visitor_visit_child_match_any
                        #unknown_child
                    }
                }
                #[allow(unreachable_code)]
                Ok(())
            }
        })
        .unwrap(),
    );

    let expanded = quote! {
        #visitor
        #visitor_impl
        impl<B: ::tokio::io::AsyncBufRead + Unpin> ::async_xml::reader::FromXml<B> for #name {
            type Visitor = #visitor_name<B>;
        }
    };

    Ok(expanded)
}
