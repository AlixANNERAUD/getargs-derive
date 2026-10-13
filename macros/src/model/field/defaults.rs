use super::{DefaultSource, DefaultValue, RawField};
use crate::model::attr::require_value;
use proc_macro2::Span;
use syn::{Expr, Lit, meta::ParseNestedMeta, spanned::Spanned};

fn set_default(
    slot: &mut Option<(DefaultSource, DefaultValue)>,
    source: DefaultSource,
    value: DefaultValue,
    span: Span,
) -> syn::Result<()> {
    if let Some((existing, _)) = slot {
        let message = if *existing == source {
            format!("duplicate {} attribute", source.describe())
        } else {
            format!(
                "conflicting default values: {} and {} cannot be used together; use only one of them",
                existing.describe(),
                source.describe()
            )
        };
        return Err(syn::Error::new(span, message));
    }
    *slot = Some((source, value));
    Ok(())
}

pub(super) fn default_value(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let literal = match require_value(meta, "default_value", "default_value = \"4\"")?
        .parse::<Lit>()?
    {
        Lit::Str(literal) => literal,
        other => {
            return Err(syn::Error::new(
                other.span(),
                "`default_value` expects a string literal; use `default_value_t = expr` for a typed value",
            ));
        }
    };
    set_default(
        &mut raw.default,
        DefaultSource::DefaultValue,
        DefaultValue::Literal(literal),
        meta.path.span(),
    )
}

pub(super) fn default_value_t(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let expr: Expr = require_value(meta, "default_value_t", "default_value_t = 4")?.parse()?;
    set_default(
        &mut raw.default,
        DefaultSource::DefaultValueT,
        DefaultValue::Expr(expr),
        meta.path.span(),
    )
}
