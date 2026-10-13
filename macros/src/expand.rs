mod args_trait;
mod bytes;
mod collection;
mod command;
mod env;
mod field;
mod flatten;
mod help;
mod parsing;
mod relations;
mod validation;
mod version;

use crate::model::{ContainerModel, Model};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, GenericParam};

pub(crate) use flatten::static_args;

pub(crate) fn variant_args(input: &DeriveInput, model: &Model) -> TokenStream {
    let args_impl = args_trait::args_impl(input, model);
    let checks = flatten::checks(model);
    quote! {
        #args_impl
        #checks
    }
}

fn reserved_longs(container: &ContainerModel) -> Vec<&'static str> {
    let mut long_names = Vec::new();
    if !container.disable_help_flag {
        long_names.push("help");
    }
    if container.version.is_some() {
        long_names.push("version");
    }
    long_names
}

pub(crate) fn expand(input: &DeriveInput, model: &Model) -> TokenStream {
    let container = &model.container;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let lifetime_params: Vec<_> = input
        .generics
        .params
        .iter()
        .filter_map(|parameter| match parameter {
            GenericParam::Lifetime(lifetime) => Some(lifetime.lifetime.clone()),
            _ => None,
        })
        .collect();

    let args = field::args_trait(&syn::parse_quote! { Self });
    let reserved_longs = reserved_longs(container);
    let help_items = help::help_items(input, model);
    let help_handler = help::help_handler(container);
    let (version_items, version_handler) = version::version_tokens(container);
    let reserved_match = if help_handler.is_empty() && version_handler.is_empty() {
        quote! {}
    } else {
        quote! {
            match argument {
                #help_handler
                #version_handler
                _ => {}
            }
        }
    };
    let args_impl = args_trait::args_impl(input, model);
    let flatten_checks = flatten::checks(model);
    let command_impl = command::command_impl(input, model);

    let context = if model.subcommand.is_some() {
        quote! { getargs_derive::__private::Context::new(__raw).with_helps(Self::__HELPS) }
    } else {
        quote! { getargs_derive::__private::Context::new(__raw) }
    };
    let body = quote! {
        let __context = #context;
        let mut __state = #args::init();

        while let Some(argument) = getargs_derive::__private::next_arg(options)? {
            #reserved_match
            if !#args::try_arg(
                &mut __state,
                argument,
                options,
                __context,
                &mut getargs_derive::__private::no_globals,
            )? {
                return Err(getargs_derive::__private::unexpected(
                    argument,
                    &[#args::LONGS, &[#(#reserved_longs),*]],
                ));
            }
        }

        #args::finish(__state, &mut __getargs_env)
    };

    let parse_methods = env::parse_methods(model, &lifetime_params, body);

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #help_items

            #version_items

            #parse_methods
        }

        #args_impl

        #command_impl

        #flatten_checks
    }
}
