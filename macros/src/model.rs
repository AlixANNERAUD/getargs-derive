use proc_macro2::Span;
use quote::ToTokens;
use std::collections::HashMap;
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, Field, Fields, GenericArgument, Ident, Lit,
    LitStr, Meta, MetaNameValue, PathArguments, Token, Type, ext::IdentExt, meta::ParseNestedMeta,
    spanned::Spanned,
};

const SUPPORTED_ARG_KEYS: &str = "`short`, `long`, `help`, `value_name`, `default_value`, `default_value_t`, `flag`, `positional`, `default` (legacy)";

const SUPPORTED_COMMAND_KEYS: &str = "`name`, `about`, `version`, `disable_help_flag`";

pub(crate) struct ContainerModel {
    pub(crate) name: Option<String>,
    pub(crate) about: Option<String>,
    pub(crate) version: Option<Version>,
    pub(crate) disable_help_flag: bool,
}

pub(crate) enum Version {
    Package,
    Literal(String),
}

pub(crate) enum FieldKind {
    Positional,
    Flag,
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
    pub(crate) default: Option<DefaultValue>,
    pub(crate) is_str: bool,
    pub(crate) help: Option<String>,
    pub(crate) value_name: String,
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

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    let segment = path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    match arguments.args.first() {
        Some(GenericArgument::Type(inner)) if arguments.args.len() == 1 => Some(inner),
        _ => None,
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

fn parse_string(meta: &ParseNestedMeta, key: &str, example: &str) -> syn::Result<LitStr> {
    match require_value(meta, key, example)?.parse::<Lit>()? {
        Lit::Str(literal) => Ok(literal),
        other => Err(syn::Error::new(
            other.span(),
            format!("`{key}` expects a string literal, e.g. `{example}`"),
        )),
    }
}

fn doc_lines(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| match &attr.meta {
            Meta::NameValue(MetaNameValue {
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(literal),
                        ..
                    }),
                ..
            }) => Some(literal.value()),
            _ => None,
        })
        .flat_map(|value| {
            value
                .split('\n')
                .map(|line| line.trim().to_owned())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn short_doc(attrs: &[Attribute]) -> Option<String> {
    let lines = doc_lines(attrs);
    let paragraph: Vec<&str> = lines
        .iter()
        .map(String::as_str)
        .skip_while(|line| line.is_empty())
        .take_while(|line| !line.is_empty())
        .collect();
    if paragraph.is_empty() {
        return None;
    }
    let mut text = paragraph.join(" ");
    if text.ends_with('.') && !text.ends_with("..") {
        text.pop();
    }
    Some(text)
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
        "help" => {
            let literal = parse_string(&meta, "help", "help = \"Print more output\"")?;
            set_once(&mut raw.help, literal.value(), &meta)
        }
        "value_name" => {
            let literal = parse_string(&meta, "value_name", "value_name = \"FILE\"")?;
            let value = literal.value();
            if value.is_empty() || value.contains(char::is_whitespace) {
                return Err(syn::Error::new(
                    literal.span(),
                    "`value_name` cannot be empty or contain whitespace",
                ));
            }
            set_once(&mut raw.value_name, (value, literal.span()), &meta)
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

    if let Some((_, span)) = &raw.value_name
        && is_bool
        && named
    {
        errors.push(syn::Error::new(
            *span,
            "`value_name` cannot be used on flags because they do not take a value",
        ));
    }

    let kind = if !named {
        FieldKind::Positional
    } else if is_bool {
        FieldKind::Flag
    } else {
        FieldKind::Option
    };

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
        default: raw.default.map(|(_, value)| value),
        help: raw.help.or_else(|| short_doc(&field.attrs)),
        value_name,
        short_span,
        long_span,
    })
}

fn validate_struct(
    fields: &[FieldModel],
    help_enabled: bool,
    version_enabled: bool,
    errors: &mut Errors,
) {
    let mut shorts: HashMap<char, &str> = HashMap::new();
    let mut longs: HashMap<&str, &str> = HashMap::new();
    let mut optional_positional: Option<&str> = None;

    for field in fields {
        if help_enabled && field.short == Some('h') {
            errors.push(syn::Error::new(
                field.short_span,
                "short option `-h` is reserved for the generated help flag; choose another short name, or add `#[command(disable_help_flag)]` to the struct to handle `-h` yourself",
            ));
        } else if version_enabled && field.short == Some('V') {
            errors.push(syn::Error::new(
                field.short_span,
                "short option `-V` is reserved for the generated version flag; choose another short name, or remove `version` from `#[command(...)]` to handle `-V` yourself",
            ));
        } else if let Some(short) = field.short {
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

        if help_enabled && field.long.as_deref() == Some("help") {
            errors.push(syn::Error::new(
                field.long_span,
                "long option `--help` is reserved for the generated help flag; choose another long name, or add `#[command(disable_help_flag)]` to the struct to handle `--help` yourself",
            ));
        } else if version_enabled && field.long.as_deref() == Some("version") {
            errors.push(syn::Error::new(
                field.long_span,
                "long option `--version` is reserved for the generated version flag; choose another long name, or remove `version` from `#[command(...)]` to handle `--version` yourself",
            ));
        } else if let Some(long) = &field.long {
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
            match (field.is_optional(), optional_positional) {
                (true, None) => optional_positional = Some(&field.name),
                (false, Some(optional)) => errors.push(syn::Error::new(
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

fn parse_command_entry(
    meta: ParseNestedMeta,
    name: &mut Option<String>,
    about: &mut Option<String>,
    version: &mut Option<Version>,
    disable_help_flag: &mut Option<()>,
) -> syn::Result<()> {
    let Some(key) = meta.path.get_ident().map(Ident::to_string) else {
        let path = meta.path.to_token_stream().to_string().replace(' ', "");
        return Err(meta.error(format!(
            "unknown `command` attribute `{path}`; supported attributes are: {SUPPORTED_COMMAND_KEYS}"
        )));
    };

    match key.as_str() {
        "name" => {
            let literal = parse_string(&meta, "name", "name = \"tool\"")?;
            let value = literal.value();
            if value.is_empty() {
                return Err(syn::Error::new(literal.span(), "`name` cannot be empty"));
            }
            set_once(name, value, &meta)
        }
        "about" => {
            let literal = parse_string(&meta, "about", "about = \"Does things\"")?;
            set_once(about, literal.value(), &meta)
        }
        "version" => {
            let value = if meta.input.peek(Token![=]) {
                let literal = parse_string(&meta, "version", "version = \"1.0.0\"")?;
                let value = literal.value();
                if value.trim().is_empty() {
                    return Err(syn::Error::new(literal.span(), "`version` cannot be empty"));
                }
                Version::Literal(value)
            } else {
                reject_value(&meta, "version")?;
                Version::Package
            };
            set_once(version, value, &meta)
        }
        "disable_help_flag" => {
            reject_value(&meta, "disable_help_flag")?;
            set_once(disable_help_flag, (), &meta)
        }
        _ => Err(meta.error(format!(
            "unknown `command` attribute `{key}`; supported attributes are: {SUPPORTED_COMMAND_KEYS}"
        ))),
    }
}

fn parse_container(input: &DeriveInput) -> syn::Result<ContainerModel> {
    let mut name = None;
    let mut about = None;
    let mut version = None;
    let mut disable_help_flag = None;
    let mut errors = Errors::default();

    for attr in &input.attrs {
        if !attr.path().is_ident("command") {
            continue;
        }
        if let Meta::Path(_) = attr.meta {
            errors.push(syn::Error::new_spanned(
                attr,
                "expected `#[command(...)]`, e.g. `#[command(name = \"tool\")]`",
            ));
            continue;
        }
        let result = attr.parse_nested_meta(|meta| {
            parse_command_entry(
                meta,
                &mut name,
                &mut about,
                &mut version,
                &mut disable_help_flag,
            )
        });
        if let Err(error) = result {
            errors.push(error);
        }
    }

    errors.finish(ContainerModel {
        name,
        about: about.or_else(|| short_doc(&input.attrs)),
        version,
        disable_help_flag: disable_help_flag.is_some(),
    })
}

pub(crate) fn parse(input: &DeriveInput) -> syn::Result<(ContainerModel, Vec<FieldModel>)> {
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
    let container = match parse_container(input) {
        Ok(container) => Some(container),
        Err(error) => {
            errors.push(error);
            None
        }
    };
    let mut models = Vec::new();

    for field in fields {
        match build_field(field) {
            Ok(model) => models.push(model),
            Err(error) => errors.push(error),
        }
    }

    let help_enabled = container
        .as_ref()
        .is_some_and(|container| !container.disable_help_flag);
    let version_enabled = container
        .as_ref()
        .is_some_and(|container| container.version.is_some());
    validate_struct(&models, help_enabled, version_enabled, &mut errors);
    match container {
        Some(container) => errors.finish((container, models)),
        None => Err(errors.0.expect("container errors are recorded")),
    }
}
