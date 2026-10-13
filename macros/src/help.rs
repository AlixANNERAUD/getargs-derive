mod layout;
mod text;
mod wrap;

use crate::model::{ContainerModel, DefaultValue, FieldKind, FieldModel, FlagAction};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, ExprLit, Lit, UnOp, ext::IdentExt};
use text::{Segment, Text};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Short,
    Long,
}

struct Entry {
    label: String,
    description: Text,
}

const NEXT_LINE_INDENT: usize = 10;

struct Style {
    mode: Mode,
    term_width: Option<usize>,
}

impl Style {
    fn next_line(&self, taken: usize, description: &str) -> bool {
        match (self.mode, self.term_width) {
            (Mode::Long, _) => true,
            (Mode::Short, None) => false,
            (Mode::Short, Some(width)) => {
                width >= taken && taken * 5 > width * 2 && wrap::width(description) > width - taken
            }
        }
    }
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

fn description(field: &FieldModel, mode: Mode) -> Text {
    let help = match mode {
        Mode::Short => field.help.as_deref(),
        Mode::Long => field.long_help.as_deref().or(field.help.as_deref()),
    };
    let mut parts = Vec::new();
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
    if let Some(help) = help.filter(|help| !help.is_empty()) {
        text.push_str(help);
        if !parts.is_empty() {
            text.push_str(match mode {
                Mode::Short => " ",
                Mode::Long => "\n\n",
            });
        }
    }
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

fn render_section(output: &mut Text, title: &str, entries: Vec<Entry>, style: &Style) {
    if entries.is_empty() {
        return;
    }
    let width = entries
        .iter()
        .map(|entry| entry.label.chars().count())
        .max()
        .unwrap_or(0);
    let taken = width + 4;
    let indent = " ".repeat(taken);
    let next_line_indent = " ".repeat(NEXT_LINE_INDENT);

    output.push_str("\n\n");
    output.push_str(title);
    output.push_str(":");
    for (index, entry) in entries.into_iter().enumerate() {
        if index > 0 && style.mode == Mode::Long {
            output.push_str("\n");
        }
        output.push_str("\n  ");
        if entry.description.is_empty() {
            output.push_str(&entry.label);
            continue;
        }
        if let Some(description) = entry.description.as_static() {
            if style.next_line(taken, description) {
                let text = wrap::wrap(
                    description,
                    style
                        .term_width
                        .map(|width| width.saturating_sub(NEXT_LINE_INDENT)),
                );
                output.push_str(&entry.label);
                output.push_str("\n");
                output.push_str(&next_line_indent);
                output.push_str(&wrap::indent(&text, &next_line_indent));
            } else {
                let text = wrap::wrap(
                    description,
                    style.term_width.map(|width| width.saturating_sub(taken)),
                );
                output.push_str(&format!("{:<width$}  ", entry.label));
                output.push_str(&wrap::indent(&text, &indent));
            }
            continue;
        }
        let indent = if style.mode == Mode::Long {
            output.push_str(&entry.label);
            output.push_str("\n");
            output.push_str(&next_line_indent);
            &next_line_indent
        } else {
            output.push_str(&format!("{:<width$}  ", entry.label));
            &indent
        };
        for segment in entry.description.segments() {
            match segment {
                Segment::Text(text) => output.push_str(&wrap::indent(&text, indent)),
                other => output.push(other),
            }
        }
    }
}

pub(crate) fn has_long_help(container: &ContainerModel, fields: &[FieldModel]) -> bool {
    container.long_about.is_some()
        || fields
            .iter()
            .any(|field| !field.hide && field.long_help.is_some())
}

fn help_flag_description(mode: Mode, long_help: bool) -> &'static str {
    match (mode, long_help) {
        (_, false) => "Print help",
        (Mode::Short, true) => "Print help (see more with '--help')",
        (Mode::Long, true) => "Print help (see a summary with '-h')",
    }
}

fn push_paragraph(output: &mut String, text: Option<&str>, style: &Style) {
    if let Some(text) = text.filter(|text| !text.is_empty()) {
        output.push_str(&wrap::wrap(text, style.term_width));
        output.push_str("\n\n");
    }
}

pub(crate) fn help_text(
    container: &ContainerModel,
    fields: &[FieldModel],
    mode: Mode,
) -> TokenStream {
    let style = Style {
        mode,
        term_width: container.max_term_width,
    };
    let mut usage = String::new();
    let mut has_options = false;

    for field in fields.iter().filter(|field| !field.hide) {
        match field.kind {
            FieldKind::Positional => {
                usage.push(' ');
                usage.push_str(&positional_label(field));
            }
            FieldKind::Flag(_) | FieldKind::Option => has_options = true,
        }
    }

    let mut sections: Vec<(&str, Vec<Entry>)> = layout::sections(fields)
        .into_iter()
        .map(|section| {
            let entries = section
                .fields
                .into_iter()
                .map(|field| Entry {
                    label: match field.kind {
                        FieldKind::Positional => positional_label(field),
                        FieldKind::Flag(_) | FieldKind::Option => option_label(field),
                    },
                    description: description(field, mode),
                })
                .collect();
            (section.title, entries)
        })
        .collect();

    if let Some((_, options)) = sections
        .iter_mut()
        .find(|(title, _)| *title == layout::OPTIONS)
    {
        if !container.disable_help_flag {
            options.push(Entry {
                label: "-h, --help".to_owned(),
                description: plain(help_flag_description(
                    mode,
                    has_long_help(container, fields),
                )),
            });
            has_options = true;
        }

        if container.version.is_some() {
            options.push(Entry {
                label: "-V, --version".to_owned(),
                description: plain("Print version"),
            });
            has_options = true;
        }
    }

    let about = match mode {
        Mode::Short => container.about.as_deref(),
        Mode::Long => container
            .long_about
            .as_deref()
            .or(container.about.as_deref()),
    };
    let mut prefix = String::new();
    push_paragraph(&mut prefix, container.before_help.as_deref(), &style);
    push_paragraph(&mut prefix, about, &style);
    prefix.push_str("Usage: ");

    let mut suffix = Text::default();
    if has_options {
        suffix.push_str(" [OPTIONS]");
    }
    suffix.push_str(&usage);
    for (title, entries) in sections {
        render_section(&mut suffix, title, entries, &style);
    }
    if let Some(after) = container
        .after_help
        .as_deref()
        .filter(|text| !text.is_empty())
    {
        suffix.push_str("\n\n");
        suffix.push_str(&wrap::wrap(after, style.term_width));
    }

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
