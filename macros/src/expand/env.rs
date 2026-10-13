use super::bytes::{BYTES, Flavor, STR, os_args_method};
use super::validation::convert;
use crate::model::{FieldModel, Model};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

fn has_env(fields: &[FieldModel]) -> bool {
    fields.iter().any(|field| field.env.is_some())
}

pub(super) fn env_lookup(field: &FieldModel) -> Option<TokenStream> {
    let env = field.env.as_deref()?;
    let value_ident = format_ident!("__value_{}", field.name);
    let error_name = format!("{} [env: {env}]", field.error_name());
    let parsed = convert(field, quote! { value }, &error_name);
    Some(quote! {
        let #value_ident = match #value_ident {
            core::option::Option::Some(value) => core::option::Option::Some(value),
            core::option::Option::None => match __getargs_env(#env) {
                core::option::Option::Some(value) => core::option::Option::Some(#parsed),
                core::option::Option::None => core::option::Option::None,
            },
        };
    })
}

fn hidden_parse(lifetime_params: &[syn::Lifetime], body: TokenStream) -> TokenStream {
    quote! {
        #[doc(hidden)]
        pub fn __parse<'__getargs, __A, I, E>(
            options: &mut getargs::Options<__A, I>,
            __raw: core::option::Option<&core::cell::Cell<core::option::Option<&'__getargs [u8]>>>,
            mut __getargs_env: E,
        ) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
        where
            __A: getargs_derive::__private::Input<'__getargs>,
            I: Iterator<Item = __A>,
            E: FnMut(&str) -> core::option::Option<&'__getargs str>,
            #('__getargs: #lifetime_params,)*
        {
            #body
        }
    }
}

fn with_env_methods(flavor: &Flavor, lifetime_params: &[syn::Lifetime]) -> TokenStream {
    let input = flavor.input();
    let parse_with_env = format_ident!("{}", flavor.parse_with_env);
    let from_args_with_env = format_ident!("{}", flavor.from_args_with_env);
    let parse_doc = flavor.doc("Parse CLI arguments from an iterator, using `getargs::Options`, reading environment\nvariable fallbacks with `env`.\n\n`env` is called with the name of an environment variable and returns its value, or\n`None` when it is not set. It is only called for arguments absent from the command line.");
    let from_doc = flavor.doc(&format!(
        "Parse CLI arguments directly from an iterator, reading environment variable fallbacks\nwith `env`.\n\nConvenience wrapper that creates a `getargs::Options` internally\nand calls [`{0}`](Self::{0}).",
        flavor.parse_with_env
    ));
    quote! {
        #parse_doc
        pub fn #parse_with_env<'__getargs, I, E>(options: &mut getargs::Options<#input, I>, env: E) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
        where
            I: Iterator<Item = #input>,
            E: FnMut(&str) -> core::option::Option<&'__getargs str>,
            #('__getargs: #lifetime_params,)*
        {
            Self::__parse(options, core::option::Option::None, env)
        }

        #from_doc
        pub fn #from_args_with_env<'__getargs, I, E>(iter: I, env: E) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
        where
            I: Iterator<Item = #input>,
            E: FnMut(&str) -> core::option::Option<&'__getargs str>,
            #('__getargs: #lifetime_params,)*
        {
            let raw = core::cell::Cell::new(core::option::Option::None);
            let mut options = getargs::Options::new(getargs_derive::__RawArgs::new(iter, &raw));
            Self::__parse(&mut options, core::option::Option::Some(&raw), env)
        }
    }
}

fn own_env_methods(flavor: &Flavor, lifetime_params: &[syn::Lifetime]) -> TokenStream {
    let input = flavor.input();
    let parse = format_ident!("{}", flavor.parse);
    let from_args = format_ident!("{}", flavor.from_args);
    let with_env = with_env_methods(flavor, lifetime_params);
    let parse_doc = flavor.doc(&format!(
        "Parse CLI arguments from an iterator, using `getargs::Options`, reading environment\nvariable fallbacks from the process environment.\n\nThis requires the `std` feature of `getargs-derive`. Without it, use\n[`{0}`](Self::{0}) with your own lookup function.\nPrefer [`{1}`](Self::{1}) for convenience.",
        flavor.parse_with_env, flavor.from_args
    ));
    let from_doc = flavor.doc(&format!(
        "Parse CLI arguments directly from an iterator, reading environment variable fallbacks\nfrom the process environment.\n\nConvenience wrapper that creates a `getargs::Options` internally\nand calls [`{0}`](Self::{0}). This requires the `std` feature of `getargs-derive`.\nWithout it, use [`{1}`](Self::{1}).",
        flavor.parse, flavor.from_args_with_env
    ));
    quote! {
        #parse_doc
        pub fn #parse<'__getargs, I>(options: &mut getargs::Options<#input, I>) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
        where
            I: Iterator<Item = #input> + getargs_derive::__EnvSupport,
            #('__getargs: #lifetime_params,)*
        {
            Self::__parse(options, core::option::Option::None, getargs_derive::__env)
        }

        #with_env

        #from_doc
        pub fn #from_args<'__getargs, I>(iter: I) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
        where
            I: Iterator<Item = #input> + getargs_derive::__EnvSupport,
            #('__getargs: #lifetime_params,)*
        {
            let raw = core::cell::Cell::new(core::option::Option::None);
            let mut options = getargs::Options::new(getargs_derive::__RawArgs::new(iter, &raw));
            Self::__parse(&mut options, core::option::Option::Some(&raw), getargs_derive::__env)
        }
    }
}

fn plain_methods(flavor: &Flavor, model: &Model, lifetime_params: &[syn::Lifetime]) -> TokenStream {
    let input = flavor.input();
    let parse = format_ident!("{}", flavor.parse);
    let from_args = format_ident!("{}", flavor.from_args);
    let (lookup, bound, with_env) = if model.flattened.is_empty() && model.subcommand.is_none() {
        (
            quote! { getargs_derive::__private::no_env },
            quote! {},
            quote! {},
        )
    } else {
        (
            quote! { getargs_derive::__env },
            quote! {
                + getargs_derive::__private::EnvSupport<
                    <Self as getargs_derive::__private::Args<'__getargs>>::Env,
                >
            },
            with_env_methods(flavor, lifetime_params),
        )
    };
    let parse_doc = flavor.doc(&format!(
        "Parse CLI arguments from an iterator, using `getargs::Options`.\n\nThis is the low-level method. Prefer [`{0}`](Self::{0}) for convenience.",
        flavor.from_args
    ));
    let from_doc = flavor.doc(&format!(
        "Parse CLI arguments directly from an iterator.\n\nConvenience wrapper that creates a `getargs::Options` internally\nand calls [`{0}`](Self::{0}).{1}",
        flavor.parse, flavor.example
    ));
    quote! {
        #parse_doc
        pub fn #parse<'__getargs, I>(options: &mut getargs::Options<#input, I>) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
        where
            I: Iterator<Item = #input> #bound,
            #('__getargs: #lifetime_params,)*
        {
            Self::__parse(options, core::option::Option::None, #lookup)
        }

        #with_env

        #from_doc
        pub fn #from_args<'__getargs, I>(iter: I) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
        where
            I: Iterator<Item = #input> #bound,
            #('__getargs: #lifetime_params,)*
        {
            let raw = core::cell::Cell::new(core::option::Option::None);
            let mut options = getargs::Options::new(getargs_derive::__RawArgs::new(iter, &raw));
            Self::__parse(&mut options, core::option::Option::Some(&raw), #lookup)
        }
    }
}

pub(super) fn parse_methods(
    model: &Model,
    lifetime_params: &[syn::Lifetime],
    body: TokenStream,
) -> TokenStream {
    let hidden = hidden_parse(lifetime_params, body);
    let flavors = [&STR, &BYTES];
    let methods = flavors.map(|flavor| {
        if has_env(&model.fields) {
            own_env_methods(flavor, lifetime_params)
        } else {
            plain_methods(flavor, model, lifetime_params)
        }
    });
    let os_args = os_args_method();
    quote! {
        #(#methods)*

        #os_args

        #hidden
    }
}
