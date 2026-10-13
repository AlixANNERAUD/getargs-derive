use super::RawContainer;
use crate::model::attr::{reject_value, set_once};
use syn::meta::ParseNestedMeta;

pub(super) fn disable_help_flag(meta: &ParseNestedMeta, raw: &mut RawContainer) -> syn::Result<()> {
    reject_value(meta, "disable_help_flag")?;
    set_once(&mut raw.disable_help_flag, (), meta)
}
