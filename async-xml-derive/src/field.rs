use crate::attr::{Field, FieldSource};
use crate::ctx::Ctx;
use crate::path::{get_generic_arg, get_type_path_type, TypePathType};
use crate::xml_struct::StructType;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt};
use syn::{Lit, Type};

#[derive(Copy, Clone, PartialEq)]
pub enum FieldType {
    Named,
    Unnamed,
}

pub struct FieldData<'a> {
    pub inner: &'a syn::Field,
    pub attrs: Field,
    pub deserialization_type: Type,
    pub field_type: FieldType,
    pub type_type: TypePathType,
    pub visitor_field_name: Ident,
    pub visitor_field_type: Type,
    pub tag_name: Lit,
}

impl<'a> FieldData<'a> {
    pub fn from_field(ctx: &Ctx, field: &'a syn::Field, index: usize) -> Result<Self, ()> {
        let attrs = Field::from_attrs(ctx, &field.attrs);
        let inner_de_type = if let Some(ty) = attrs.from.as_ref() {
            ty
        } else {
            &field.ty
        };
        let type_type = get_type_path_type(inner_de_type);
        let visitor_field_type = match type_type {
            TypePathType::Any => syn::parse2(quote! { Option<#inner_de_type> }).unwrap(),
            TypePathType::Vec | TypePathType::Option | TypePathType::XmlNode => {
                inner_de_type.to_owned()
            }
        };
        let deserialization_type = match type_type {
            TypePathType::Any | TypePathType::Option | TypePathType::XmlNode => {
                inner_de_type.clone()
            }
            TypePathType::Vec => get_generic_arg(inner_de_type),
        };

        match (type_type, attrs.source) {
            // allow child elements to be read into a vec
            (TypePathType::Vec, FieldSource::Child) => {}
            // disallow non-xmlnode for remains
            (t, s) if (t == TypePathType::XmlNode) ^ (s == FieldSource::Remains) => {
                ctx.error_spanned_by(field, "remains must be used with XmlNode");
                return Err(());
            }
            // allow xmlnode remains
            (TypePathType::XmlNode, FieldSource::Remains) => {}
            // allow "standard" types for all sources
            (TypePathType::Any, _) => {}
            // allow option types for all sources
            (TypePathType::Option, _) => {}
            _ => {
                ctx.error_spanned_by(field, "field type invalid for this source");
                return Err(());
            }
        }

        let (visitor_field_name, field_type) = if let Some(ident) = field.ident.as_ref() {
            (ident.to_owned(), FieldType::Named)
        } else {
            (
                Ident::new(&format!("__{}", index), Span::call_site()),
                FieldType::Unnamed,
            )
        };
        let tag_name = if let Some(rename) = &attrs.rename {
            syn::LitStr::new(rename, Span::call_site())
        } else {
            syn::LitStr::new(&visitor_field_name.to_string(), Span::call_site())
        };
        let tag_name = syn::Lit::Str(tag_name);

        Ok(Self {
            inner: field,
            attrs,
            deserialization_type,
            field_type,
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
            TypePathType::XmlNode => {
                quote! { #name: Default::default(), }
            }
        }
    }

    pub fn visitor_visit(
        &self,
        visit_attr: &mut TokenStream,
        visit_child: &mut TokenStream,
        visit_text: &mut TokenStream,
        visit_tag: &mut TokenStream,
    ) {
        let tag = &self.tag_name;
        let ident = &self.visitor_field_name;
        let ty = &self.deserialization_type;
        match self.attrs.source {
            FieldSource::Attribute => {
                visit_attr.append_all(quote! {
                   #tag => {
                        let mut visitor = <#ty as async_xml::reader::FromXml<B>>::Visitor::default();
                        <<#ty as async_xml::reader::FromXml<B>>::Visitor as async_xml::reader::Visitor<B>>::visit_text(&mut visitor, value)?;
                        let val = <<#ty as async_xml::reader::FromXml<B>>::Visitor as async_xml::reader::Visitor<B>>::build(visitor)?;
                        self.#ident.replace(val);
                    }
                });
            }
            FieldSource::Value => {
                *visit_text = quote! {
                    let mut visitor = <#ty as async_xml::reader::FromXml<B>>::Visitor::default();
                    <<#ty as async_xml::reader::FromXml<B>>::Visitor as async_xml::reader::Visitor<B>>::visit_text(&mut visitor, text)?;
                    let val = <<#ty as async_xml::reader::FromXml<B>>::Visitor as async_xml::reader::Visitor<B>>::build(visitor)?;
                    if self.#ident.replace(val).is_some() {
                        Err(async_xml::Error::DoubleText)
                    } else {
                        Ok(())
                    }
                };
            }
            FieldSource::Remains => {
                // assume we are the last field so we are the very last match arm for attr and child
                visit_attr.append_all(quote! {
                    _ => {
                        <#ty as async_xml::reader::Visitor<B>>::visit_attribute(&mut self.#ident, name, value)?;
                    }
                });
                visit_child.append_all(quote! {
                    _ => {
                        <#ty as async_xml::reader::Visitor<B>>::visit_child(&mut self.#ident, name, reader).await?;
                    }
                });
                visit_tag.append_all(quote! {
                    <#ty as async_xml::reader::Visitor<B>>::visit_tag(&mut self.#ident, name)?;
                })
            }
            FieldSource::Child => match self.type_type {
                TypePathType::Vec => {
                    visit_child.append_all(quote! {
                        #tag => {
                            self.#ident.push(reader.deserialize::<#ty>().await?);
                        }
                    });
                }
                TypePathType::Any => {
                    visit_child.append_all(quote! {
                        #tag => {
                            if self.#ident.is_some() {
                                return Err(async_xml::Error::DoubleChild(name.into()));
                            }
                            self.#ident = Some(reader.deserialize::<#ty>().await?);
                        }
                    });
                }
                TypePathType::Option => {
                    visit_child.append_all(quote! {
                        #tag => {
                            if self.#ident.is_some() {
                                return Err(async_xml::Error::DoubleChild(name.into()));
                            }
                            self.#ident = reader.deserialize::<#ty>().await?;
                        }
                    });
                }
                TypePathType::XmlNode => unreachable!(),
            },
        }
    }

    pub fn visitor_build(&self) -> TokenStream {
        match self.type_type {
            TypePathType::Vec | TypePathType::Option | TypePathType::XmlNode => TokenStream::new(),
            TypePathType::Any => self.build_default(),
        }
    }

    pub fn visitor_build_field(&self, struct_type: &StructType) -> TokenStream {
        let name = &self.visitor_field_name;
        let val = match self.type_type {
            TypePathType::Any => {
                quote! { #name.into() }
            }
            TypePathType::Vec | TypePathType::Option => {
                quote! { self.#name.into() }
            }
            TypePathType::XmlNode => {
                quote! { self.#name }
            }
        };
        match struct_type {
            StructType::Normal => quote! { #name: #val, },
            StructType::Newtype | StructType::Tuple => quote! { #val, },
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
            FieldSource::Remains => unreachable!("remains cannot fail"),
        }
    }
}
