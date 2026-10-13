use super::{RawField, validate_long, validate_short};
use crate::model::attr::{expected, parse_list, parse_string, require_value};
use proc_macro2::Span;
use syn::{Lit, meta::ParseNestedMeta};

pub(crate) struct Alias<T> {
    pub(crate) name: T,
    pub(crate) visible: bool,
    pub(crate) span: Span,
}

fn to_long_alias(
    literal: Lit,
    key: &str,
    example: &str,
    visible: bool,
) -> syn::Result<Alias<String>> {
    match literal {
        Lit::Str(literal) => {
            let name = literal.value();
            validate_long(key, &name, literal.span())?;
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

fn to_short_alias(
    literal: Lit,
    key: &str,
    example: &str,
    visible: bool,
) -> syn::Result<Alias<char>> {
    match literal {
        Lit::Char(literal) => {
            let name = literal.value();
            validate_short(key, name, literal.span())?;
            Ok(Alias {
                name,
                visible,
                span: literal.span(),
            })
        }
        other => Err(syn::Error::new(
            other.span(),
            expected(key, "character", example),
        )),
    }
}

fn single_long(
    meta: &ParseNestedMeta,
    raw: &mut RawField,
    key: &str,
    visible: bool,
) -> syn::Result<()> {
    let example = format!("{key} = \"name\"");
    let literal = parse_string(meta, key, &example)?;
    let alias = to_long_alias(Lit::Str(literal), key, &example, visible)?;
    raw.long_aliases.push(alias);
    Ok(())
}

fn multiple_long(
    meta: &ParseNestedMeta,
    raw: &mut RawField,
    key: &str,
    visible: bool,
) -> syn::Result<()> {
    let example = format!("{key} = [\"first\", \"second\"]");
    for literal in parse_list(meta, key, "string", &example)? {
        let alias = to_long_alias(literal, key, &example, visible)?;
        raw.long_aliases.push(alias);
    }
    Ok(())
}

fn single_short(
    meta: &ParseNestedMeta,
    raw: &mut RawField,
    key: &str,
    visible: bool,
) -> syn::Result<()> {
    let example = format!("{key} = 'x'");
    let literal = require_value(meta, key, &example)?.parse::<Lit>()?;
    let alias = to_short_alias(literal, key, &example, visible)?;
    raw.short_aliases.push(alias);
    Ok(())
}

fn multiple_short(
    meta: &ParseNestedMeta,
    raw: &mut RawField,
    key: &str,
    visible: bool,
) -> syn::Result<()> {
    let example = format!("{key} = ['x', 'y']");
    for literal in parse_list(meta, key, "character", &example)? {
        let alias = to_short_alias(literal, key, &example, visible)?;
        raw.short_aliases.push(alias);
    }
    Ok(())
}

pub(super) fn alias(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    single_long(meta, raw, "alias", false)
}

pub(super) fn aliases(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    multiple_long(meta, raw, "aliases", false)
}

pub(super) fn visible_alias(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    single_long(meta, raw, "visible_alias", true)
}

pub(super) fn visible_aliases(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    multiple_long(meta, raw, "visible_aliases", true)
}

pub(super) fn short_alias(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    single_short(meta, raw, "short_alias", false)
}

pub(super) fn short_aliases(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    multiple_short(meta, raw, "short_aliases", false)
}

pub(super) fn visible_short_alias(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    single_short(meta, raw, "visible_short_alias", true)
}

pub(super) fn visible_short_aliases(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    multiple_short(meta, raw, "visible_short_aliases", true)
}
