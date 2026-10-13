use super::{FieldKind, FieldModel, RawField};
use crate::model::attr::set_once;
use crate::model::errors::Errors;
use proc_macro2::Span;
use syn::{Lit, Token, meta::ParseNestedMeta, spanned::Spanned};

#[derive(Default)]
pub(super) struct RawParsing {
    allow_hyphen_values: Option<(bool, Span)>,
    allow_negative_numbers: Option<(bool, Span)>,
    last: Option<(bool, Span)>,
    global: Option<(bool, Span)>,
}

#[derive(Default)]
pub(crate) struct Parsing {
    pub(crate) allow_hyphen_values: bool,
    pub(crate) allow_negative_numbers: bool,
    pub(crate) last: bool,
    pub(crate) global: bool,
    last_span: Option<Span>,
}

pub(in crate::model) fn parse_bool(meta: &ParseNestedMeta, key: &str) -> syn::Result<(bool, Span)> {
    if !meta.input.peek(Token![=]) {
        if meta.input.peek(syn::token::Paren) {
            return Err(meta.error(format!(
                "`{key}` expects a boolean, e.g. `{key}` or `{key} = true`"
            )));
        }
        return Ok((true, meta.path.span()));
    }
    match meta.value()?.parse::<Lit>()? {
        Lit::Bool(literal) => Ok((literal.value, literal.span())),
        other => Err(syn::Error::new(
            other.span(),
            format!("`{key}` expects a boolean literal, e.g. `{key} = true`"),
        )),
    }
}

pub(super) fn allow_hyphen_values(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let value = parse_bool(meta, "allow_hyphen_values")?;
    set_once(&mut raw.parsing.allow_hyphen_values, value, meta)
}

pub(super) fn allow_negative_numbers(
    meta: &ParseNestedMeta,
    raw: &mut RawField,
) -> syn::Result<()> {
    let value = parse_bool(meta, "allow_negative_numbers")?;
    set_once(&mut raw.parsing.allow_negative_numbers, value, meta)
}

pub(super) fn last(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let value = parse_bool(meta, "last")?;
    set_once(&mut raw.parsing.last, value, meta)
}

pub(super) fn global(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let value = parse_bool(meta, "global")?;
    set_once(&mut raw.parsing.global, value, meta)
}

fn enabled(setting: Option<(bool, Span)>) -> Option<Span> {
    setting.and_then(|(value, span)| value.then_some(span))
}

pub(super) fn build_parsing(raw: RawParsing, kind: &FieldKind, errors: &mut Errors) -> Parsing {
    let allow_hyphen_values = enabled(raw.allow_hyphen_values);
    let allow_negative_numbers = enabled(raw.allow_negative_numbers);
    let last = enabled(raw.last);
    let global = enabled(raw.global);

    if let (Some(span), FieldKind::Positional) = (global, kind) {
        errors.push(syn::Error::new(
            span,
            "`global` can only be used on options and flags; positional arguments of a parent are not accepted after its subcommand",
        ));
    }

    if let Some(span) = allow_hyphen_values {
        match kind {
            FieldKind::Option => {}
            FieldKind::Flag(_) => errors.push(syn::Error::new(
                span,
                "`allow_hyphen_values` can only be used on options taking a value; flags do not take a value",
            )),
            FieldKind::Positional => errors.push(syn::Error::new(
                span,
                "`allow_hyphen_values` is not supported on positional arguments, because `getargs` splits `-x` into options before the field sees it; use `allow_negative_numbers` for numbers, or `last = true` to accept any value after `--`",
            )),
        }
    }

    if let (Some(span), FieldKind::Flag(_)) = (allow_negative_numbers, kind) {
        errors.push(syn::Error::new(
            span,
            "`allow_negative_numbers` can only be used on options taking a value and positional arguments; flags do not take a value",
        ));
    }

    if let Some(span) = last
        && !matches!(kind, FieldKind::Positional)
    {
        errors.push(syn::Error::new(
            span,
            "`last` can only be used on positional arguments; remove `short` and `long`",
        ));
    }

    Parsing {
        allow_hyphen_values: allow_hyphen_values.is_some(),
        allow_negative_numbers: allow_negative_numbers.is_some(),
        last: last.is_some(),
        global: global.is_some(),
        last_span: last,
    }
}

pub(in crate::model) fn apply_container(fields: &mut [FieldModel], allow_negative_numbers: bool) {
    if !allow_negative_numbers {
        return;
    }
    for field in fields {
        if matches!(field.kind, FieldKind::Option | FieldKind::Positional) {
            field.parsing.allow_negative_numbers = true;
        }
    }
}

pub(in crate::model) fn validate_last(fields: &[FieldModel], errors: &mut Errors) {
    let positionals: Vec<&FieldModel> = fields
        .iter()
        .filter(|field| matches!(field.kind, FieldKind::Positional))
        .collect();
    let lasts: Vec<(usize, &FieldModel, Span)> = positionals
        .iter()
        .enumerate()
        .filter_map(|(index, field)| field.parsing.last_span.map(|span| (index, *field, span)))
        .collect();
    if let [(_, first, _), others @ ..] = lasts.as_slice()
        && !others.is_empty()
    {
        for (_, _, span) in others {
            errors.push(syn::Error::new(
                *span,
                format!(
                    "only one positional argument can use `last`, and `{}` already does",
                    first.name
                ),
            ));
        }
        return;
    }
    for (index, field, span) in lasts {
        if index + 1 != positionals.len() {
            errors.push(syn::Error::new(
                span,
                format!(
                    "`last` positional argument `{}` must be the final positional argument; move it after `{}`",
                    field.name,
                    positionals[positionals.len() - 1].name
                ),
            ));
        }
    }
}
