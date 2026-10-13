use super::RawField;
use crate::model::attr::{parse_string, set_once};
use syn::{LitStr, meta::ParseNestedMeta};

#[derive(Default)]
pub(crate) struct Relations {
    pub(crate) conflicts_with: Vec<LitStr>,
    pub(crate) requires: Vec<LitStr>,
    pub(crate) required_unless_present: Vec<LitStr>,
    pub(crate) group: Option<LitStr>,
}

fn parse_id(meta: &ParseNestedMeta, key: &str, example: &str) -> syn::Result<LitStr> {
    let literal = parse_string(meta, key, example)?;
    if literal.value().is_empty() {
        return Err(syn::Error::new(
            literal.span(),
            format!("`{key}` cannot be empty"),
        ));
    }
    Ok(literal)
}

pub(super) fn conflicts_with(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let id = parse_id(meta, "conflicts_with", "conflicts_with = \"other\"")?;
    raw.relations.conflicts_with.push(id);
    Ok(())
}

pub(super) fn requires(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let id = parse_id(meta, "requires", "requires = \"other\"")?;
    raw.relations.requires.push(id);
    Ok(())
}

pub(super) fn required_unless_present(
    meta: &ParseNestedMeta,
    raw: &mut RawField,
) -> syn::Result<()> {
    let id = parse_id(
        meta,
        "required_unless_present",
        "required_unless_present = \"other\"",
    )?;
    raw.relations.required_unless_present.push(id);
    Ok(())
}

pub(super) fn group(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let id = parse_id(meta, "group", "group = \"mode\"")?;
    set_once(&mut raw.relations.group, id, meta)
}
