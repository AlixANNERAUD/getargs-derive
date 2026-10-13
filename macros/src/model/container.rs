mod group;
mod help;
mod help_text;
mod parsing;
mod text;
mod version;

use super::attr::{Key, dispatch};
use super::doc::{long_doc, short_doc};
use super::errors::Errors;
pub(crate) use group::GroupDecl;
use syn::{DeriveInput, Meta};

const COMMAND_KEYS: &[Key<RawContainer>] = &[
    Key::new("about", text::about),
    Key::new("after_help", help_text::after_help),
    Key::new("allow_negative_numbers", parsing::allow_negative_numbers),
    Key::new("before_help", help_text::before_help),
    Key::new("disable_help_flag", help::disable_help_flag),
    Key::new("group", group::group),
    Key::new("long_about", help_text::long_about),
    Key::new("max_term_width", help_text::max_term_width),
    Key::new("name", text::name),
    Key::new("version", version::version),
];

pub(crate) struct ContainerModel {
    pub(crate) name: Option<String>,
    pub(crate) about: Option<String>,
    pub(crate) version: Option<Version>,
    pub(crate) disable_help_flag: bool,
    pub(crate) groups: Vec<GroupDecl>,
    pub(crate) long_about: Option<String>,
    pub(crate) before_help: Option<String>,
    pub(crate) after_help: Option<String>,
    pub(crate) max_term_width: Option<usize>,
    pub(crate) allow_negative_numbers: bool,
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
    groups: Vec<GroupDecl>,
    long_about: Option<String>,
    before_help: Option<String>,
    after_help: Option<String>,
    max_term_width: Option<usize>,
    allow_negative_numbers: Option<bool>,
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
        groups: raw.groups,
        long_about: raw.long_about.or_else(|| long_doc(&input.attrs)),
        before_help: raw.before_help,
        after_help: raw.after_help,
        max_term_width: raw.max_term_width,
        allow_negative_numbers: raw.allow_negative_numbers.unwrap_or(false),
    })
}
