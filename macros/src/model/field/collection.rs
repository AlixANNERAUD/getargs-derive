use super::{DefaultSource, DefaultValue, FieldKind, FieldModel, RawField};
use crate::model::attr::{require_value, set_once};
use crate::model::errors::Errors;
use crate::model::types::{collection_element, option_inner_type};
use proc_macro2::Span;
use syn::{
    Expr, ExprLit, ExprRange, Lit, LitStr, RangeLimits, Type, meta::ParseNestedMeta,
    spanned::Spanned,
};

const SUPPORTED: &str = "`getargs_derive::Values<T, N>`, `heapless::Vec<T, N>` (feature `heapless`) or `Vec<T>` (feature `alloc`)";

#[derive(Default)]
pub(super) struct RawCollection {
    default_values: Option<(Vec<LitStr>, Span)>,
    num_args: Option<((usize, Option<usize>), Span)>,
    value_delimiter: Option<(char, Span)>,
}

pub(crate) struct CollectionModel {
    pub(crate) delimiter: Option<char>,
    pub(crate) min: usize,
    pub(crate) max: Option<usize>,
    pub(crate) default_values: Vec<LitStr>,
}

impl FieldModel {
    pub(crate) fn ellipsis(&self) -> &'static str {
        if self.collection.is_some() { "..." } else { "" }
    }
}

pub(super) fn default_values(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let example = "default_values = [\"a\", \"b\"]";
    let expr: Expr = require_value(meta, "default_values", example)?.parse()?;
    let Expr::Array(array) = expr else {
        return Err(syn::Error::new_spanned(
            expr,
            format!("`default_values` expects an array of string literals, e.g. `{example}`"),
        ));
    };
    if array.elems.is_empty() {
        return Err(syn::Error::new_spanned(
            array,
            "`default_values` cannot be empty; a collection is already empty when absent",
        ));
    }
    let mut values = Vec::new();
    for element in &array.elems {
        match element {
            Expr::Lit(ExprLit {
                lit: Lit::Str(literal),
                ..
            }) => values.push(literal.clone()),
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    format!("`default_values` expects string literals, e.g. `{example}`"),
                ));
            }
        }
    }
    set_once(
        &mut raw.collection.default_values,
        (values, meta.path.span()),
        meta,
    )
}

fn bound(expr: &Expr) -> syn::Result<usize> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(literal),
            ..
        }) => literal.base10_parse(),
        other => Err(syn::Error::new_spanned(
            other,
            "`num_args` bounds must be integer literals, e.g. `num_args = 1..`",
        )),
    }
}

fn range(range: &ExprRange) -> syn::Result<(usize, Option<usize>)> {
    let min = range.start.as_deref().map(bound).transpose()?.unwrap_or(0);
    let max = match (&range.end, &range.limits) {
        (None, _) => None,
        (Some(end), RangeLimits::Closed(_)) => Some(bound(end)?),
        (Some(end), RangeLimits::HalfOpen(_)) => match bound(end)?.checked_sub(1) {
            Some(max) => Some(max),
            None => {
                return Err(syn::Error::new_spanned(range, "`num_args` range is empty"));
            }
        },
    };
    match max {
        Some(max) if max < min => Err(syn::Error::new_spanned(range, "`num_args` range is empty")),
        Some(0) => Err(syn::Error::new_spanned(
            range,
            "`num_args` must accept at least one value",
        )),
        _ => Ok((min, max)),
    }
}

pub(super) fn num_args(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let expr: Expr = require_value(meta, "num_args", "num_args = 1..")?.parse()?;
    let value = match &expr {
        Expr::Range(expr) => range(expr)?,
        Expr::Lit(_) => match bound(&expr)? {
            0 => {
                return Err(syn::Error::new_spanned(
                    expr,
                    "`num_args` must accept at least one value",
                ));
            }
            count => (count, Some(count)),
        },
        other => {
            return Err(syn::Error::new_spanned(
                other,
                "`num_args` expects an integer or a range of integer literals, e.g. `num_args = 1..` or `num_args = 2..=4`",
            ));
        }
    };
    set_once(
        &mut raw.collection.num_args,
        (value, meta.path.span()),
        meta,
    )
}

pub(super) fn value_delimiter(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let delimiter =
        match require_value(meta, "value_delimiter", "value_delimiter = ','")?.parse::<Lit>()? {
            Lit::Char(literal) => (literal.value(), literal.span()),
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "`value_delimiter` expects a character literal, e.g. `value_delimiter = ','`",
                ));
            }
        };
    set_once(&mut raw.collection.value_delimiter, delimiter, meta)
}

pub(super) fn element<'t>(ty: &'t Type, errors: &mut Errors) -> Option<&'t Type> {
    if let Type::Array(array) = ty {
        errors.push(syn::Error::new_spanned(
            array,
            format!(
                "fixed-size array fields are not supported; collect repeated values into {SUPPORTED}"
            ),
        ));
        return None;
    }
    if let Some(inner) = option_inner_type(ty)
        && (collection_element(inner).is_some() || matches!(inner, Type::Array(_)))
    {
        errors.push(syn::Error::new_spanned(
            ty,
            "`Option<T>` cannot wrap a collection; a collection field is already empty when absent, so remove the `Option`",
        ));
        return None;
    }
    let element = collection_element(ty)?;
    if collection_element(element).is_some()
        || option_inner_type(element).is_some()
        || matches!(element, Type::Array(_))
    {
        errors.push(syn::Error::new_spanned(
            element,
            "collection elements cannot be collections, arrays or `Option<T>`",
        ));
        return None;
    }
    Some(element)
}

fn split(value: &str, delimiter: Option<char>) -> Vec<&str> {
    match delimiter {
        Some(delimiter) => value.split(delimiter).collect(),
        None => vec![value],
    }
}

fn check_default(
    literal: &LitStr,
    delimiter: Option<char>,
    possible_values: &[String],
    errors: &mut Errors,
) {
    if possible_values.is_empty() {
        return;
    }
    let value = literal.value();
    for piece in split(&value, delimiter) {
        if !possible_values.iter().any(|possible| possible == piece) {
            errors.push(syn::Error::new(
                literal.span(),
                format!(
                    "default value \"{piece}\" is not one of the possible values: {}",
                    possible_values.join(", ")
                ),
            ));
        }
    }
}

pub(super) fn build(
    raw: RawCollection,
    collection: bool,
    kind: &FieldKind,
    default: Option<&(DefaultSource, DefaultValue)>,
    possible_values: &[String],
    errors: &mut Errors,
) -> Option<CollectionModel> {
    if !collection {
        let keys = [
            ("default_values", raw.default_values.map(|(_, span)| span)),
            ("num_args", raw.num_args.map(|(_, span)| span)),
            ("value_delimiter", raw.value_delimiter.map(|(_, span)| span)),
        ];
        for (key, span) in keys {
            if let Some(span) = span {
                errors.push(syn::Error::new(
                    span,
                    format!(
                        "`{key}` requires a field collecting repeated values, of type {SUPPORTED}"
                    ),
                ));
            }
        }
        return None;
    }

    if let Some((_, span)) = raw.num_args
        && !matches!(kind, FieldKind::Positional)
    {
        errors.push(syn::Error::new(
            span,
            "`num_args` is only supported on positional arguments; a repeated option takes one value per occurrence",
        ));
    }

    let delimiter = raw.value_delimiter.map(|(delimiter, _)| delimiter);
    match (default, &raw.default_values) {
        (Some((DefaultSource::DefaultValueT, value)), _) => errors.push(syn::Error::new(
            value.span(),
            "`default_value_t` is not supported on collections; use `default_value` or `default_values`",
        )),
        (Some((source, _)), Some((_, span))) => errors.push(syn::Error::new(
            *span,
            format!(
                "conflicting default values: {} and `default_values` cannot be used together; use only one of them",
                source.describe()
            ),
        )),
        (Some((_, DefaultValue::Literal(literal))), None) => {
            check_default(literal, delimiter, possible_values, errors)
        }
        _ => {}
    }
    let default_values = raw
        .default_values
        .map(|(values, _)| values)
        .unwrap_or_default();
    for literal in &default_values {
        check_default(literal, None, possible_values, errors);
    }

    let (min, max) = raw.num_args.map_or((0, None), |(range, _)| range);
    Some(CollectionModel {
        delimiter,
        min,
        max,
        default_values,
    })
}

pub(in crate::model) fn validate_positionals(fields: &[FieldModel], errors: &mut Errors) {
    let mut collecting: Option<&FieldModel> = None;
    for field in fields {
        if !matches!(field.kind, FieldKind::Positional) || field.parsing.last {
            continue;
        }
        match collecting {
            Some(collecting) => errors.push(syn::Error::new(
                field.ident.span(),
                format!(
                    "positional argument `{}` cannot follow `{}`, which collects all remaining positional arguments; move it before `{}` or make it `last = true`",
                    field.name, collecting.name, collecting.name
                ),
            )),
            None if field.collection.is_some() => collecting = Some(field),
            None => {}
        }
    }
}
