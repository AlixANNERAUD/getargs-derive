use proc_macro2::Span;
use quote::ToTokens;
use std::collections::HashMap;
use syn::{
    Data, DeriveInput, Expr, Field, Fields, Ident, Lit, LitStr, Meta, Token, Type, ext::IdentExt,
    meta::ParseNestedMeta, spanned::Spanned,
};

const SUPPORTED_ARG_KEYS: &str =
    "`short`, `long`, `default_value`, `default_value_t`, `flag`, `positional`, `default` (legacy)";

pub(crate) enum FieldKind {
    Positional,
    Flag,
    Option,
}

pub(crate) enum DefaultValue {
    Literal(LitStr),
    Expr(Expr),
}

pub(crate) struct FieldModel {
    pub(crate) ident: Ident,
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) kind: FieldKind,
    pub(crate) short: Option<char>,
    pub(crate) long: Option<String>,
    pub(crate) default: Option<DefaultValue>,
    pub(crate) is_str: bool,
    short_span: Span,
    long_span: Span,
}

impl FieldModel {
    pub(crate) fn error_name(&self) -> &str {
        match (&self.kind, &self.long) {
            (FieldKind::Positional, _) | (_, None) => &self.name,
            (_, Some(long)) => long,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DefaultSource {
    DefaultValue,
    DefaultValueT,
    ArgDefault,
    DefaultAttribute,
}

impl DefaultSource {
    fn describe(self) -> &'static str {
        match self {
            DefaultSource::DefaultValue => "`default_value`",
            DefaultSource::DefaultValueT => "`default_value_t`",
            DefaultSource::ArgDefault => "`default`",
            DefaultSource::DefaultAttribute => "`#[default]`",
        }
    }
}

#[derive(Default)]
struct RawField {
    short: Option<(Option<char>, Span)>,
    long: Option<(Option<String>, Span)>,
    flag: Option<Span>,
    positional: Option<Span>,
    default: Option<(DefaultSource, DefaultValue)>,
}

#[derive(Default)]
struct Errors(Option<syn::Error>);

impl Errors {
    fn push(&mut self, error: syn::Error) {
        match &mut self.0 {
            Some(existing) => existing.combine(error),
            None => self.0 = Some(error),
        }
    }

    fn finish<T>(self, value: T) -> syn::Result<T> {
        match self.0 {
            Some(error) => Err(error),
            None => Ok(value),
        }
    }
}

fn is_bool_type(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident == "bool")
            .unwrap_or(false),
        _ => false,
    }
}

fn is_str_reference_type(ty: &Type) -> bool {
    match ty {
        Type::Reference(reference) => match reference.elem.as_ref() {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| segment.ident == "str")
                .unwrap_or(false),
            _ => false,
        },
        _ => false,
    }
}

fn to_kebab_case(name: &str) -> String {
    name.replace('_', "-")
}

fn set_once<T>(slot: &mut Option<T>, value: T, meta: &ParseNestedMeta) -> syn::Result<()> {
    if slot.is_some() {
        let key = meta.path.to_token_stream().to_string();
        return Err(meta.error(format!("duplicate `{key}` attribute")));
    }
    *slot = Some(value);
    Ok(())
}

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
                "conflicting default values: {} and {} cannot be used together; use only `default_value` or `default_value_t`",
                existing.describe(),
                source.describe()
            )
        };
        return Err(syn::Error::new(span, message));
    }
    *slot = Some((source, value));
    Ok(())
}

fn reject_value(meta: &ParseNestedMeta, key: &str) -> syn::Result<()> {
    if meta.input.peek(Token![=]) || meta.input.peek(syn::token::Paren) {
        return Err(meta.error(format!("`{key}` does not take a value")));
    }
    Ok(())
}

fn require_value<'a>(
    meta: &'a ParseNestedMeta,
    key: &str,
    example: &str,
) -> syn::Result<syn::parse::ParseStream<'a>> {
    if !meta.input.peek(Token![=]) {
        return Err(meta.error(format!("`{key}` requires a value, e.g. `{example}`")));
    }
    meta.value()
}

fn parse_arg_entry(meta: ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let key_span = meta.path.span();
    let Some(key) = meta.path.get_ident().map(Ident::to_string) else {
        let path = meta.path.to_token_stream().to_string().replace(' ', "");
        return Err(meta.error(format!(
            "unknown `arg` attribute `{path}`; supported attributes are: {SUPPORTED_ARG_KEYS}"
        )));
    };

    match key.as_str() {
        "short" => {
            let value = if meta.input.peek(Token![=]) {
                match meta.value()?.parse::<Lit>()? {
                    Lit::Char(literal) => (Some(literal.value()), literal.span()),
                    other => {
                        return Err(syn::Error::new(
                            other.span(),
                            "`short` expects a character literal, e.g. `short = 'o'`",
                        ));
                    }
                }
            } else {
                (None, key_span)
            };
            set_once(&mut raw.short, value, &meta)
        }
        "long" => {
            let value = if meta.input.peek(Token![=]) {
                match meta.value()?.parse::<Lit>()? {
                    Lit::Str(literal) => (Some(literal.value()), literal.span()),
                    other => {
                        return Err(syn::Error::new(
                            other.span(),
                            "`long` expects a string literal, e.g. `long = \"output\"`",
                        ));
                    }
                }
            } else {
                (None, key_span)
            };
            set_once(&mut raw.long, value, &meta)
        }
        "flag" => {
            reject_value(&meta, "flag")?;
            set_once(&mut raw.flag, key_span, &meta)
        }
        "positional" => {
            reject_value(&meta, "positional")?;
            set_once(&mut raw.positional, key_span, &meta)
        }
        "default_value" => {
            let literal = match require_value(&meta, "default_value", "default_value = \"4\"")?
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
                key_span,
            )
        }
        "default_value_t" => {
            let expr: Expr =
                require_value(&meta, "default_value_t", "default_value_t = 4")?.parse()?;
            set_default(
                &mut raw.default,
                DefaultSource::DefaultValueT,
                DefaultValue::Expr(expr),
                key_span,
            )
        }
        "default" => {
            let expr: Expr = require_value(&meta, "default", "default = 4")?.parse()?;
            set_default(
                &mut raw.default,
                DefaultSource::ArgDefault,
                DefaultValue::Expr(expr),
                key_span,
            )
        }
        _ => Err(meta.error(format!(
            "unknown `arg` attribute `{key}`; supported attributes are: {SUPPORTED_ARG_KEYS}"
        ))),
    }
}

fn parse_raw_field(field: &Field) -> syn::Result<RawField> {
    let mut raw = RawField::default();
    let mut errors = Errors::default();

    for attr in &field.attrs {
        if attr.path().is_ident("arg") {
            if let Meta::Path(_) = attr.meta {
                errors.push(syn::Error::new_spanned(
                    attr,
                    "expected `#[arg(...)]`, e.g. `#[arg(short, long)]`",
                ));
                continue;
            }
            if let Err(error) = attr.parse_nested_meta(|meta| parse_arg_entry(meta, &mut raw)) {
                errors.push(error);
            }
        } else if attr.path().is_ident("default") {
            let value = match &attr.meta {
                Meta::List(list) => syn::parse2::<Expr>(list.tokens.clone()),
                Meta::NameValue(named) => Ok(named.value.clone()),
                Meta::Path(_) => Err(syn::Error::new_spanned(
                    attr,
                    "`#[default]` requires a value, e.g. `#[arg(default_value_t = 4)]`",
                )),
            };
            let result = value.and_then(|expr| {
                set_default(
                    &mut raw.default,
                    DefaultSource::DefaultAttribute,
                    DefaultValue::Expr(expr),
                    attr.path().span(),
                )
            });
            if let Err(error) = result {
                errors.push(error);
            }
        }
    }

    errors.finish(raw)
}

fn validate_short(short: char, span: Span) -> syn::Result<()> {
    if short == '-' {
        return Err(syn::Error::new(span, "`short` cannot be `-`"));
    }
    if short.is_whitespace() {
        return Err(syn::Error::new(span, "`short` cannot be whitespace"));
    }
    Ok(())
}

fn validate_long(long: &str, span: Span) -> syn::Result<()> {
    if long.is_empty() {
        return Err(syn::Error::new(span, "`long` cannot be empty"));
    }
    if long.starts_with('-') {
        return Err(syn::Error::new(
            span,
            format!(
                "`long` must not start with `-`; write `long = \"{}\"`",
                long.trim_start_matches('-')
            ),
        ));
    }
    if long.contains(|character: char| character.is_whitespace() || character == '=') {
        return Err(syn::Error::new(
            span,
            "`long` cannot contain whitespace or `=`",
        ));
    }
    Ok(())
}

fn build_field(field: &Field) -> syn::Result<FieldModel> {
    let Some(ident) = field.ident.clone() else {
        return Err(syn::Error::new_spanned(
            field,
            "GetArgs requires a struct with named fields",
        ));
    };
    let raw = parse_raw_field(field)?;
    let name = ident.unraw().to_string();
    let is_bool = is_bool_type(&field.ty);
    let named = raw.short.is_some() || raw.long.is_some();
    let mut errors = Errors::default();

    if let Some(span) = raw.positional
        && named
    {
        errors.push(syn::Error::new(
            span,
            "`positional` cannot be combined with `short` or `long`",
        ));
    }

    if let Some(span) = raw.flag {
        if !is_bool {
            errors.push(syn::Error::new(
                span,
                "`flag` can only be used on `bool` fields",
            ));
        } else if !named {
            errors.push(syn::Error::new(
                span,
                "`flag` requires a named option; add `short` and/or `long`, e.g. `#[arg(short, long)]`",
            ));
        }
    } else if is_bool && !named {
        errors.push(syn::Error::new(
            ident.span(),
            "`bool` fields cannot be positional; add `short` and/or `long` to make it a flag, e.g. `#[arg(short, long)]`",
        ));
    }

    let (short, short_span) = match raw.short {
        Some((value, span)) => {
            let short = value.unwrap_or_else(|| {
                name.chars()
                    .next()
                    .expect("field name should never be empty")
            });
            if let Err(error) = validate_short(short, span) {
                errors.push(error);
            }
            (Some(short), span)
        }
        None => (None, ident.span()),
    };

    let (long, long_span) = match raw.long {
        Some((value, span)) => {
            let long = value.unwrap_or_else(|| to_kebab_case(&name));
            if let Err(error) = validate_long(&long, span) {
                errors.push(error);
            }
            (Some(long), span)
        }
        None => (None, ident.span()),
    };

    let kind = if !named {
        FieldKind::Positional
    } else if is_bool {
        FieldKind::Flag
    } else {
        FieldKind::Option
    };

    errors.finish(FieldModel {
        ident,
        name,
        is_str: is_str_reference_type(&field.ty),
        ty: field.ty.clone(),
        kind,
        short,
        long,
        default: raw.default.map(|(_, value)| value),
        short_span,
        long_span,
    })
}

fn validate_struct(fields: &[FieldModel], errors: &mut Errors) {
    let mut shorts: HashMap<char, &str> = HashMap::new();
    let mut longs: HashMap<&str, &str> = HashMap::new();
    let mut optional_positional: Option<&str> = None;

    for field in fields {
        if let Some(short) = field.short {
            match shorts.get(&short) {
                Some(owner) => errors.push(syn::Error::new(
                    field.short_span,
                    format!("short option `-{short}` is already used by field `{owner}`"),
                )),
                None => {
                    shorts.insert(short, &field.name);
                }
            }
        }

        if let Some(long) = &field.long {
            match longs.get(long.as_str()) {
                Some(owner) => errors.push(syn::Error::new(
                    field.long_span,
                    format!("long option `--{long}` is already used by field `{owner}`"),
                )),
                None => {
                    longs.insert(long, &field.name);
                }
            }
        }

        if let FieldKind::Positional = field.kind {
            match (&field.default, optional_positional) {
                (Some(_), None) => optional_positional = Some(&field.name),
                (None, Some(optional)) => errors.push(syn::Error::new(
                    field.ident.span(),
                    format!(
                        "required positional argument `{}` cannot follow optional positional argument `{optional}`",
                        field.name
                    ),
                )),
                _ => {}
            }
        }
    }
}

pub(crate) fn parse_fields(input: &DeriveInput) -> syn::Result<Vec<FieldModel>> {
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(fields) => {
                return Err(syn::Error::new_spanned(
                    fields,
                    "GetArgs requires a struct with named fields; tuple structs are not supported",
                ));
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    &input.ident,
                    "GetArgs requires a struct with named fields; unit structs are not supported",
                ));
            }
        },
        Data::Enum(data) => {
            return Err(syn::Error::new_spanned(
                data.enum_token,
                "GetArgs can only be derived for structs with named fields",
            ));
        }
        Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "GetArgs can only be derived for structs with named fields",
            ));
        }
    };

    let mut errors = Errors::default();
    let mut models = Vec::new();

    for field in fields {
        match build_field(field) {
            Ok(model) => models.push(model),
            Err(error) => errors.push(error),
        }
    }

    validate_struct(&models, &mut errors);
    errors.finish(models)
}
