use super::keys;
use super::names::to_kebab_case;
use crate::model::attr::{Key, dispatch};
use crate::model::errors::Errors;
use std::collections::HashMap;
use syn::{Attribute, Data, DeriveInput, Fields, Ident, LitStr, Meta, ext::IdentExt};

const VALUE_KEYS: &[Key<RawVariant>] =
    &[Key::new("alias", keys::alias), Key::new("name", keys::name)];

#[derive(Default)]
pub(super) struct RawVariant {
    pub(super) name: Option<LitStr>,
    pub(super) aliases: Vec<LitStr>,
}

pub(crate) struct Variant {
    pub(super) ident: Ident,
    pub(super) name: String,
    pub(super) aliases: Vec<String>,
}

fn parse_attributes(attrs: &[Attribute], errors: &mut Errors) -> RawVariant {
    let mut raw = RawVariant::default();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("value")) {
        if let Meta::Path(_) = attr.meta {
            errors.push(syn::Error::new_spanned(
                attr,
                "expected `#[value(...)]`, e.g. `#[value(name = \"fast\")]`",
            ));
            continue;
        }
        if let Err(error) =
            attr.parse_nested_meta(|meta| dispatch(meta, "value", VALUE_KEYS, &mut raw))
        {
            errors.push(error);
        }
    }
    raw
}

pub(crate) fn parse(input: &DeriveInput) -> syn::Result<Vec<Variant>> {
    let data = match &input.data {
        Data::Enum(data) => data,
        Data::Struct(data) => {
            return Err(syn::Error::new_spanned(
                data.struct_token,
                "ValueEnum can only be derived for enums with unit variants",
            ));
        }
        Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "ValueEnum can only be derived for enums with unit variants",
            ));
        }
    };

    let mut errors = Errors::default();
    if !input.generics.params.is_empty() {
        errors.push(syn::Error::new_spanned(
            &input.generics,
            "ValueEnum cannot be derived for generic enums",
        ));
    }
    if let Some(attr) = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("value"))
    {
        errors.push(syn::Error::new_spanned(
            attr,
            "`#[value(...)]` is only supported on variants",
        ));
    }
    if data.variants.is_empty() {
        errors.push(syn::Error::new_spanned(
            &input.ident,
            "ValueEnum requires at least one variant",
        ));
    }

    let mut variants = Vec::new();
    let mut owners: HashMap<String, String> = HashMap::new();
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            errors.push(syn::Error::new_spanned(
                &variant.fields,
                format!(
                    "ValueEnum variants cannot have fields; `{}` must be a unit variant",
                    variant.ident
                ),
            ));
        }
        let raw = parse_attributes(&variant.attrs, &mut errors);
        let ident_name = variant.ident.unraw().to_string();
        let name = match &raw.name {
            Some(literal) => (literal.value(), literal.span()),
            None => (to_kebab_case(&ident_name), variant.ident.span()),
        };
        let aliases = raw
            .aliases
            .iter()
            .map(|literal| (literal.value(), literal.span()));
        for (value, span) in std::iter::once(name.clone()).chain(aliases) {
            match owners.get(&value) {
                Some(owner) => errors.push(syn::Error::new(
                    span,
                    format!("value `{value}` is already used by variant `{owner}`"),
                )),
                None => {
                    owners.insert(value, ident_name.clone());
                }
            }
        }
        variants.push(Variant {
            ident: variant.ident.clone(),
            name: name.0,
            aliases: raw.aliases.iter().map(LitStr::value).collect(),
        });
    }

    errors.finish(variants)
}
