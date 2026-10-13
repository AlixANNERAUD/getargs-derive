mod action;
mod aliases;
mod defaults;
mod env;
mod help;
mod kind;
mod names;
mod relations;

use super::attr::{Key, dispatch};
use super::doc::short_doc;
use super::errors::Errors;
use super::types::{is_bool_type, is_count_type, is_str_reference_type, option_inner_type};
pub(crate) use action::FlagAction;
pub(crate) use aliases::Alias;
use defaults::set_default;
use proc_macro2::Span;
pub(crate) use relations::Relations;
use syn::{Expr, Field, Ident, LitStr, Meta, Type, ext::IdentExt, spanned::Spanned};

const ARG_KEYS: &[Key<RawField>] = &[
    Key::new("action", action::action),
    Key::new("alias", aliases::alias),
    Key::new("aliases", aliases::aliases),
    Key::new("conflicts_with", relations::conflicts_with),
    Key::legacy("default", defaults::default),
    Key::new("default_value", defaults::default_value),
    Key::new("default_value_t", defaults::default_value_t),
    Key::new("env", env::env),
    Key::new("flag", kind::flag),
    Key::new("group", relations::group),
    Key::new("help", help::help),
    Key::new("long", names::long),
    Key::new("positional", kind::positional),
    Key::new(
        "required_unless_present",
        relations::required_unless_present,
    ),
    Key::new("requires", relations::requires),
    Key::new("short", names::short),
    Key::new("short_alias", aliases::short_alias),
    Key::new("short_aliases", aliases::short_aliases),
    Key::new("value_name", help::value_name),
    Key::new("visible_alias", aliases::visible_alias),
    Key::new("visible_aliases", aliases::visible_aliases),
    Key::new("visible_short_alias", aliases::visible_short_alias),
    Key::new("visible_short_aliases", aliases::visible_short_aliases),
];

pub(crate) enum FieldKind {
    Positional,
    Flag(FlagAction),
    Option,
}

pub(crate) enum DefaultValue {
    Literal(LitStr),
    Expr(Expr),
}

impl DefaultValue {
    fn span(&self) -> Span {
        match self {
            DefaultValue::Literal(literal) => literal.span(),
            DefaultValue::Expr(expr) => expr.span(),
        }
    }
}

pub(crate) struct FieldModel {
    pub(crate) ident: Ident,
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) value_ty: Type,
    pub(crate) optional: bool,
    pub(crate) kind: FieldKind,
    pub(crate) short: Option<char>,
    pub(crate) long: Option<String>,
    pub(crate) short_aliases: Vec<Alias<char>>,
    pub(crate) long_aliases: Vec<Alias<String>>,
    pub(crate) default: Option<DefaultValue>,
    pub(crate) env: Option<String>,
    pub(crate) is_str: bool,
    pub(crate) help: Option<String>,
    pub(crate) value_name: String,
    pub(crate) relations: Relations,
    pub(super) short_span: Span,
    pub(super) long_span: Span,
}

impl FieldModel {
    pub(crate) fn error_name(&self) -> String {
        let option = match (self.short, &self.long) {
            (_, Some(long)) => format!("--{long}"),
            (Some(short), None) => format!("-{short}"),
            (None, None) => return format!("<{}>", self.value_name),
        };
        match self.kind {
            FieldKind::Flag(_) => option,
            _ => format!("{option} <{}>", self.value_name),
        }
    }

    pub(crate) fn is_optional(&self) -> bool {
        self.optional || self.default.is_some()
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
    help: Option<String>,
    value_name: Option<(String, Span)>,
    action: Option<(FlagAction, syn::Path)>,
    relations: Relations,
    short_aliases: Vec<Alias<char>>,
    long_aliases: Vec<Alias<String>>,
    env: Option<(Option<String>, Span)>,
}

fn to_kebab_case(name: &str) -> String {
    name.replace('_', "-")
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
            if let Err(error) =
                attr.parse_nested_meta(|meta| dispatch(meta, "arg", ARG_KEYS, &mut raw))
            {
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

fn validate_short(key: &str, short: char, span: Span) -> syn::Result<()> {
    if short == '-' {
        return Err(syn::Error::new(span, format!("`{key}` cannot be `-`")));
    }
    if short.is_whitespace() {
        return Err(syn::Error::new(
            span,
            format!("`{key}` cannot be whitespace"),
        ));
    }
    Ok(())
}

fn validate_long(key: &str, long: &str, span: Span) -> syn::Result<()> {
    if long.is_empty() {
        return Err(syn::Error::new(span, format!("`{key}` cannot be empty")));
    }
    if long.starts_with('-') {
        return Err(syn::Error::new(
            span,
            format!(
                "`{key}` must not start with `-`; write `{}\"{}\"`",
                match key.ends_with("aliases") {
                    true => String::new(),
                    false => format!("{key} = "),
                },
                long.trim_start_matches('-')
            ),
        ));
    }
    if long.contains(|character: char| character.is_whitespace() || character == '=') {
        return Err(syn::Error::new(
            span,
            format!("`{key}` cannot contain whitespace or `=`"),
        ));
    }
    Ok(())
}

pub(super) fn build_field(field: &Field) -> syn::Result<FieldModel> {
    let Some(ident) = field.ident.clone() else {
        return Err(syn::Error::new_spanned(
            field,
            "GetArgs requires a struct with named fields",
        ));
    };
    let raw = parse_raw_field(field)?;
    let name = ident.unraw().to_string();
    let inner_ty = option_inner_type(&field.ty);
    let value_ty = inner_ty.unwrap_or(&field.ty);
    let is_bool = is_bool_type(&field.ty);
    let named = raw.short.is_some() || raw.long.is_some();
    let mut errors = Errors::default();

    if let Some(inner) = inner_ty {
        if is_bool_type(inner) {
            errors.push(syn::Error::new_spanned(
                &field.ty,
                "`Option<bool>` fields are not supported; use a `bool` field with `short` and/or `long` to make it a flag",
            ));
        } else if option_inner_type(inner).is_some() {
            errors.push(syn::Error::new_spanned(
                &field.ty,
                "`Option<Option<T>>` fields are not supported; use `Option<T>`",
            ));
        }
        if let Some((source, value)) = &raw.default {
            errors.push(syn::Error::new(
                value.span(),
                format!(
                    "`Option<T>` fields are `None` when absent and cannot have a default; remove {} or change the field type to `T`",
                    source.describe()
                ),
            ));
        }
    }

    if let Some(span) = raw.positional
        && named
    {
        errors.push(syn::Error::new(
            span,
            "`positional` cannot be combined with `short` or `long`",
        ));
    }

    if !named
        && let Some(span) = raw
            .short_aliases
            .first()
            .map(|alias| alias.span)
            .or_else(|| raw.long_aliases.first().map(|alias| alias.span))
    {
        errors.push(syn::Error::new(
            span,
            "aliases require a named option; add `short` and/or `long`, e.g. `#[arg(long, alias = \"other\")]`",
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
    } else if is_bool && !named && raw.action.is_none() {
        errors.push(syn::Error::new(
            ident.span(),
            "`bool` fields cannot be positional; add `short` and/or `long` to make it a flag, e.g. `#[arg(short, long)]`",
        ));
    }

    if let Some((action, path)) = &raw.action {
        let name = action.name();
        if !named {
            errors.push(syn::Error::new_spanned(
                path,
                format!(
                    "`action = {name}` requires a named option; add `short` and/or `long`, e.g. `#[arg(short, long, action = {name})]`"
                ),
            ));
        }
        match action {
            FlagAction::SetTrue | FlagAction::SetFalse if !is_bool => {
                errors.push(syn::Error::new_spanned(
                    path,
                    format!("`action = {name}` can only be used on `bool` fields"),
                ));
            }
            FlagAction::Count if !is_count_type(&field.ty) => {
                errors.push(syn::Error::new_spanned(
                    path,
                    "`action = Count` requires an integer field type, e.g. `u8`",
                ));
            }
            _ => {}
        }
    }

    let (short, short_span) = match raw.short {
        Some((value, span)) => {
            let short = value.unwrap_or_else(|| {
                name.chars()
                    .next()
                    .expect("field name should never be empty")
            });
            if let Err(error) = validate_short("short", short, span) {
                errors.push(error);
            }
            (Some(short), span)
        }
        None => (None, ident.span()),
    };

    let (long, long_span) = match raw.long {
        Some((value, span)) => {
            let long = value.unwrap_or_else(|| to_kebab_case(&name));
            if let Err(error) = validate_long("long", &long, span) {
                errors.push(error);
            }
            (Some(long), span)
        }
        None => (None, ident.span()),
    };

    if let Some((_, span)) = &raw.value_name
        && (is_bool || raw.action.is_some())
        && named
    {
        errors.push(syn::Error::new(
            *span,
            "`value_name` cannot be used on flags because they do not take a value",
        ));
    }

    let kind = if !named {
        FieldKind::Positional
    } else if let Some((action, _)) = &raw.action {
        FieldKind::Flag(*action)
    } else if is_bool {
        FieldKind::Flag(FlagAction::SetTrue)
    } else {
        FieldKind::Option
    };

    let env = raw.env.map(|(value, span)| {
        let env = value.unwrap_or_else(|| name.to_uppercase());
        if let Err(error) = env::validate_env(&env, span) {
            errors.push(error);
        }
        if let FieldKind::Flag(_) = kind {
            errors.push(syn::Error::new(
                span,
                "`env` cannot be used on flags; it is only supported on options and positional arguments",
            ));
        }
        env
    });

    let value_name = raw
        .value_name
        .map(|(value, _)| value)
        .unwrap_or_else(|| name.to_uppercase());

    errors.finish(FieldModel {
        ident,
        name,
        is_str: is_str_reference_type(value_ty),
        value_ty: value_ty.clone(),
        optional: inner_ty.is_some(),
        ty: field.ty.clone(),
        kind,
        short,
        long,
        short_aliases: raw.short_aliases,
        long_aliases: raw.long_aliases,
        default: raw.default.map(|(_, value)| value),
        env,
        help: raw.help.or_else(|| short_doc(&field.attrs)),
        value_name,
        relations: raw.relations,
        short_span,
        long_span,
    })
}
