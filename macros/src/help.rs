mod text;

use crate::model::{ContainerModel, DefaultValue, FieldKind, FieldModel, FlagAction};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, ExprLit, Lit, UnOp, ext::IdentExt};
use text::{Segment, Text};

struct Entry {
    label: String,
    description: Text,
}

pub(crate) fn expr_text(expr: &Expr) -> String {
    match expr {
        Expr::Lit(literal) => match &literal.lit {
            Lit::Str(value) => value.value(),
            Lit::Int(value) => value.base10_digits().to_owned(),
            Lit::Float(value) => value.base10_digits().to_owned(),
            Lit::Char(value) => value.value().to_string(),
            Lit::Bool(value) => value.value.to_string(),
            other => other.to_token_stream().to_string(),
        },
        Expr::Unary(unary) if matches!(unary.op, UnOp::Neg(_)) => {
            format!("-{}", expr_text(&unary.expr))
        }
        other => other.to_token_stream().to_string(),
    }
}

pub(crate) fn quote_if_spaced(text: String) -> String {
    if text.is_empty() || text.contains(char::is_whitespace) {
        format!("{text:?}")
    } else {
        text
    }
}

fn default_text(default: &DefaultValue) -> String {
    match default {
        DefaultValue::Literal(literal) => quote_if_spaced(literal.value()),
        DefaultValue::Expr(Expr::Lit(ExprLit {
            lit: Lit::Str(literal),
            ..
        })) => quote_if_spaced(literal.value()),
        DefaultValue::Expr(expr) => expr_text(expr),
    }
}

fn value_enum_default(field: &FieldModel, default: &DefaultValue) -> Option<Segment> {
    let DefaultValue::Expr(Expr::Path(path)) = default else {
        return None;
    };
    let ident = path.path.segments.last()?.ident.unraw().to_string();
    let fallback = expr_text(&Expr::Path(path.clone()));
    let value_ty = &field.value_ty;
    Some(Segment::Str(quote! {
        getargs_derive::__value_name(
            <#value_ty as getargs_derive::ValueEnum>::__VARIANT_IDENTS,
            <#value_ty as getargs_derive::ValueEnum>::POSSIBLE_VALUES,
            #ident,
            #fallback,
        )
    }))
}

fn possible_values(field: &FieldModel) -> Option<Segment> {
    if field.value_enum {
        let value_ty = &field.value_ty;
        return Some(Segment::List(
            quote! { <#value_ty as getargs_derive::ValueEnum>::POSSIBLE_VALUES },
        ));
    }
    if field.possible_values.is_empty() {
        return None;
    }
    let values: Vec<String> = field
        .possible_values
        .iter()
        .map(|value| quote_if_spaced(value.clone()))
        .collect();
    Some(Segment::Text(values.join(", ")))
}

fn description(field: &FieldModel) -> Text {
    let mut parts = Vec::new();
    if let Some(help) = field.help.as_deref().filter(|help| !help.is_empty()) {
        parts.push(vec![Segment::Text(help.to_owned())]);
    }
    if let Some(env) = &field.env {
        parts.push(vec![Segment::Text(format!("[env: {env}=]"))]);
    }
    let takes_value = matches!(field.kind, FieldKind::Positional | FieldKind::Option);
    if let (Some(default), true) = (&field.default, takes_value) {
        let value = field
            .value_enum
            .then(|| value_enum_default(field, default))
            .flatten()
            .unwrap_or_else(|| Segment::Text(default_text(default)));
        parts.push(vec![
            Segment::Text("[default: ".to_owned()),
            value,
            Segment::Text("]".to_owned()),
        ]);
    }
    let aliases: Vec<String> = field
        .long_aliases
        .iter()
        .filter(|alias| alias.visible)
        .map(|alias| format!("--{}", alias.name))
        .collect();
    if !aliases.is_empty() {
        parts.push(vec![Segment::Text(format!(
            "[aliases: {}]",
            aliases.join(", ")
        ))]);
    }
    let short_aliases: Vec<String> = field
        .short_aliases
        .iter()
        .filter(|alias| alias.visible)
        .map(|alias| format!("-{}", alias.name))
        .collect();
    if !short_aliases.is_empty() {
        parts.push(vec![Segment::Text(format!(
            "[short aliases: {}]",
            short_aliases.join(", ")
        ))]);
    }
    if let (Some(values), true) = (possible_values(field), takes_value) {
        parts.push(vec![
            Segment::Text("[possible values: ".to_owned()),
            values,
            Segment::Text("]".to_owned()),
        ]);
    }
    let mut text = Text::default();
    for (index, part) in parts.into_iter().enumerate() {
        if index > 0 {
            text.push_str(" ");
        }
        for segment in part {
            text.push(segment);
        }
    }
    text
}

fn positional_label(field: &FieldModel) -> String {
    if field.is_optional() {
        format!("[{}]", field.value_name)
    } else {
        format!("<{}>", field.value_name)
    }
}

fn option_label(field: &FieldModel) -> String {
    let mut label = match (field.short, &field.long) {
        (Some(short), Some(long)) => format!("-{short}, --{long}"),
        (Some(short), None) => format!("-{short}"),
        (None, Some(long)) => format!("    --{long}"),
        (None, None) => unreachable!("named options always have a short or long name"),
    };
    match field.kind {
        FieldKind::Option => label.push_str(&format!(" <{}>", field.value_name)),
        FieldKind::Flag(FlagAction::Count) => label.push_str("..."),
        _ => {}
    }
    label
}

fn render_section(output: &mut Text, title: &str, entries: Vec<Entry>) {
    if entries.is_empty() {
        return;
    }
    let width = entries
        .iter()
        .map(|entry| entry.label.chars().count())
        .max()
        .unwrap_or(0);
    let indent = " ".repeat(width + 4);

    output.push_str("\n\n");
    output.push_str(title);
    output.push_str(":");
    for entry in entries {
        output.push_str("\n  ");
        if entry.description.is_empty() {
            output.push_str(&entry.label);
            continue;
        }
        output.push_str(&format!("{:<width$}  ", entry.label));
        for segment in entry.description.segments() {
            let Segment::Text(description) = segment else {
                output.push(segment);
                continue;
            };
            for (index, line) in description.lines().enumerate() {
                if index > 0 {
                    output.push_str("\n");
                    if !line.is_empty() {
                        output.push_str(&indent);
                    }
                }
                output.push_str(line);
            }
        }
    }
}

pub(crate) fn help_text(container: &ContainerModel, fields: &[FieldModel]) -> TokenStream {
    let mut arguments = Vec::new();
    let mut options = Vec::new();
    let mut usage = String::new();

    for field in fields {
        match field.kind {
            FieldKind::Positional => {
                let label = positional_label(field);
                usage.push(' ');
                usage.push_str(&label);
                arguments.push(Entry {
                    label,
                    description: description(field),
                });
            }
            FieldKind::Flag(_) | FieldKind::Option => options.push(Entry {
                label: option_label(field),
                description: description(field),
            }),
        }
    }

    if !container.disable_help_flag {
        options.push(Entry {
            label: "-h, --help".to_owned(),
            description: plain("Print help"),
        });
    }

    if container.version.is_some() {
        options.push(Entry {
            label: "-V, --version".to_owned(),
            description: plain("Print version"),
        });
    }

    let mut prefix = String::new();
    if let Some(about) = container.about.as_deref().filter(|about| !about.is_empty()) {
        prefix.push_str(about);
        prefix.push_str("\n\n");
    }
    prefix.push_str("Usage: ");

    let mut suffix = Text::default();
    if !options.is_empty() {
        suffix.push_str(" [OPTIONS]");
    }
    suffix.push_str(&usage);
    render_section(&mut suffix, "Arguments", arguments);
    render_section(&mut suffix, "Options", options);

    let name = command_name(container);

    suffix.into_tokens(quote! { ::core::concat!(#prefix, #name) })
}

fn plain(text: &str) -> Text {
    let mut output = Text::default();
    output.push_str(text);
    output
}

pub(crate) fn command_name(container: &ContainerModel) -> TokenStream {
    match &container.name {
        Some(name) => quote! { #name },
        None => quote! { ::core::env!("CARGO_PKG_NAME") },
    }
}
