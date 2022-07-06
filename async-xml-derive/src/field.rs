use crate::attr::{Field, FieldSource};
use crate::ctx::Ctx;
use crate::path::{get_type_path_type, TypePathType};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Lit;

pub struct FieldData<'a> {
    pub inner: &'a syn::Field,
    pub attrs: Field,
    pub type_type: TypePathType,
    pub visitor_field_name: TokenStream,
    pub visitor_field_type: TokenStream,
    pub tag_name: Lit,
}

impl<'a> FieldData<'a> {
    pub fn from_field(ctx: &Ctx, field: &'a syn::Field) -> Result<Self, ()> {
        let attrs = Field::from_attrs(ctx, &field.attrs);
        let type_type = get_type_path_type(&field.ty);

        match (type_type, attrs.source) {
            // allow child elements to be read into a vec
            (TypePathType::Vec, FieldSource::Child) => {}
            // allow "standard" types for all sources
            (TypePathType::Any, _) => {}
            // allow option types for all sources
            (TypePathType::Option, _) => {}
            _ => {
                ctx.error_spanned_by(field, "field type invalid for this source");
                return Err(());
            }
        }

        let visitor_field_name = field.ident.as_ref().unwrap().to_token_stream();
        let ty = &field.ty;
        let visitor_field_type = match type_type {
            TypePathType::Any => {
                quote! { Option<#ty> }
            }
            TypePathType::Vec | TypePathType::Option => {
                quote! { #ty }
            }
        };
        let tag_name = if let Some(rename) = &attrs.rename {
            syn::LitStr::new(rename, Span::call_site())
        } else {
            syn::LitStr::new(
                &field.ident.as_ref().unwrap().to_string(),
                Span::call_site(),
            )
        };
        let tag_name = syn::Lit::Str(tag_name);

        Ok(Self {
            inner: field,
            attrs,
            type_type,
            visitor_field_name,
            visitor_field_type,
            tag_name,
        })
    }

    pub fn visitor_struct_field_def(&self) -> TokenStream {
        let name = &self.visitor_field_name;
        let ty = &self.visitor_field_type;
        quote! { #name: #ty, }
    }

    pub fn visitor_struct_default_field(&self) -> TokenStream {
        let name = &self.visitor_field_name;
        match self.type_type {
            TypePathType::Vec => {
                quote! { #name: Vec::new(), }
            }
            TypePathType::Any | TypePathType::Option => {
                quote! { #name: None, }
            }
        }
    }

    pub fn visitor_visit(
        &self,
        visit_attr: &mut TokenStream,
        visit_child: &mut TokenStream,
        visit_text: &mut TokenStream,
    ) {
        let tag = &self.tag_name;
        let ident = &self.visitor_field_name;
        match self.attrs.source {
            FieldSource::Attribute => {
                visit_attr.append_all(quote! {
                   #tag => { self.#ident.replace(value.into()); }
                });
            }
            FieldSource::Value => {
                *visit_text = quote! {
                    if self.#ident.replace(text.into()).is_some() {
                        Err(async_xml::Error::DoubleText)
                    } else {
                        Ok(())
                    }
                };
            }
            FieldSource::Child => match self.type_type {
                TypePathType::Vec => {
                    visit_child.append_all(quote! {
                        #tag => {
                            self.#ident.push(reader.deserialize().await?);
                        }
                    });
                }
                TypePathType::Any => {
                    visit_child.append_all(quote! {
                        #tag => {
                            if self.#ident.is_some() {
                                return Err(async_xml::Error::DoubleChild(name.into()));
                            }
                            self.#ident = Some(reader.deserialize().await?);
                        }
                    });
                }
                TypePathType::Option => {
                    visit_child.append_all(quote! {
                        #tag => {
                            if self.#ident.is_some() {
                                return Err(async_xml::Error::DoubleChild(name.into()));
                            }
                            self.#ident = reader.deserialize().await?;
                        }
                    });
                }
            },
        }
    }

    pub fn visitor_build(&self) -> TokenStream {
        match self.type_type {
            TypePathType::Vec | TypePathType::Option => TokenStream::new(),
            TypePathType::Any => self.build_default(),
        }
    }

    pub fn visitor_build_field(&self) -> TokenStream {
        let name = &self.visitor_field_name;
        match self.type_type {
            TypePathType::Any => {
                quote! { #name, }
            }
            TypePathType::Vec | TypePathType::Option => {
                quote! { #name: self.#name, }
            }
        }
    }

    fn build_default(&self) -> TokenStream {
        let name = &self.visitor_field_name;
        if self.attrs.default.is_none() {
            let build_error = self.build_error();
            quote! {
                let #name = if let Some(#name) = self.#name {
                    #name
                } else {
                    return Err(#build_error);
                };
            }
        } else {
            let default = match &self.attrs.default {
                crate::attr::Default::Default => syn::parse_str("Default::default").unwrap(),
                crate::attr::Default::Path(path) => path.clone(),
                crate::attr::Default::None => unreachable!(),
            };
            quote! { let #name = self.#name.unwrap_or_else(#default); }
        }
    }

    fn build_error(&self) -> TokenStream {
        let tag = &self.tag_name;
        match self.attrs.source {
            FieldSource::Attribute => quote! {async_xml::Error::MissingAttribute(#tag.into())},
            FieldSource::Child => quote! {async_xml::Error::MissingChild(#tag.into())},
            FieldSource::Value => quote! {async_xml::Error::MissingText},
        }
    }
}
