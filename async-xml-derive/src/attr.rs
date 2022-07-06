use crate::ctx::Ctx;
use crate::respan::respan;
use crate::symbol::*;
use proc_macro2::TokenStream;
use syn::{Attribute, Meta, NestedMeta};

/// container attributes
pub struct Container {
    pub tag_name: Option<String>,
    pub use_from_str: bool,
}

impl Container {
    pub fn from_attrs(ctx: &Ctx, attrs: &Vec<Attribute>) -> Self {
        let mut tag_name = None;
        let mut use_from_str = false;

        for attr in attrs {
            if attr.path != FROM_XML {
                continue;
            }
            match attr.parse_meta() {
                Ok(Meta::List(meta)) => {
                    for nested in meta.nested {
                        match nested {
                            NestedMeta::Meta(Meta::NameValue(m)) if m.path == TAG_NAME => {
                                let str = get_lit_str(ctx, &m.lit);
                                if let Ok(str) = str {
                                    if tag_name.replace(str.value()).is_some() {
                                        ctx.error_spanned_by(m, "tag name given multiple times");
                                    }
                                }
                            }
                            NestedMeta::Meta(Meta::Path(m)) if m == USE_FROM_STR => {
                                use_from_str = true;
                            }
                            NestedMeta::Meta(meta) => {
                                ctx.error_spanned_by(meta, "unexpected meta");
                            }
                            NestedMeta::Lit(lit) => {
                                ctx.error_spanned_by(lit, "unexpected literal");
                            }
                        }
                    }
                }
                Ok(other) => {
                    ctx.error_spanned_by(other, "expected #[from_xml(...)]");
                }
                Err(e) => {
                    ctx.syn_error(e);
                }
            }
        }

        Self {
            tag_name,
            use_from_str,
        }
    }
}

pub struct Field {
    pub source: FieldSource,
    pub default: Default,
    pub rename: Option<String>,
    pub from: Option<syn::Type>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum FieldSource {
    Attribute,
    Child,
    Value,
}

pub enum Default {
    None,
    Default,
    Path(syn::ExprPath),
}

impl Default {
    pub fn is_none(&self) -> bool {
        match self {
            Default::None => true,
            Default::Default | Default::Path(_) => false,
        }
    }
}

impl Field {
    pub fn from_attrs(ctx: &Ctx, attrs: &Vec<Attribute>) -> Self {
        let mut source = None;
        let mut default = None;
        let mut rename = None;
        let mut from = None;

        for attr in attrs {
            if attr.path != FROM_XML {
                continue;
            }
            match attr.parse_meta() {
                Ok(Meta::List(meta)) => {
                    for nested in meta.nested {
                        match nested {
                            NestedMeta::Meta(Meta::Path(m)) if m == ATTRIBUTE => {
                                if source.replace(FieldSource::Attribute).is_some() {
                                    ctx.error_spanned_by(m, "source already specified");
                                }
                            }
                            NestedMeta::Meta(Meta::Path(m)) if m == VALUE => {
                                if source.replace(FieldSource::Value).is_some() {
                                    ctx.error_spanned_by(m, "source already specified");
                                }
                            }
                            NestedMeta::Meta(Meta::Path(m)) if m == CHILD => {
                                if source.replace(FieldSource::Child).is_some() {
                                    ctx.error_spanned_by(m, "source already specified");
                                }
                            }
                            NestedMeta::Meta(Meta::Path(m)) if m == DEFAULT => {
                                if default.replace(Default::Default).is_some() {
                                    ctx.error_spanned_by(m, "default already specified");
                                }
                            }
                            NestedMeta::Meta(Meta::NameValue(m)) if m.path == DEFAULT => {
                                let path = get_lit_str_as_expr_path(ctx, &m.lit);
                                if let Ok(path) = path {
                                    if default.replace(Default::Path(path)).is_some() {
                                        ctx.error_spanned_by(m, "default already specified");
                                    }
                                }
                            }
                            NestedMeta::Meta(Meta::NameValue(m)) if m.path == RENAME => {
                                let str = get_lit_str(ctx, &m.lit);
                                if let Ok(str) = str {
                                    if rename.replace(str.value()).is_some() {
                                        ctx.error_spanned_by(m, "rename already specified");
                                    }
                                }
                            }
                            NestedMeta::Meta(Meta::NameValue(m)) if m.path == FROM => {
                                let _type = get_lit_str_as_type(ctx, &m.lit);
                                if let Ok(_type) = _type {
                                    if from.replace(_type).is_some() {
                                        ctx.error_spanned_by(m, "from already specified");
                                    }
                                }
                            }
                            NestedMeta::Meta(meta) => {
                                ctx.error_spanned_by(meta, "unexpected meta");
                            }
                            NestedMeta::Lit(lit) => {
                                ctx.error_spanned_by(lit, "unexpected literal");
                            }
                        }
                    }
                }
                Ok(other) => {
                    ctx.error_spanned_by(other, "expected #[from_xml(...)]");
                }
                Err(e) => {
                    ctx.syn_error(e);
                }
            }
        }

        Self {
            source: source.unwrap_or(FieldSource::Value),
            default: default.unwrap_or(Default::None),
            rename,
            from,
        }
    }
}

fn get_lit_str<'a>(ctx: &Ctx, lit: &'a syn::Lit) -> Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(lit_str) = lit {
        Ok(lit_str)
    } else {
        ctx.error_spanned_by(lit, "expected string literal");
        Err(())
    }
}

fn get_lit_str_as_expr_path(ctx: &Ctx, lit: &syn::Lit) -> Result<syn::ExprPath, ()> {
    let str = get_lit_str(ctx, lit)?;
    parse_lit_str(str).map_err(|_| {
        ctx.error_spanned_by(lit, "failed to parse path");
    })
}

fn get_lit_str_as_type(ctx: &Ctx, lit: &syn::Lit) -> Result<syn::Type, ()> {
    let str = get_lit_str(ctx, lit)?;
    parse_lit_str(str).map_err(|_| {
        ctx.error_spanned_by(lit, "failed to parse path");
    })
}

fn spanned_tokens(s: &syn::LitStr) -> syn::parse::Result<TokenStream> {
    let stream = syn::parse_str(&s.value())?;
    Ok(respan(stream, s.span()))
}

fn parse_lit_str<T>(s: &syn::LitStr) -> syn::parse::Result<T>
where
    T: syn::parse::Parse,
{
    let tokens = spanned_tokens(s)?;
    syn::parse2(tokens)
}
