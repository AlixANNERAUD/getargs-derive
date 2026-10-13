use super::field::{FieldTokens, args_trait, subcommand_trait};
use super::flatten::{Merged, static_args, static_commands};
use super::relations::relation_checks;
use crate::help::{Mode, command_name, entry_description, entry_label, usage_label};
use crate::model::{FieldKind, FieldModel, Model, Slot};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, GenericParam, Lifetime, LifetimeParam, parse_quote};

fn help_entry(field: &FieldModel) -> TokenStream {
    let label = entry_label(field);
    let positional = matches!(field.kind, FieldKind::Positional);
    let usage = if positional {
        usage_label(field)
    } else {
        String::new()
    };
    let hidden = field.hide;
    let global = field.parsing.global;
    let heading = field.help_heading.as_deref().unwrap_or_default();
    let ordered = field.display_order.is_some();
    let order = field.display_order.unwrap_or_default();
    let (short, short_wraps) = entry_description(field, Mode::Short);
    let (long, long_wraps) = entry_description(field, Mode::Long);
    quote! {
        getargs_derive::__private::HelpEntry {
            label: #label,
            usage: #usage,
            positional: #positional,
            hidden: #hidden,
            global: #global,
            heading: #heading,
            order: #order,
            ordered: #ordered,
            short: #short,
            short_wraps: #short_wraps,
            long: #long,
            long_wraps: #long_wraps,
        }
    }
}

fn any_flattened(model: &Model, own: bool, constant: &str) -> TokenStream {
    let constant = syn::Ident::new(constant, proc_macro2::Span::call_site());
    let flattened = model.flattened.iter().map(|(_, flatten)| {
        let args = static_args(&flatten.ty);
        quote! { #args::#constant }
    });
    quote! { #own #(|| #flattened)* }
}

fn constants(model: &Model) -> TokenStream {
    let entries = Merged {
        element: quote! { getargs_derive::__private::HelpEntry },
        fill: quote! { getargs_derive::__private::HelpEntry::EMPTY },
        constant: "ENTRIES",
    }
    .tokens(model, |field| Some(vec![help_entry(field)]));
    let shorts = Merged {
        element: quote! { char },
        fill: quote! { '\0' },
        constant: "SHORTS",
    }
    .tokens(model, |field| {
        let shorts: Vec<TokenStream> = field
            .short
            .into_iter()
            .chain(field.short_aliases.iter().map(|alias| alias.name))
            .map(|short| quote! { #short })
            .collect();
        Some(shorts)
    });
    let longs = Merged {
        element: quote! { &'static str },
        fill: quote! { "" },
        constant: "LONGS",
    }
    .tokens(model, |field| {
        let longs: Vec<TokenStream> = field
            .long
            .iter()
            .chain(field.long_aliases.iter().map(|alias| &alias.name))
            .map(|long| quote! { #long })
            .collect();
        Some(longs)
    });
    let positionals = Merged {
        element: quote! { bool },
        fill: quote! { false },
        constant: "POSITIONALS",
    }
    .tokens(model, |field| {
        let optional = field.is_optional();
        matches!(field.kind, FieldKind::Positional).then(|| vec![quote! { #optional }])
    });
    let last = any_flattened(
        model,
        model.fields.iter().any(|field| field.parsing.last),
        "LAST",
    );
    let mut env = if model.fields.iter().any(|field| field.env.is_some()) {
        quote! { getargs_derive::__private::UsesEnv }
    } else {
        quote! { getargs_derive::__private::NoEnv }
    };
    for (_, flatten) in &model.flattened {
        let args = args_trait(&flatten.ty);
        env = quote! { <#env as getargs_derive::__private::EnvFlag>::Or<#args::Env> };
    }
    if let Some(subcommand) = &model.subcommand {
        let commands = subcommand_trait(&subcommand.ty);
        env = quote! { <#env as getargs_derive::__private::EnvFlag>::Or<#commands::Env> };
    }
    let has_long_help = any_flattened(
        model,
        model
            .fields
            .iter()
            .any(|field| !field.hide && field.long_help.is_some()),
        "HAS_LONG_HELP",
    );
    let command = command_help(model);

    quote! {
        const ENTRIES: &'static [getargs_derive::__private::HelpEntry] = #entries;
        const SHORTS: &'static [char] = #shorts;
        const LONGS: &'static [&'static str] = #longs;
        const POSITIONALS: &'static [bool] = #positionals;
        const LAST: bool = #last;
        type Env = #env;
        const HAS_LONG_HELP: bool = #has_long_help;
        #command
    }
}

fn command_help(model: &Model) -> TokenStream {
    let container = &model.container;
    let name = command_name(container);
    let text = |text: &Option<String>| text.clone().unwrap_or_default();
    let before_help = text(&container.before_help);
    let about = text(&container.about);
    let long_about = text(&container.long_about);
    let after_help = text(&container.after_help);
    let term_width = container.max_term_width.unwrap_or_default();
    let help_flag = !container.disable_help_flag;
    let version = container.version.is_some();
    let (commands, infos, required) = match &model.subcommand {
        Some(subcommand) => {
            let commands = static_commands(&subcommand.ty);
            let required = !subcommand.optional;
            (
                quote! { #commands::COMMANDS },
                quote! { #commands::INFOS },
                required,
            )
        }
        None => (quote! { &[] }, quote! { &[] }, false),
    };
    quote! {
        const COMMAND: getargs_derive::__private::CommandHelp = getargs_derive::__private::CommandHelp {
            name: #name,
            before_help: #before_help,
            about: #about,
            long_about: #long_about,
            after_help: #after_help,
            term_width: #term_width,
            help_flag: #help_flag,
            version: #version,
            entries: Self::ENTRIES,
            has_long_help: Self::HAS_LONG_HELP,
            shorts: Self::SHORTS,
            longs: Self::LONGS,
            commands: #commands,
            subcommand_required: #required,
            infos: #infos,
        };
    }
}

pub(super) fn args_impl(input: &DeriveInput, model: &Model) -> TokenStream {
    let name = &input.ident;
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let lifetime: Lifetime = parse_quote! { '__getargs };
    let mut generics = input.generics.clone();
    let outlives: Vec<Lifetime> = generics
        .lifetimes()
        .map(|parameter| parameter.lifetime.clone())
        .collect();
    generics.params.insert(
        0,
        GenericParam::Lifetime(LifetimeParam::new(lifetime.clone())),
    );
    let where_clause = generics.make_where_clause();
    for outlived in outlives {
        where_clause
            .predicates
            .push(parse_quote! { #lifetime: #outlived });
    }
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let mut tokens = FieldTokens::default();
    for slot in model.slots() {
        tokens.push(&slot, model.container.allow_negative_numbers);
    }
    if let Some(subcommand) = &model.subcommand {
        tokens.push_subcommand(subcommand);
    }
    let FieldTokens {
        state_types,
        initializers,
        value_idents,
        prologue,
        option_match_arms,
        steps,
        filled,
        finalizers,
        field_builds,
        global_arms,
        global_steps,
        subcommand: subcommand_index,
    } = tokens;

    let option_match = if option_match_arms.is_empty() {
        quote! {}
    } else {
        quote! {
            match __argument {
                #(#option_match_arms)*
                _ => {}
            }
        }
    };
    let global_match = if global_arms.is_empty() {
        quote! {}
    } else {
        quote! {
            match __argument {
                #(#global_arms)*
                _ => {}
            }
        }
    };
    let (subcommand_step, subcommand_fallback) = match (&model.subcommand, subcommand_index) {
        (Some(subcommand), Some(index)) => {
            let commands = subcommand_trait(&subcommand.ty);
            (
                quote! {
                    if let getargs::Arg::Positional(__name) = __argument {
                        if !__options.opts_ended() {
                            let __found = {
                                let mut __forward = |
                                    __argument: getargs::Arg<&#lifetime str>,
                                    __options: &mut getargs::Options<&#lifetime str, __I>,
                                | -> core::result::Result<bool, getargs_derive::Error<#lifetime>> {
                                    if <Self as getargs_derive::__private::Args<#lifetime>>::try_global(
                                        &mut *__state,
                                        __argument,
                                        __options,
                                        __context,
                                    )? {
                                        return core::result::Result::Ok(true);
                                    }
                                    __globals(__argument, __options)
                                };
                                #commands::try_command(__name, __options, __context, &mut __forward)?
                            };
                            if let core::option::Option::Some(__found) = __found {
                                __state.#index = core::option::Option::Some(__found);
                                return core::result::Result::Ok(true);
                            }
                        }
                    }
                },
                quote! {
                    if let getargs::Arg::Positional(__name) = __argument {
                        if !__options.opts_ended() {
                            return Err(getargs_derive::Error::UnknownSubcommand {
                                name: __name,
                                suggestion: getargs_derive::__suggest(__name, &[#commands::NAMES]),
                            });
                        }
                    }
                },
            )
        }
        _ => (quote! {}, quote! {}),
    };
    let (init, destructure) = if value_idents.is_empty() {
        (quote! {}, quote! { let _ = __state; })
    } else {
        (
            quote! { (#(#initializers,)*) },
            quote! { let (#(#value_idents,)*) = __state; },
        )
    };
    let filled = if filled.is_empty() {
        quote! { let _ = __state; true }
    } else {
        quote! { #(#filled)&&* }
    };
    let relation_checks = relation_checks(&model.container, &model.fields);
    let constants = constants(model);
    let command_constants = super::command::constants(input, model);
    let uses_last = model.slots().iter().any(
        |slot| !matches!(slot, Slot::Arg(field) if !matches!(field.kind, FieldKind::Positional)),
    );
    let last = if uses_last {
        quote! {
            let __last = __context.last
                || <Self as getargs_derive::__private::Args<#lifetime>>::LAST;
        }
    } else {
        quote! {}
    };

    quote! {
        impl #impl_generics getargs_derive::__private::Args<#lifetime> for #name #ty_generics #where_clause {
            type State = (#(#state_types,)*);

            #constants

            #command_constants

            fn init() -> Self::State {
                #init
            }

            fn filled(__state: &Self::State) -> bool {
                #filled
            }

            fn try_arg<__I: Iterator<Item = &#lifetime str>>(
                __state: &mut Self::State,
                __argument: getargs::Arg<&#lifetime str>,
                __options: &mut getargs::Options<&#lifetime str, __I>,
                __context: getargs_derive::__private::Context<#lifetime, '_>,
                __globals: &mut getargs_derive::__private::Globals<'_, #lifetime, __I>,
            ) -> core::result::Result<bool, getargs_derive::Error<#lifetime>> {
                #last
                #(#prologue)*
                #option_match
                #subcommand_step
                #(#steps)*
                #subcommand_fallback
                core::result::Result::Ok(false)
            }

            fn try_global<__I: Iterator<Item = &#lifetime str>>(
                __state: &mut Self::State,
                __argument: getargs::Arg<&#lifetime str>,
                __options: &mut getargs::Options<&#lifetime str, __I>,
                __context: getargs_derive::__private::Context<#lifetime, '_>,
            ) -> core::result::Result<bool, getargs_derive::Error<#lifetime>> {
                #global_match
                #(#global_steps)*
                core::result::Result::Ok(false)
            }

            fn finish<__E: FnMut(&str) -> core::option::Option<&#lifetime str>>(
                __state: Self::State,
                __getargs_env: &mut __E,
            ) -> core::result::Result<Self, getargs_derive::Error<#lifetime>> {
                #destructure

                #relation_checks

                #(#finalizers)*

                core::result::Result::Ok(Self {
                    #(#field_builds),*
                })
            }
        }
    }
}
