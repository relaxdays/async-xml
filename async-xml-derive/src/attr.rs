use crate::ctx::Ctx;
use crate::respan::respan;
use crate::symbol::*;
use proc_macro2::TokenStream;
use syn::{Attribute, Meta, NestedMeta};

pub enum From {
    Default,
    FromStr,
    From(syn::Type),
    TryFrom(syn::Type),
}

/// container attributes
pub struct Container {
    pub tag_name: Option<String>,
    pub from: From,
    pub allow_unknown_children: bool,
    pub allow_unknown_attributes: bool,
    pub allow_unknown_text: bool,
}

impl Container {
    pub fn from_attrs(ctx: &Ctx, attrs: &Vec<Attribute>) -> Self {
        let mut tag_name = None;
        let mut from = None;
        let mut allow_unknown_children = false;
        let mut allow_unknown_attributes = false;
        let mut allow_unknown_text = false;

        for attr in attrs {
            if attr.path != ASYNC_XML {
                continue;
            }
            match attr.parse_meta() {
                Ok(Meta::List(meta)) => {
                    for nested in meta.nested {
                        match nested {
                            NestedMeta::Meta(Meta::NameValue(m)) if m.path == RENAME => {
                                let str = get_lit_str(ctx, &m.lit);
                                if let Ok(str) = str {
                                    if tag_name.replace(str.value()).is_some() {
                                        ctx.error_spanned_by(m, "tag name given multiple times");
                                    }
                                }
                            }
                            NestedMeta::Meta(Meta::Path(m)) if m == FROM_STR => {
                                if from.replace(From::FromStr).is_some() {
                                    ctx.error_spanned_by(m, "from already specified");
                                }
                            }
                            NestedMeta::Meta(Meta::NameValue(m)) if m.path == FROM => {
                                let _type = get_lit_str_as_type(ctx, &m.lit);
                                if let Ok(_type) = _type {
                                    if from.replace(From::From(_type)).is_some() {
                                        ctx.error_spanned_by(m, "from already specified");
                                    }
                                }
                            }
                            NestedMeta::Meta(Meta::NameValue(m)) if m.path == TRY_FROM => {
                                let _type = get_lit_str_as_type(ctx, &m.lit);
                                if let Ok(_type) = _type {
                                    if from.replace(From::TryFrom(_type)).is_some() {
                                        ctx.error_spanned_by(m, "from already specified");
                                    }
                                }
                            }
                            NestedMeta::Meta(Meta::Path(m)) if m == ALLOW_UNKNOWN_CHILDREN => {
                                allow_unknown_children = true;
                            }
                            NestedMeta::Meta(Meta::Path(m)) if m == ALLOW_UNKNOWN_ATTRIBUTES => {
                                allow_unknown_attributes = true;
                            }
                            NestedMeta::Meta(Meta::Path(m)) if m == ALLOW_UNKNOWN_TEXT => {
                                allow_unknown_text = true;
                            }
                            NestedMeta::Meta(Meta::Path(m)) if m == ALLOW_UNKNOWN => {
                                allow_unknown_children = true;
                                allow_unknown_attributes = true;
                                allow_unknown_text = true;
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
            from: from.unwrap_or(From::Default),
            allow_unknown_children,
            allow_unknown_attributes,
            allow_unknown_text,
        }
    }
}

pub struct Field {
    pub source: FieldSource,
    pub default: Default,
    pub rename: Option<String>,
    pub from: From,
}

#[derive(Copy, Clone, PartialEq)]
pub enum FieldSource {
    Attribute,
    Child,
    Value,
    Flatten,
    Remains,
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
            if attr.path != ASYNC_XML {
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
                            NestedMeta::Meta(Meta::Path(m)) if m == FLATTEN => {
                                if source.replace(FieldSource::Flatten).is_some() {
                                    ctx.error_spanned_by(m, "source already specified");
                                }
                            }
                            NestedMeta::Meta(Meta::Path(m)) if m == REMAINS => {
                                if source.replace(FieldSource::Remains).is_some() {
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
                            NestedMeta::Meta(Meta::Path(m)) if m == FROM_STR => {
                                if from.replace(From::FromStr).is_some() {
                                    ctx.error_spanned_by(m, "from already specified");
                                }
                            }
                            NestedMeta::Meta(Meta::NameValue(m)) if m.path == FROM => {
                                let _type = get_lit_str_as_type(ctx, &m.lit);
                                if let Ok(_type) = _type {
                                    if from.replace(From::From(_type)).is_some() {
                                        ctx.error_spanned_by(m, "from already specified");
                                    }
                                }
                            }
                            NestedMeta::Meta(Meta::NameValue(m)) if m.path == TRY_FROM => {
                                let _type = get_lit_str_as_type(ctx, &m.lit);
                                if let Ok(_type) = _type {
                                    if from.replace(From::TryFrom(_type)).is_some() {
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
            from: from.unwrap_or(From::Default),
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
