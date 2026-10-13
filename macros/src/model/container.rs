mod help;
mod text;
mod version;

use super::attr::{Key, dispatch};
use super::doc::short_doc;
use super::errors::Errors;
use syn::{DeriveInput, Meta};

const COMMAND_KEYS: &[Key<RawContainer>] = &[
    Key::new("about", text::about),
    Key::new("disable_help_flag", help::disable_help_flag),
    Key::new("name", text::name),
    Key::new("version", version::version),
];

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

#[derive(Default)]
struct RawContainer {
    name: Option<String>,
    about: Option<String>,
    version: Option<Version>,
    disable_help_flag: Option<()>,
}

pub(super) fn parse_container(input: &DeriveInput) -> syn::Result<ContainerModel> {
    let mut raw = RawContainer::default();
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
        let result =
            attr.parse_nested_meta(|meta| dispatch(meta, "command", COMMAND_KEYS, &mut raw));
        if let Err(error) = result {
            errors.push(error);
        }
    }

    errors.finish(ContainerModel {
        name: raw.name,
        about: raw.about.or_else(|| short_doc(&input.attrs)),
        version: raw.version,
        disable_help_flag: raw.disable_help_flag.is_some(),
    })
}
