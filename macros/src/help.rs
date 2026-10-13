use crate::model::{ContainerModel, DefaultValue, FieldKind, FieldModel, FlagAction};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, ExprLit, Lit, UnOp};

struct Entry {
    label: String,
    description: String,
}

fn expr_text(expr: &Expr) -> String {
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

fn quote_if_spaced(text: String) -> String {
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

fn description(field: &FieldModel) -> String {
    let mut parts = Vec::new();
    if let Some(help) = field.help.as_deref().filter(|help| !help.is_empty()) {
        parts.push(help.to_owned());
    }
    if let Some(env) = &field.env {
        parts.push(format!("[env: {env}=]"));
    }
    if let (Some(default), FieldKind::Positional | FieldKind::Option) =
        (&field.default, &field.kind)
    {
        parts.push(format!("[default: {}]", default_text(default)));
    }
    let aliases: Vec<String> = field
        .long_aliases
        .iter()
        .filter(|alias| alias.visible)
        .map(|alias| format!("--{}", alias.name))
        .collect();
    if !aliases.is_empty() {
        parts.push(format!("[aliases: {}]", aliases.join(", ")));
    }
    let short_aliases: Vec<String> = field
        .short_aliases
        .iter()
        .filter(|alias| alias.visible)
        .map(|alias| format!("-{}", alias.name))
        .collect();
    if !short_aliases.is_empty() {
        parts.push(format!("[short aliases: {}]", short_aliases.join(", ")));
    }
    parts.join(" ")
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

fn render_section(output: &mut String, title: &str, entries: &[Entry]) {
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
    output.push(':');
    for entry in entries {
        output.push_str("\n  ");
        if entry.description.is_empty() {
            output.push_str(&entry.label);
            continue;
        }
        output.push_str(&format!("{:<width$}  ", entry.label));
        for (index, line) in entry.description.lines().enumerate() {
            if index > 0 {
                output.push('\n');
                if !line.is_empty() {
                    output.push_str(&indent);
                }
            }
            output.push_str(line);
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
            description: "Print help".to_owned(),
        });
    }

    if container.version.is_some() {
        options.push(Entry {
            label: "-V, --version".to_owned(),
            description: "Print version".to_owned(),
        });
    }

    let mut prefix = String::new();
    if let Some(about) = container.about.as_deref().filter(|about| !about.is_empty()) {
        prefix.push_str(about);
        prefix.push_str("\n\n");
    }
    prefix.push_str("Usage: ");

    let mut suffix = String::new();
    if !options.is_empty() {
        suffix.push_str(" [OPTIONS]");
    }
    suffix.push_str(&usage);
    render_section(&mut suffix, "Arguments", &arguments);
    render_section(&mut suffix, "Options", &options);

    let name = command_name(container);

    quote! { ::core::concat!(#prefix, #name, #suffix) }
}

pub(crate) fn command_name(container: &ContainerModel) -> TokenStream {
    match &container.name {
        Some(name) => quote! { #name },
        None => quote! { ::core::env!("CARGO_PKG_NAME") },
    }
}
