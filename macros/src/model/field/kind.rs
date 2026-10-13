use super::RawField;
use crate::model::attr::{reject_value, set_once};
use syn::{meta::ParseNestedMeta, spanned::Spanned};

pub(super) fn flag(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    reject_value(meta, "flag")?;
    set_once(&mut raw.flag, meta.path.span(), meta)
}

pub(super) fn positional(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    reject_value(meta, "positional")?;
    set_once(&mut raw.positional, meta.path.span(), meta)
}
