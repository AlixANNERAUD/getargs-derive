use super::RawContainer;
use crate::model::attr::set_once;
use crate::model::field::parse_bool;
use syn::meta::ParseNestedMeta;

pub(super) fn allow_negative_numbers(
    meta: &ParseNestedMeta,
    raw: &mut RawContainer,
) -> syn::Result<()> {
    let (value, _) = parse_bool(meta, "allow_negative_numbers")?;
    set_once(&mut raw.allow_negative_numbers, value, meta)
}
