use quote::ToTokens;
use syn::{Lit, LitStr, Token, meta::ParseNestedMeta};

pub(super) fn set_once<T>(
    slot: &mut Option<T>,
    value: T,
    meta: &ParseNestedMeta,
) -> syn::Result<()> {
    if slot.is_some() {
        let key = meta.path.to_token_stream().to_string();
        return Err(meta.error(format!("duplicate `{key}` attribute")));
    }
    *slot = Some(value);
    Ok(())
}

pub(super) fn reject_value(meta: &ParseNestedMeta, key: &str) -> syn::Result<()> {
    if meta.input.peek(Token![=]) || meta.input.peek(syn::token::Paren) {
        return Err(meta.error(format!("`{key}` does not take a value")));
    }
    Ok(())
}

pub(super) fn require_value<'a>(
    meta: &'a ParseNestedMeta,
    key: &str,
    example: &str,
) -> syn::Result<syn::parse::ParseStream<'a>> {
    if !meta.input.peek(Token![=]) {
        return Err(meta.error(format!("`{key}` requires a value, e.g. `{example}`")));
    }
    meta.value()
}

pub(super) fn parse_string(
    meta: &ParseNestedMeta,
    key: &str,
    example: &str,
) -> syn::Result<LitStr> {
    match require_value(meta, key, example)?.parse::<Lit>()? {
        Lit::Str(literal) => Ok(literal),
        other => Err(syn::Error::new(
            other.span(),
            format!("`{key}` expects a string literal, e.g. `{example}`"),
        )),
    }
}
