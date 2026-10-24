use crate::model::errors::Errors;
use crate::model::{
    self, Alias, ContainerModel, Keys, Model, parse_container, validate_command_name,
};
use crate::value_enum::to_kebab_case;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident};
use std::collections::{HashMap, HashSet};
use syn::{
    Data, DataStruct, DeriveInput, Fields, FieldsNamed, GenericParam, Generics, Ident, Type,
    Variant as SynVariant, ext::IdentExt, punctuated::Punctuated, token,
};

pub(crate) enum Kind {
    Unit,
    Named(Vec<Ident>),
    Newtype(Box<Type>),
}

pub(crate) struct Variant {
    pub(crate) ident: Ident,
    pub(crate) name: String,
    pub(crate) aliases: Vec<Alias<String>>,
    pub(crate) hidden: bool,
    pub(crate) about: Option<(String, String)>,
    pub(crate) kind: Kind,
    pub(crate) args: Option<(DeriveInput, Model)>,
}

impl Variant {
    pub(crate) fn visible_aliases(&self) -> Vec<&str> {
        self.aliases
            .iter()
            .filter(|alias| alias.visible)
            .map(|alias| alias.name.as_str())
            .collect()
    }
}

struct Command {
    name: Option<String>,
    aliases: Vec<Alias<String>>,
    hidden: bool,
    about: Option<(String, String)>,
}

impl Command {
    fn take(container: &mut ContainerModel) -> Self {
        Command {
            name: container.name.clone(),
            aliases: std::mem::take(&mut container.aliases),
            hidden: container.hide,
            about: container
                .about
                .clone()
                .map(|about| (about, container.long_about.clone().unwrap_or_default())),
        }
    }
}

fn lifetimes_in(tokens: TokenStream, found: &mut HashSet<String>) {
    let mut tokens = tokens.into_iter().peekable();
    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Punct(punct) if punct.as_char() == '\'' => {
                if let Some(TokenTree::Ident(ident)) = tokens.peek() {
                    found.insert(ident.to_string());
                }
            }
            TokenTree::Group(group) => lifetimes_in(group.stream(), found),
            _ => {}
        }
    }
}

fn variant_generics(generics: &Generics, fields: &Fields) -> Generics {
    let mut used = HashSet::new();
    for field in fields {
        lifetimes_in(field.ty.to_token_stream(), &mut used);
    }
    let mut output = Generics::default();
    for parameter in &generics.params {
        if let GenericParam::Lifetime(lifetime) = parameter
            && used.contains(&lifetime.lifetime.ident.to_string())
        {
            let mut lifetime = lifetime.clone();
            lifetime.bounds = lifetime
                .bounds
                .into_iter()
                .filter(|bound| used.contains(&bound.ident.to_string()))
                .collect();
            output.params.push(GenericParam::Lifetime(lifetime));
        }
    }
    output
}

fn args_input(enum_input: &DeriveInput, variant: &SynVariant) -> DeriveInput {
    let fields = match &variant.fields {
        Fields::Named(fields) => fields.clone(),
        _ => FieldsNamed {
            brace_token: token::Brace::default(),
            named: Punctuated::new(),
        },
    };
    let fields = Fields::Named(fields);
    DeriveInput {
        attrs: variant.attrs.clone(),
        vis: enum_input.vis.clone(),
        ident: format_ident!(
            "__GetArgs{}{}",
            enum_input.ident.unraw(),
            variant.ident.unraw()
        ),
        generics: variant_generics(&enum_input.generics, &fields),
        data: Data::Struct(DataStruct {
            struct_token: token::Struct::default(),
            fields,
            semi_token: None,
        }),
    }
}

fn build_variant(enum_input: &DeriveInput, variant: &SynVariant) -> syn::Result<Variant> {
    let default_name = to_kebab_case(&variant.ident.unraw().to_string());
    let (kind, command, args) = match &variant.fields {
        Fields::Unit | Fields::Named(_) => {
            let input = args_input(enum_input, variant);
            let mut model = model::parse(&input, Keys::Variant)?;
            let kind = match &variant.fields {
                Fields::Named(fields) => Kind::Named(
                    fields
                        .named
                        .iter()
                        .filter_map(|field| field.ident.clone())
                        .collect(),
                ),
                _ => Kind::Unit,
            };
            let command = Command::take(&mut model.container);
            model.container.name = Some(command.name.clone().unwrap_or(default_name.clone()));
            (kind, command, Some((input, model)))
        }
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let field = &fields.unnamed[0];
            let mut errors = Errors::default();
            for attr in &field.attrs {
                if attr.path().is_ident("arg") || attr.path().is_ident("command") {
                    errors.push(syn::Error::new_spanned(
                        attr,
                        "attributes are not supported on the field of a newtype subcommand variant; put `#[command(...)]` on the variant and `#[arg(...)]` on the fields of the wrapped struct",
                    ));
                }
            }
            let container = parse_container(&variant.attrs, Keys::Newtype);
            let mut container = match container {
                Ok(container) => errors.finish(container)?,
                Err(error) => {
                    errors.push(error);
                    return Err(errors.0.expect("errors are recorded"));
                }
            };
            let command = Command::take(&mut container);
            (Kind::Newtype(Box::new(field.ty.clone())), command, None)
        }
        Fields::Unnamed(fields) => {
            return Err(syn::Error::new_spanned(
                fields,
                format!(
                    "subcommand variant `{}` must be a unit variant, a variant with named fields, or a newtype variant wrapping a struct deriving `GetArgs`",
                    variant.ident
                ),
            ));
        }
    };

    let name = command.name.unwrap_or(default_name);
    validate_command_name("name", &name, variant.ident.span())?;
    Ok(Variant {
        ident: variant.ident.clone(),
        name,
        aliases: command.aliases,
        hidden: command.hidden,
        about: command.about,
        kind,
        args,
    })
}

fn names(variant: &Variant) -> Vec<(String, Span)> {
    std::iter::once((variant.name.clone(), variant.ident.span()))
        .chain(
            variant
                .aliases
                .iter()
                .map(|alias| (alias.name.clone(), alias.span)),
        )
        .collect()
}

pub(crate) fn parse(input: &DeriveInput) -> syn::Result<Vec<Variant>> {
    let data = match &input.data {
        Data::Enum(data) => data,
        Data::Struct(data) => {
            return Err(syn::Error::new_spanned(
                data.struct_token,
                "Subcommand can only be derived for enums; derive `GetArgs` for structs",
            ));
        }
        Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "Subcommand can only be derived for enums",
            ));
        }
    };

    let mut errors = Errors::default();
    if input
        .generics
        .params
        .iter()
        .any(|parameter| !matches!(parameter, GenericParam::Lifetime(_)))
    {
        errors.push(syn::Error::new_spanned(
            &input.generics,
            "Subcommand cannot be derived for enums with type or const parameters",
        ));
    }
    for attr in &input.attrs {
        if attr.path().is_ident("command") || attr.path().is_ident("arg") {
            errors.push(syn::Error::new_spanned(
                attr,
                "attributes are not supported on a `Subcommand` enum; put `#[command(...)]` on its variants",
            ));
        }
    }
    if data.variants.is_empty() {
        errors.push(syn::Error::new_spanned(
            &input.ident,
            "Subcommand requires at least one variant",
        ));
    }

    let mut variants = Vec::new();
    for variant in &data.variants {
        match build_variant(input, variant) {
            Ok(variant) => variants.push(variant),
            Err(error) => errors.push(error),
        }
    }

    let mut owners: HashMap<String, String> = HashMap::new();
    for variant in &variants {
        for (name, span) in names(variant) {
            match owners.get(&name) {
                Some(owner) => errors.push(syn::Error::new(
                    span,
                    format!("subcommand name `{name}` is already used by variant `{owner}`"),
                )),
                None => {
                    owners.insert(name, variant.ident.unraw().to_string());
                }
            }
        }
    }

    errors.finish(variants)
}
