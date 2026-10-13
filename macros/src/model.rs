mod attr;
mod container;
mod doc;
mod errors;
mod field;
mod types;
mod validate;

pub(crate) use container::{ContainerModel, Version};
pub(crate) use field::{DefaultValue, FieldKind, FieldModel, FlagAction};

use errors::Errors;
use syn::{Data, DeriveInput, Fields};

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
    let container = match container::parse_container(input) {
        Ok(container) => Some(container),
        Err(error) => {
            errors.push(error);
            None
        }
    };
    let mut models = Vec::new();

    for field in fields {
        match field::build_field(field) {
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
    validate::validate_struct(&models, help_enabled, version_enabled, &mut errors);
    match container {
        Some(container) => errors.finish((container, models)),
        None => Err(errors.0.expect("container errors are recorded")),
    }
}
