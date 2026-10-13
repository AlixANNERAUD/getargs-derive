use super::{DefaultValue, FieldKind, RawField};
use crate::model::attr::{require_value, set_once};
use crate::model::errors::Errors;
use crate::model::types::is_count_type;
use proc_macro2::Span;
use std::collections::HashSet;
use syn::{Expr, ExprLit, ExprRange, Lit, LitStr, Type, meta::ParseNestedMeta, spanned::Spanned};

const VALUE_PARSER_EXAMPLE: &str = "`value_parser = parse_port` or `value_parser = 1..=64`";

pub(crate) enum ValueParser {
    Function(syn::Path),
    Range(ExprRange),
}

pub(crate) struct PossibleValues {
    pub(crate) values: Vec<String>,
    span: Span,
}

pub(super) fn value_parser(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let expr: Expr = require_value(meta, "value_parser", "value_parser = 1..=64")?.parse()?;
    let parser = match expr {
        Expr::Path(path) if path.qself.is_none() && path.attrs.is_empty() => {
            ValueParser::Function(path.path)
        }
        Expr::Range(range) if range.start.is_none() && range.end.is_none() => {
            return Err(syn::Error::new_spanned(
                range,
                "`value_parser` range needs at least one bound, e.g. `value_parser = 1..=64`",
            ));
        }
        Expr::Range(range) => ValueParser::Range(range),
        other => {
            return Err(syn::Error::new_spanned(
                other,
                format!(
                    "`value_parser` expects a function path or a range, e.g. {VALUE_PARSER_EXAMPLE}"
                ),
            ));
        }
    };
    set_once(&mut raw.value_parser, parser, meta)
}

fn parse_possible_values(meta: &ParseNestedMeta) -> syn::Result<PossibleValues> {
    let example = "possible_values = [\"fast\", \"slow\"]";
    let expr: Expr = require_value(meta, "possible_values", example)?.parse()?;
    let Expr::Array(array) = expr else {
        return Err(syn::Error::new_spanned(
            expr,
            format!("`possible_values` expects an array of string literals, e.g. `{example}`"),
        ));
    };
    if array.elems.is_empty() {
        return Err(syn::Error::new_spanned(
            array,
            "`possible_values` cannot be empty",
        ));
    }
    let mut values = Vec::new();
    let mut seen = HashSet::new();
    let mut errors = Errors::default();
    for element in &array.elems {
        let Expr::Lit(ExprLit {
            lit: Lit::Str(literal),
            ..
        }) = element
        else {
            errors.push(syn::Error::new_spanned(
                element,
                format!("`possible_values` expects string literals, e.g. `{example}`"),
            ));
            continue;
        };
        let value = literal.value();
        if value.is_empty() {
            errors.push(syn::Error::new(
                literal.span(),
                "a possible value cannot be empty",
            ));
        } else if !seen.insert(value.clone()) {
            errors.push(syn::Error::new(
                literal.span(),
                format!("duplicate possible value \"{value}\""),
            ));
        }
        values.push(value);
    }
    errors.finish(PossibleValues {
        values,
        span: array.span(),
    })
}

pub(super) fn possible_values(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let values = parse_possible_values(meta)?;
    set_once(&mut raw.possible_values, values, meta)
}

fn key_error(
    value_parser: Option<&ValueParser>,
    possible_values: Option<&PossibleValues>,
    message: impl Fn(&str) -> String,
) -> Option<syn::Error> {
    match (value_parser, possible_values) {
        (Some(ValueParser::Function(path)), _) => {
            Some(syn::Error::new_spanned(path, message("value_parser")))
        }
        (Some(ValueParser::Range(range)), _) => {
            Some(syn::Error::new_spanned(range, message("value_parser")))
        }
        (None, Some(values)) => Some(syn::Error::new(values.span, message("possible_values"))),
        (None, None) => None,
    }
}

pub(super) fn check(
    value_parser: Option<&ValueParser>,
    possible_values: Option<&PossibleValues>,
    kind: &FieldKind,
    value_ty: &Type,
    default: Option<&DefaultValue>,
    value_enum: bool,
    errors: &mut Errors,
) {
    if let FieldKind::Flag(_) = kind
        && let Some(error) = key_error(value_parser, possible_values, |key| {
            format!("`{key}` cannot be used on flags because they do not take a value")
        })
    {
        errors.push(error);
        return;
    }

    if value_enum
        && let Some(error) = key_error(value_parser, possible_values, |key| {
            format!(
                "`{key}` cannot be combined with `value_enum`, which already defines the accepted values"
            )
        })
    {
        errors.push(error);
        return;
    }

    if let Some(ValueParser::Range(range)) = value_parser
        && !is_count_type(value_ty)
    {
        errors.push(syn::Error::new_spanned(
            range,
            "a `value_parser` range requires an integer field type, e.g. `u8`; use a function for other types",
        ));
    }

    if let (Some(values), Some(DefaultValue::Literal(literal))) = (possible_values, default) {
        check_default(values, literal, errors);
    }
}

fn check_default(values: &PossibleValues, literal: &LitStr, errors: &mut Errors) {
    let default = literal.value();
    if !values.values.contains(&default) {
        errors.push(syn::Error::new(
            literal.span(),
            format!(
                "`default_value` \"{default}\" is not one of the possible values: {}",
                values.values.join(", ")
            ),
        ));
    }
}
