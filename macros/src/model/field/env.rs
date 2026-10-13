use super::RawField;
use crate::model::attr::set_once;
use proc_macro2::Span;
use syn::{Lit, Token, meta::ParseNestedMeta, spanned::Spanned};

pub(super) fn env(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let value = if meta.input.peek(Token![=]) {
        match meta.value()?.parse::<Lit>()? {
            Lit::Str(literal) => (Some(literal.value()), literal.span()),
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "`env` expects a string literal, e.g. `env = \"PORT\"`",
                ));
            }
        }
    } else {
        (None, meta.path.span())
    };
    set_once(&mut raw.env, value, meta)
}

pub(super) fn validate_env(name: &str, span: Span) -> syn::Result<()> {
    if name.is_empty() {
        return Err(syn::Error::new(span, "`env` cannot be empty"));
    }
    if name.contains(['=', '\0']) {
        return Err(syn::Error::new(
            span,
            "`env` cannot contain `=` or NUL characters",
        ));
    }
    Ok(())
}
