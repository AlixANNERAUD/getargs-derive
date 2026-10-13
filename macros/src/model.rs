pub(crate) mod attr;
mod container;
pub(crate) mod doc;
pub(crate) mod errors;
mod field;
mod relations;
pub(crate) mod types;
mod validate;

pub(crate) use container::{ContainerModel, Keys, Version, parse_container, validate_command_name};
use field::Member;
pub(crate) use field::{
    Alias, CollectionModel, DefaultValue, FieldKind, FieldModel, FlagAction, FlattenModel,
    SubcommandModel, ValueParser,
};
pub(crate) use relations::resolve_groups;

use errors::Errors;
use syn::{Data, DeriveInput, Fields, GenericParam};

pub(crate) enum Slot<'a> {
    Arg(&'a FieldModel),
    Flatten(&'a FlattenModel),
}

pub(crate) struct Model {
    pub(crate) container: ContainerModel,
    pub(crate) fields: Vec<FieldModel>,
    pub(crate) flattened: Vec<(usize, FlattenModel)>,
    pub(crate) subcommand: Option<SubcommandModel>,
}

impl Model {
    pub(crate) fn slots(&self) -> Vec<Slot<'_>> {
        let mut slots = Vec::new();
        let mut flattened = self.flattened.iter().peekable();
        for (index, field) in self.fields.iter().enumerate() {
            while let Some((_, flatten)) = flattened.next_if(|(before, _)| *before == index) {
                slots.push(Slot::Flatten(flatten));
            }
            slots.push(Slot::Arg(field));
        }
        slots.extend(flattened.map(|(_, flatten)| Slot::Flatten(flatten)));
        slots
    }
}

pub(crate) fn parse(input: &DeriveInput, keys: Keys) -> syn::Result<Model> {
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
    let container = match container::parse_container(&input.attrs, keys) {
        Ok(container) => Some(container),
        Err(error) => {
            errors.push(error);
            None
        }
    };
    let mut models = Vec::new();
    let mut flattened = Vec::new();
    let mut subcommand: Option<SubcommandModel> = None;
    let mut built = true;

    for field in fields {
        match field::build_member(field) {
            Ok(Member::Arg(model)) => models.push(*model),
            Ok(Member::Flatten(model)) => flattened.push((models.len(), *model)),
            Ok(Member::Subcommand(model)) => match &subcommand {
                Some(first) => errors.push(syn::Error::new(
                    model.span,
                    format!(
                        "a struct can only have one `#[command(subcommand)]` field, and `{}` already is one",
                        first.name
                    ),
                )),
                None => subcommand = Some(*model),
            },
            Err(error) => {
                built = false;
                errors.push(error);
            }
        }
    }

    let generic = input
        .generics
        .params
        .iter()
        .any(|parameter| !matches!(parameter, GenericParam::Lifetime(_)));
    if generic {
        for (_, flatten) in &flattened {
            errors.push(syn::Error::new(
                flatten.span,
                "`#[command(flatten)]` is not supported in structs with type or const parameters",
            ));
        }
        if let Some(subcommand) = &subcommand {
            errors.push(syn::Error::new(
                subcommand.span,
                "`#[command(subcommand)]` is not supported in structs with type or const parameters",
            ));
        }
    }

    let help_enabled = container
        .as_ref()
        .is_some_and(|container| !container.disable_help_flag);
    let version_enabled = container
        .as_ref()
        .is_some_and(|container| container.version.is_some());
    if let Some(container) = &container {
        field::apply_container(&mut models, container.allow_negative_numbers);
    }
    field::validate_last(&models, &mut errors);
    field::validate_positionals(&models, &mut errors);
    validate::validate_struct(&models, help_enabled, version_enabled, &mut errors);
    let groups = container
        .as_ref()
        .map_or(&[][..], |container| container.groups.as_slice());
    if built {
        relations::validate_relations(groups, &models, &mut errors);
    }
    match container {
        Some(container) => errors.finish(Model {
            container,
            fields: models,
            flattened,
            subcommand,
        }),
        None => Err(errors.0.expect("container errors are recorded")),
    }
}
