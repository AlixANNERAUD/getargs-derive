use super::RawField;
use crate::model::attr::{parse_string, set_once};
use syn::meta::ParseNestedMeta;

pub(super) fn help(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let literal = parse_string(meta, "help", "help = \"Print more output\"")?;
    set_once(&mut raw.help, literal.value(), meta)
}

pub(super) fn value_name(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let literal = parse_string(meta, "value_name", "value_name = \"FILE\"")?;
    let value = literal.value();
    if value.is_empty() || value.contains(char::is_whitespace) {
        return Err(syn::Error::new(
            literal.span(),
            "`value_name` cannot be empty or contain whitespace",
        ));
    }
    set_once(&mut raw.value_name, (value, literal.span()), meta)
}
