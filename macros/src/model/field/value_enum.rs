use super::RawField;
use crate::model::attr::{reject_value, set_once};
use syn::{meta::ParseNestedMeta, spanned::Spanned};

pub(super) fn value_enum(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    reject_value(meta, "value_enum")?;
    set_once(&mut raw.value_enum, meta.path.span(), meta)
}
