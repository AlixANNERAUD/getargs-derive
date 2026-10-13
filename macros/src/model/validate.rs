use super::errors::Errors;
use super::field::{FieldKind, FieldModel};
use std::collections::HashMap;

pub(super) fn validate_struct(
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
