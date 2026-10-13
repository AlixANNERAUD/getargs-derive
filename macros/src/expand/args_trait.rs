use super::field::{FieldTokens, args_trait};
use super::flatten::{Merged, static_args};
use super::relations::relation_checks;
use crate::help::{Mode, entry_description, entry_label, usage_label};
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
    let has_long_help = any_flattened(
        model,
        model
            .fields
            .iter()
            .any(|field| !field.hide && field.long_help.is_some()),
        "HAS_LONG_HELP",
    );

    quote! {
        const ENTRIES: &'static [getargs_derive::__private::HelpEntry] = #entries;
        const SHORTS: &'static [char] = #shorts;
        const LONGS: &'static [&'static str] = #longs;
        const POSITIONALS: &'static [bool] = #positionals;
        const LAST: bool = #last;
        type Env = #env;
        const HAS_LONG_HELP: bool = #has_long_help;
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
            ) -> core::result::Result<bool, getargs_derive::Error<#lifetime>> {
                #last
                #(#prologue)*
                #option_match
                #(#steps)*
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
