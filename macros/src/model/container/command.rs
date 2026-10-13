use super::RawContainer;
use crate::model::attr::{expected, parse_list, parse_string, reject_value, set_once};
use crate::model::field::Alias;
use proc_macro2::Span;
use syn::{Lit, meta::ParseNestedMeta};

pub(crate) fn validate_command_name(key: &str, name: &str, span: Span) -> syn::Result<()> {
    if name.is_empty() {
        return Err(syn::Error::new(span, format!("`{key}` cannot be empty")));
    }
    if name.starts_with('-') {
        return Err(syn::Error::new(
            span,
            format!("`{key}` cannot start with `-`, because it would be read as an option"),
        ));
    }
    if name.contains(char::is_whitespace) {
        return Err(syn::Error::new(
            span,
            format!("`{key}` cannot contain whitespace"),
        ));
    }
    Ok(())
}

fn to_alias(literal: Lit, key: &str, example: &str, visible: bool) -> syn::Result<Alias<String>> {
    match literal {
        Lit::Str(literal) => {
            let name = literal.value();
            validate_command_name(key, &name, literal.span())?;
            Ok(Alias {
                name,
                visible,
                span: literal.span(),
            })
        }
        other => Err(syn::Error::new(
            other.span(),
            expected(key, "string", example),
        )),
    }
}

fn single(
    meta: &ParseNestedMeta,
    raw: &mut RawContainer,
    key: &str,
    visible: bool,
) -> syn::Result<()> {
    let example = format!("{key} = \"name\"");
    let literal = parse_string(meta, key, &example)?;
    let alias = to_alias(Lit::Str(literal), key, &example, visible)?;
    raw.aliases.push(alias);
    Ok(())
}

fn multiple(
    meta: &ParseNestedMeta,
    raw: &mut RawContainer,
    key: &str,
    visible: bool,
) -> syn::Result<()> {
    let example = format!("{key} = [\"first\", \"second\"]");
    for literal in parse_list(meta, key, "string", &example)? {
        let alias = to_alias(literal, key, &example, visible)?;
        raw.aliases.push(alias);
    }
    Ok(())
}

pub(super) fn alias(meta: &ParseNestedMeta, raw: &mut RawContainer) -> syn::Result<()> {
    single(meta, raw, "alias", false)
}

pub(super) fn aliases(meta: &ParseNestedMeta, raw: &mut RawContainer) -> syn::Result<()> {
    multiple(meta, raw, "aliases", false)
}

pub(super) fn visible_alias(meta: &ParseNestedMeta, raw: &mut RawContainer) -> syn::Result<()> {
    single(meta, raw, "visible_alias", true)
}

pub(super) fn visible_aliases(meta: &ParseNestedMeta, raw: &mut RawContainer) -> syn::Result<()> {
    multiple(meta, raw, "visible_aliases", true)
}

pub(super) fn hide(meta: &ParseNestedMeta, raw: &mut RawContainer) -> syn::Result<()> {
    reject_value(meta, "hide")?;
    set_once(&mut raw.hide, (), meta)
}
