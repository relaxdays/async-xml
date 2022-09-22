use crate::attr::{self, Field, FieldSource};
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
    /// source field
    pub inner: &'a syn::Field,
    /// attributes on the source field
    pub attrs: Field,
    /// the type to actually deserialize
    pub deserialization_type: Type,
    /// whether this is a named or unnamed field
    pub field_type: FieldType,
    /// the kind of type this field is
    pub type_type: TypePathType,
    /// name of the corresponding field in the generated visitor struct
    pub visitor_field_name: Ident,
    /// type of the corresponding field in the generated visitor struct
    pub visitor_field_type: Type,
    /// xml element name to deserialize into this field
    pub tag_name: Lit,
}

impl<'a> FieldData<'a> {
    pub fn from_field(ctx: &Ctx, field: &'a syn::Field, index: usize) -> Result<Self, ()> {
        let attrs = Field::from_attrs(ctx, &field.attrs);
        let inner_de_type = match &attrs.from {
            attr::From::Default => &field.ty,
            attr::From::From(t) => t,
            attr::From::FromStr => unimplemented!(),
            attr::From::TryFrom(_t) => unimplemented!(),
        };
        let type_type = get_type_path_type(inner_de_type);
        let type_type = match (attrs.source, type_type) {
            (FieldSource::Flatten, TypePathType::Option) => TypePathType::OptionalNode,
            (FieldSource::Flatten, TypePathType::Any) => TypePathType::XmlNode,
            (FieldSource::Child, TypePathType::XmlNode) => TypePathType::Any,
            _ => type_type,
        };
        let deserialization_type = match type_type {
            TypePathType::Any | TypePathType::Option | TypePathType::XmlNode => {
                inner_de_type.clone()
            }
            TypePathType::Vec | TypePathType::OptionalNode => get_generic_arg(inner_de_type),
        };
        let visitor_field_type = match type_type {
            TypePathType::Any => syn::parse2(quote! { Option<#inner_de_type> }).unwrap(),
            TypePathType::Vec | TypePathType::Option => inner_de_type.to_owned(),
            TypePathType::XmlNode | TypePathType::OptionalNode => syn::parse2(
                quote! { <#deserialization_type as ::async_xml::reader::FromXml<B>>::Visitor },
            )
            .unwrap(),
        };

        match (type_type, attrs.source) {
            // allow child elements to be read into a vec
            (TypePathType::Vec, FieldSource::Child) => {}
            // allow xmlnode remains and flatten
            (TypePathType::XmlNode, FieldSource::Remains) => {}
            (TypePathType::XmlNode, FieldSource::Flatten) => {}
            // allow optionalnode only for flatten
            (TypePathType::OptionalNode, FieldSource::Flatten) => {}
            // allow "standard" types for all sources
            (TypePathType::Any, _) => {}
            // allow option types for all sources
            (TypePathType::Option, _) => {}
            // disallow everything else
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
            TypePathType::XmlNode | TypePathType::OptionalNode => {
                quote! { #name: Default::default(), }
            }
        }
    }

    pub fn visitor_visit(
        &self,
        visit_attr: &mut TokenStream,
        visit_attr_any: &mut TokenStream,
        visit_child: &mut TokenStream,
        visit_child_any: &mut TokenStream,
        visit_text: &mut TokenStream,
        visit_tag: &mut TokenStream,
    ) {
        let tag = &self.tag_name;
        let ident = &self.visitor_field_name;
        let ty = &self.deserialization_type;
        let field_ty = &self.visitor_field_type;
        match self.attrs.source {
            FieldSource::Attribute => {
                let build_val = quote! {
                    let mut visitor = <#ty as ::async_xml::reader::FromXml<B>>::Visitor::default();
                    <<#ty as ::async_xml::reader::FromXml<B>>::Visitor as ::async_xml::reader::Visitor<B>>::visit_text(&mut visitor, value)?;
                    let val = <<#ty as ::async_xml::reader::FromXml<B>>::Visitor as ::async_xml::reader::Visitor<B>>::build(visitor)?;
                };
                match self.type_type {
                    TypePathType::Any => {
                        visit_attr.append_all(quote! {
                           #tag => {
                                #build_val
                                self.#ident.replace(val);
                            }
                        });
                    }
                    TypePathType::Option => {
                        visit_attr.append_all(quote! {
                           #tag => {
                                #build_val
                                self.#ident = val;
                            }
                        });
                    }
                    TypePathType::Vec | TypePathType::XmlNode | TypePathType::OptionalNode => {
                        unreachable!("vec and xmlnode aren't valid for attribute")
                    }
                }
            }
            FieldSource::Value => {
                let build_val = quote! {
                    let mut visitor = <#ty as ::async_xml::reader::FromXml<B>>::Visitor::default();
                    <<#ty as ::async_xml::reader::FromXml<B>>::Visitor as ::async_xml::reader::Visitor<B>>::visit_text(&mut visitor, text)?;
                    let val = <<#ty as ::async_xml::reader::FromXml<B>>::Visitor as ::async_xml::reader::Visitor<B>>::build(visitor)?;
                };
                match self.type_type {
                    TypePathType::Any => {
                        visit_text.append_all(quote! {
                            #build_val
                            return if self.#ident.replace(val).is_some() {
                                Err(::async_xml::Error::DoubleText)
                            } else {
                                Ok(())
                            };
                        });
                    }
                    TypePathType::Option => {
                        visit_text.append_all(quote! {
                            if self.#ident.is_some() {
                                return Err(::async_xml::Error::DoubleText);
                            }
                            #build_val
                            self.#ident = val;
                        });
                    }
                    TypePathType::Vec | TypePathType::XmlNode | TypePathType::OptionalNode => {
                        unreachable!("vec and xmlnode aren't valid for value")
                    }
                }
            }
            FieldSource::Remains | FieldSource::Flatten => {
                visit_attr_any.append_all(quote! {
                    // ignore unexpected attribute errors here to maybe pass on to remains/other flattens
                    match <#field_ty as ::async_xml::reader::Visitor<B>>::visit_attribute(&mut self.#ident, name, value) {
                        Err(::async_xml::Error::UnexpectedAttribute(_)) => {},
                        r => return r,
                    }
                });
                visit_child_any.append_all(quote! {
                    // ignore unexpected child errors here to maybe pass on to remains/other flattens
                    match <#field_ty as ::async_xml::reader::Visitor<B>>::visit_child(&mut self.#ident, name, reader).await {
                        Err(::async_xml::Error::UnexpectedChild(_)) => {},
                        r => return r,
                    }
                });
                visit_tag.append_all(quote! {
                    <#field_ty as ::async_xml::reader::Visitor<B>>::visit_tag(&mut self.#ident, name)?;
                });
                visit_text.append_all(quote! {
                    // ignore unexpected text errors here to maybe pass on to remains/other flattens
                    match <#field_ty as ::async_xml::reader::Visitor<B>>::visit_text(&mut self.#ident, text) {
                        Err(::async_xml::Error::UnexpectedText) => {},
                        r => return r,
                    }
                });
            }
            FieldSource::Child => match self.type_type {
                TypePathType::Vec => {
                    visit_child.append_all(quote! {
                        #tag => {
                            self.#ident.push(reader.deserialize::<#ty>().await.map_err(|e| {
                                ::async_xml::Error::InnerDeserialiaztionError(#tag.into(), Box::new(e))
                            })?);
                        }
                    });
                }
                TypePathType::Any => {
                    visit_child.append_all(quote! {
                        #tag => {
                            if self.#ident.is_some() {
                                return Err(::async_xml::Error::DoubleChild(name.into()));
                            }
                            self.#ident = Some(reader.deserialize::<#ty>().await.map_err(|e| {
                                ::async_xml::Error::InnerDeserialiaztionError(#tag.into(), Box::new(e))
                            })?);
                        }
                    });
                }
                TypePathType::Option => {
                    visit_child.append_all(quote! {
                        #tag => {
                            if self.#ident.is_some() {
                                return Err(::async_xml::Error::DoubleChild(name.into()));
                            }
                            self.#ident = reader.deserialize::<#ty>().await.map_err(|e| {
                                ::async_xml::Error::InnerDeserialiaztionError(#tag.into(), Box::new(e))
                            })?;
                        }
                    });
                }
                TypePathType::XmlNode | TypePathType::OptionalNode => unreachable!(),
            },
        }
    }

    pub fn visitor_build(&self) -> TokenStream {
        match self.type_type {
            TypePathType::Vec | TypePathType::Option => TokenStream::new(),
            TypePathType::Any => self.build_default(),
            TypePathType::OptionalNode => {
                let name = &self.visitor_field_name;
                let ty = &self.visitor_field_type;
                quote! {
                    let #name = match <#ty as ::async_xml::reader::Visitor<B>>::build(self.#name) {
                        Ok(#name) => Some(#name),
                        Err(e) => {
                            ::tracing::trace!("discarding flattened build error: {:?}", e);
                            None
                        }
                    };
                }
            }
            TypePathType::XmlNode => {
                let name = &self.visitor_field_name;
                let ty = &self.visitor_field_type;
                quote! {
                    let #name = <#ty as ::async_xml::reader::Visitor<B>>::build(self.#name)?;
                }
            }
        }
    }

    pub fn visitor_build_field(&self, struct_type: &StructType) -> TokenStream {
        let name = &self.visitor_field_name;
        let val = match self.type_type {
            TypePathType::Any => {
                quote! { #name.into() }
            }
            TypePathType::Vec | TypePathType::Option => {
                quote! { self.#name }
            }
            TypePathType::XmlNode | TypePathType::OptionalNode => {
                quote! { #name }
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
                    let e = #build_error;
                    ::tracing::debug!("throwing build error: {:?}", e);
                    return Err(e);
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
            FieldSource::Attribute => quote! {::async_xml::Error::MissingAttribute(#tag.into())},
            FieldSource::Child => quote! {::async_xml::Error::MissingChild(#tag.into())},
            FieldSource::Value => quote! {::async_xml::Error::MissingText},
            FieldSource::Remains | FieldSource::Flatten => {
                unreachable!("remains/flatten cannot fail")
            }
        }
    }
}
