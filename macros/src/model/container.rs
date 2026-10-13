use super::attr::{parse_string, reject_value, set_once};
use super::doc::short_doc;
use super::errors::Errors;
use quote::ToTokens;
use syn::{DeriveInput, Ident, Meta, Token, meta::ParseNestedMeta};

const SUPPORTED_COMMAND_KEYS: &str = "`name`, `about`, `version`, `disable_help_flag`";

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

fn parse_command_entry(
    meta: ParseNestedMeta,
    name: &mut Option<String>,
    about: &mut Option<String>,
    version: &mut Option<Version>,
    disable_help_flag: &mut Option<()>,
) -> syn::Result<()> {
    let Some(key) = meta.path.get_ident().map(Ident::to_string) else {
        let path = meta.path.to_token_stream().to_string().replace(' ', "");
        return Err(meta.error(format!(
            "unknown `command` attribute `{path}`; supported attributes are: {SUPPORTED_COMMAND_KEYS}"
        )));
    };

    match key.as_str() {
        "name" => {
            let literal = parse_string(&meta, "name", "name = \"tool\"")?;
            let value = literal.value();
            if value.is_empty() {
                return Err(syn::Error::new(literal.span(), "`name` cannot be empty"));
            }
            set_once(name, value, &meta)
        }
        "about" => {
            let literal = parse_string(&meta, "about", "about = \"Does things\"")?;
            set_once(about, literal.value(), &meta)
        }
        "version" => {
            let value = if meta.input.peek(Token![=]) {
                let literal = parse_string(&meta, "version", "version = \"1.0.0\"")?;
                let value = literal.value();
                if value.trim().is_empty() {
                    return Err(syn::Error::new(literal.span(), "`version` cannot be empty"));
                }
                Version::Literal(value)
            } else {
                reject_value(&meta, "version")?;
                Version::Package
            };
            set_once(version, value, &meta)
        }
        "disable_help_flag" => {
            reject_value(&meta, "disable_help_flag")?;
            set_once(disable_help_flag, (), &meta)
        }
        _ => Err(meta.error(format!(
            "unknown `command` attribute `{key}`; supported attributes are: {SUPPORTED_COMMAND_KEYS}"
        ))),
    }
}

pub(super) fn parse_container(input: &DeriveInput) -> syn::Result<ContainerModel> {
    let mut name = None;
    let mut about = None;
    let mut version = None;
    let mut disable_help_flag = None;
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
        let result = attr.parse_nested_meta(|meta| {
            parse_command_entry(
                meta,
                &mut name,
                &mut about,
                &mut version,
                &mut disable_help_flag,
            )
        });
        if let Err(error) = result {
            errors.push(error);
        }
    }

    errors.finish(ContainerModel {
        name,
        about: about.or_else(|| short_doc(&input.attrs)),
        version,
        disable_help_flag: disable_help_flag.is_some(),
    })
}
