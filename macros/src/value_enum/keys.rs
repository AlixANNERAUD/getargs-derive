use super::model::RawVariant;
use crate::model::attr::{parse_string, set_once};
use syn::{LitStr, meta::ParseNestedMeta};

fn non_empty(literal: LitStr, key: &str) -> syn::Result<LitStr> {
    if literal.value().is_empty() {
        return Err(syn::Error::new(
            literal.span(),
            format!("`{key}` cannot be empty"),
        ));
    }
    Ok(literal)
}

pub(super) fn alias(meta: &ParseNestedMeta, raw: &mut RawVariant) -> syn::Result<()> {
    let literal = non_empty(parse_string(meta, "alias", "alias = \"quick\"")?, "alias")?;
    raw.aliases.push(literal);
    Ok(())
}

pub(super) fn name(meta: &ParseNestedMeta, raw: &mut RawVariant) -> syn::Result<()> {
    let literal = non_empty(parse_string(meta, "name", "name = \"fast\"")?, "name")?;
    set_once(&mut raw.name, literal, meta)
}
