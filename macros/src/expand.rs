mod args_trait;
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

    let body = quote! {
        let __context = getargs_derive::__private::Context::new(__raw);
        let mut __state = #args::init();

        while let Some(argument) = options.next_arg().map_err(|error| match error {
            getargs::Error::DoesNotRequireValue(getargs::Opt::Short(short)) => {
                getargs_derive::Error::UnexpectedValue(getargs_derive::Opt::Short(short))
            }
            getargs::Error::DoesNotRequireValue(getargs::Opt::Long(long)) => {
                getargs_derive::Error::UnexpectedValue(getargs_derive::Opt::Long(long))
            }
            _ => getargs_derive::Error::InvalidArguments,
        })? {
            #reserved_match
            if !#args::try_arg(&mut __state, argument, options, __context)? {
                return Err(match argument {
                    getargs::Arg::Positional(value) => {
                        getargs_derive::Error::UnexpectedArgument(value)
                    }
                    getargs::Arg::Short(short) => getargs_derive::Error::UnknownOption {
                        option: getargs_derive::Opt::Short(short),
                        suggestion: core::option::Option::None,
                    },
                    getargs::Arg::Long(long) => getargs_derive::Error::UnknownOption {
                        option: getargs_derive::Opt::Long(long),
                        suggestion: getargs_derive::__suggest(
                            long,
                            &[#args::LONGS, &[#(#reserved_longs),*]],
                        ),
                    },
                });
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
