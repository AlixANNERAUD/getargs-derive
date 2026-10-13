use super::RawContainer;
use crate::model::attr::{parse_string, set_once};
use syn::meta::ParseNestedMeta;

pub(super) fn name(meta: &ParseNestedMeta, raw: &mut RawContainer) -> syn::Result<()> {
    let literal = parse_string(meta, "name", "name = \"tool\"")?;
    let value = literal.value();
    if value.is_empty() {
        return Err(syn::Error::new(literal.span(), "`name` cannot be empty"));
    }
    set_once(&mut raw.name, value, meta)
}

pub(super) fn about(meta: &ParseNestedMeta, raw: &mut RawContainer) -> syn::Result<()> {
    let literal = parse_string(meta, "about", "about = \"Does things\"")?;
    set_once(&mut raw.about, literal.value(), meta)
}
