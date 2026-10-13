use super::RawField;
use crate::model::attr::set_once;
use syn::{Lit, Token, meta::ParseNestedMeta, spanned::Spanned};

pub(super) fn short(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let value = if meta.input.peek(Token![=]) {
        match meta.value()?.parse::<Lit>()? {
            Lit::Char(literal) => (Some(literal.value()), literal.span()),
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "`short` expects a character literal, e.g. `short = 'o'`",
                ));
            }
        }
    } else {
        (None, meta.path.span())
    };
    set_once(&mut raw.short, value, meta)
}

pub(super) fn long(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let value = if meta.input.peek(Token![=]) {
        match meta.value()?.parse::<Lit>()? {
            Lit::Str(literal) => (Some(literal.value()), literal.span()),
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "`long` expects a string literal, e.g. `long = \"output\"`",
                ));
            }
        }
    } else {
        (None, meta.path.span())
    };
    set_once(&mut raw.long, value, meta)
}
