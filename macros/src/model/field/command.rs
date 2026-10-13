use crate::model::attr::{Key, dispatch, reject_value, set_once};
use crate::model::errors::Errors;
use proc_macro2::Span;
use syn::{Field, Meta, meta::ParseNestedMeta, spanned::Spanned};

const FIELD_COMMAND_KEYS: &[Key<RawCommand>] = &[Key::new("flatten", flatten)];

#[derive(Default)]
pub(super) struct RawCommand {
    pub(super) flatten: Option<Span>,
}

fn flatten(meta: &ParseNestedMeta, raw: &mut RawCommand) -> syn::Result<()> {
    reject_value(meta, "flatten")?;
    set_once(&mut raw.flatten, meta.path.span(), meta)
}

pub(super) fn parse_command(field: &Field) -> syn::Result<RawCommand> {
    let mut raw = RawCommand::default();
    let mut errors = Errors::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("command") {
            continue;
        }
        if let Meta::Path(_) = attr.meta {
            errors.push(syn::Error::new_spanned(
                attr,
                "expected `#[command(...)]`, e.g. `#[command(flatten)]`",
            ));
            continue;
        }
        let result =
            attr.parse_nested_meta(|meta| dispatch(meta, "command", FIELD_COMMAND_KEYS, &mut raw));
        if let Err(error) = result {
            errors.push(error);
        }
    }

    errors.finish(raw)
}
