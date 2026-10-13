use super::RawContainer;
use crate::model::attr::{parse_string, require_value, set_once};
use syn::{Lit, meta::ParseNestedMeta};

pub(super) fn long_about(meta: &ParseNestedMeta, raw: &mut RawContainer) -> syn::Result<()> {
    let literal = parse_string(
        meta,
        "long_about",
        "long_about = \"Does things.\\n\\nIn more detail.\"",
    )?;
    set_once(&mut raw.long_about, literal.value(), meta)
}

pub(super) fn before_help(meta: &ParseNestedMeta, raw: &mut RawContainer) -> syn::Result<()> {
    let literal = parse_string(meta, "before_help", "before_help = \"Read this first\"")?;
    set_once(&mut raw.before_help, literal.value(), meta)
}

pub(super) fn after_help(meta: &ParseNestedMeta, raw: &mut RawContainer) -> syn::Result<()> {
    let literal = parse_string(meta, "after_help", "after_help = \"See the manual\"")?;
    set_once(&mut raw.after_help, literal.value(), meta)
}

pub(super) fn max_term_width(meta: &ParseNestedMeta, raw: &mut RawContainer) -> syn::Result<()> {
    const MESSAGE: &str =
        "`max_term_width` expects a positive integer literal, e.g. `max_term_width = 80`";
    let width =
        match require_value(meta, "max_term_width", "max_term_width = 80")?.parse::<Lit>()? {
            Lit::Int(literal) => match literal.base10_parse::<usize>() {
                Ok(width) if width > 0 => width,
                _ => return Err(syn::Error::new(literal.span(), MESSAGE)),
            },
            other => return Err(syn::Error::new(other.span(), MESSAGE)),
        };
    set_once(&mut raw.max_term_width, width, meta)
}
