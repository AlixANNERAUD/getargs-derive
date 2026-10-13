use super::RawField;
use crate::model::attr::{parse_string, reject_value, require_value, set_once};
use syn::{Lit, meta::ParseNestedMeta};

pub(super) fn hide(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    reject_value(meta, "hide")?;
    set_once(&mut raw.hide, (), meta)
}

pub(super) fn help_heading(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let literal = parse_string(meta, "help_heading", "help_heading = \"Network\"")?;
    let value = literal.value();
    if value.trim().is_empty() {
        return Err(syn::Error::new(
            literal.span(),
            "`help_heading` cannot be empty",
        ));
    }
    set_once(&mut raw.help_heading, value, meta)
}

pub(super) fn display_order(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    const MESSAGE: &str =
        "`display_order` expects a non-negative integer literal, e.g. `display_order = 1`";
    let value = match require_value(meta, "display_order", "display_order = 1")?.parse::<Lit>()? {
        Lit::Int(literal) => match literal.base10_parse::<usize>() {
            Ok(value) => (value, literal.span()),
            Err(_) => return Err(syn::Error::new(literal.span(), MESSAGE)),
        },
        other => return Err(syn::Error::new(other.span(), MESSAGE)),
    };
    set_once(&mut raw.display_order, value, meta)
}

pub(super) fn long_help(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let literal = parse_string(
        meta,
        "long_help",
        "long_help = \"Print more output.\\n\\nRepeat for even more.\"",
    )?;
    set_once(&mut raw.long_help, literal.value(), meta)
}
