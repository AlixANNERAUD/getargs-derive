use quote::ToTokens;
use syn::{Expr, Ident, Lit, LitStr, Token, meta::ParseNestedMeta, spanned::Spanned};

pub(crate) type Parser<T> = fn(&ParseNestedMeta, &mut T) -> syn::Result<()>;

pub(crate) struct Key<T> {
    name: &'static str,
    parse: Parser<T>,
}

impl<T> Key<T> {
    pub(crate) const fn new(name: &'static str, parse: Parser<T>) -> Self {
        Self { name, parse }
    }
}

fn supported_keys<T>(keys: &[Key<T>]) -> String {
    keys.iter()
        .map(|key| format!("`{}`", key.name))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn dispatch<T>(
    meta: ParseNestedMeta,
    attribute: &str,
    keys: &[Key<T>],
    target: &mut T,
) -> syn::Result<()> {
    let key = meta.path.get_ident().map(Ident::to_string);
    match key
        .as_deref()
        .and_then(|key| keys.iter().find(|entry| entry.name == key))
    {
        Some(entry) => (entry.parse)(&meta, target),
        None => {
            let path =
                key.unwrap_or_else(|| meta.path.to_token_stream().to_string().replace(' ', ""));
            Err(meta.error(format!(
                "unknown `{attribute}` attribute `{path}`; supported attributes are: {}",
                supported_keys(keys)
            )))
        }
    }
}

pub(crate) fn set_once<T>(
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

pub(crate) fn reject_value(meta: &ParseNestedMeta, key: &str) -> syn::Result<()> {
    if meta.input.peek(Token![=]) || meta.input.peek(syn::token::Paren) {
        return Err(meta.error(format!("`{key}` does not take a value")));
    }
    Ok(())
}

pub(crate) fn require_value<'a>(
    meta: &'a ParseNestedMeta,
    key: &str,
    example: &str,
) -> syn::Result<syn::parse::ParseStream<'a>> {
    if !meta.input.peek(Token![=]) {
        return Err(meta.error(format!("`{key}` requires a value, e.g. `{example}`")));
    }
    meta.value()
}

pub(crate) fn expected(key: &str, kind: &str, example: &str) -> String {
    match key.ends_with("aliases") {
        true => format!("`{key}` expects an array of {kind} literals, e.g. `{example}`"),
        false => format!("`{key}` expects a {kind} literal, e.g. `{example}`"),
    }
}

pub(crate) fn parse_list(
    meta: &ParseNestedMeta,
    key: &str,
    kind: &str,
    example: &str,
) -> syn::Result<Vec<Lit>> {
    let array = match require_value(meta, key, example)?.parse::<Expr>()? {
        Expr::Array(array) => array,
        other => {
            return Err(syn::Error::new(other.span(), expected(key, kind, example)));
        }
    };
    array
        .elems
        .into_iter()
        .map(|element| match element {
            Expr::Lit(literal) => Ok(literal.lit),
            other => Err(syn::Error::new(other.span(), expected(key, kind, example))),
        })
        .collect()
}

pub(crate) fn parse_string(
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
