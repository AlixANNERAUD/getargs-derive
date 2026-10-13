use super::errors::Errors;
use super::field::{FieldKind, FieldModel};
use proc_macro2::Span;
use std::collections::HashMap;

struct Names<'a> {
    help_enabled: bool,
    version_enabled: bool,
    shorts: HashMap<char, &'a str>,
    longs: HashMap<&'a str, &'a str>,
}

impl<'a> Names<'a> {
    fn short(&mut self, short: char, span: Span, owner: &'a str, errors: &mut Errors) {
        if self.help_enabled && short == 'h' {
            errors.push(syn::Error::new(
                span,
                "short option `-h` is reserved for the generated help flag; choose another short name, or add `#[command(disable_help_flag)]` to the struct to handle `-h` yourself",
            ));
        } else if self.version_enabled && short == 'V' {
            errors.push(syn::Error::new(
                span,
                "short option `-V` is reserved for the generated version flag; choose another short name, or remove `version` from `#[command(...)]` to handle `-V` yourself",
            ));
        } else if let Some(existing) = self.shorts.get(&short) {
            errors.push(syn::Error::new(
                span,
                format!("short option `-{short}` is already used by field `{existing}`"),
            ));
        } else {
            self.shorts.insert(short, owner);
        }
    }

    fn long(&mut self, long: &'a str, span: Span, owner: &'a str, errors: &mut Errors) {
        if self.help_enabled && long == "help" {
            errors.push(syn::Error::new(
                span,
                "long option `--help` is reserved for the generated help flag; choose another long name, or add `#[command(disable_help_flag)]` to the struct to handle `--help` yourself",
            ));
        } else if self.version_enabled && long == "version" {
            errors.push(syn::Error::new(
                span,
                "long option `--version` is reserved for the generated version flag; choose another long name, or remove `version` from `#[command(...)]` to handle `--version` yourself",
            ));
        } else if let Some(existing) = self.longs.get(long) {
            errors.push(syn::Error::new(
                span,
                format!("long option `--{long}` is already used by field `{existing}`"),
            ));
        } else {
            self.longs.insert(long, owner);
        }
    }
}

pub(super) fn validate_struct(
    fields: &[FieldModel],
    help_enabled: bool,
    version_enabled: bool,
    errors: &mut Errors,
) {
    let mut names = Names {
        help_enabled,
        version_enabled,
        shorts: HashMap::new(),
        longs: HashMap::new(),
    };
    let mut optional_positional: Option<&str> = None;

    for field in fields {
        if let Some(short) = field.short {
            names.short(short, field.short_span, &field.name, errors);
        }
        for alias in &field.short_aliases {
            names.short(alias.name, alias.span, &field.name, errors);
        }
        if let Some(long) = &field.long {
            names.long(long, field.long_span, &field.name, errors);
        }
        for alias in &field.long_aliases {
            names.long(&alias.name, alias.span, &field.name, errors);
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
