use super::model::{Kind, Variant};
use crate::expand::{static_args, variant_args};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, GenericParam, Lifetime, LifetimeParam, Type, parse_quote};

fn hidden_struct(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let vis = &input.vis;
    let (impl_generics, _, _) = input.generics.split_for_impl();
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(|field| {
                    let ident = &field.ident;
                    let ty = &field.ty;
                    quote! { #vis #ident: #ty }
                })
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };
    let generics = impl_generics;
    quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #name #generics {
            #(#fields,)*
        }
    }
}

fn variant_type(variant: &Variant) -> Type {
    match (&variant.kind, &variant.args) {
        (Kind::Newtype(ty), _) => ty.clone(),
        (_, Some((input, _))) => {
            let name = &input.ident;
            let (_, ty_generics, _) = input.generics.split_for_impl();
            parse_quote! { #name #ty_generics }
        }
        _ => unreachable!("unit and named variants have a generated struct"),
    }
}

fn entry(variant: &Variant, ty: &Type) -> TokenStream {
    let args = static_args(ty);
    let name = &variant.name;
    let hidden = variant.hidden;
    let (about, long_about) = match (&variant.kind, &variant.about) {
        (Kind::Newtype(_), Some((about, long_about))) => {
            (quote! { #about }, quote! { #long_about })
        }
        _ => (
            quote! { #args::COMMAND.about },
            quote! { #args::COMMAND.long_about },
        ),
    };
    let visible = variant.visible_aliases();
    let description = if visible.is_empty() {
        about.clone()
    } else {
        let aliases = format!("[aliases: {}]", visible.join(", "));
        quote! {
            {
                const ABOUT: &str = #about;
                const PARTS: &[getargs_derive::__HelpPart] = &[
                    getargs_derive::__HelpPart::Text(ABOUT),
                    getargs_derive::__HelpPart::Text(
                        getargs_derive::__private::separator(ABOUT, #aliases),
                    ),
                    getargs_derive::__HelpPart::Text(#aliases),
                ];
                const LENGTH: usize = getargs_derive::__help_len(PARTS);
                const BYTES: [u8; LENGTH] = getargs_derive::__help_bytes::<LENGTH>(PARTS);
                getargs_derive::__help_str(&BYTES)
            }
        }
    };
    quote! {
        getargs_derive::__private::CommandEntry {
            name: #name,
            description: #description,
            about: #about,
            long_about: #long_about,
            hidden: #hidden,
            command: &#args::COMMAND,
        }
    }
}

fn info(variant: &Variant, ty: &Type) -> TokenStream {
    let args = static_args(ty);
    let name = &variant.name;
    let hidden = variant.hidden;
    let (about, long_about) = match (&variant.kind, &variant.about) {
        (Kind::Newtype(_), Some((about, long_about))) => {
            (quote! { #about }, quote! { #long_about })
        }
        _ => (
            quote! { #args::COMMAND.about },
            quote! { #args::COMMAND.long_about },
        ),
    };
    let visible: Vec<&str> = variant.visible_aliases();
    let aliases: Vec<&str> = variant
        .aliases
        .iter()
        .filter(|alias| !alias.visible)
        .map(|alias| alias.name.as_str())
        .collect();
    quote! {
        {
            const LONG_ABOUT: &str = #long_about;
            const HELP: &[getargs_derive::ArgInfo] = &[getargs_derive::__private::help_arg(
                !LONG_ABOUT.is_empty() || #args::HAS_LONG_HELP,
            )];
            const NONE: &[getargs_derive::ArgInfo] = &[];
            const PARTS: &[&[getargs_derive::ArgInfo]] = &[
                #args::ARGS,
                if #args::COMMAND.help_flag { HELP } else { NONE },
            ];
            const LENGTH: usize = getargs_derive::__private::count(PARTS);
            const MERGED: [getargs_derive::ArgInfo; LENGTH] = getargs_derive::__private::concat(
                PARTS,
                getargs_derive::ArgInfo::new("", getargs_derive::ArgKind::Flag),
            );
            getargs_derive::__private::subcommand_info(
                #name,
                #about,
                LONG_ABOUT,
                &MERGED,
                &#args::COMMAND,
            )
            .visible_aliases(&[#(#visible),*])
            .aliases(&[#(#aliases),*])
            .hidden(#hidden)
        }
    }
}

fn build(variant: &Variant, ty: &Type) -> TokenStream {
    let ident = &variant.ident;
    let args = quote! { <#ty as getargs_derive::__private::Args<'__getargs>> };
    match &variant.kind {
        Kind::Unit => quote! {
            let _ = #args::finish(state, __getargs_env)?;
            core::result::Result::Ok(Self::#ident)
        },
        Kind::Named(fields) => quote! {
            let value = #args::finish(state, __getargs_env)?;
            core::result::Result::Ok(Self::#ident { #(#fields: value.#fields),* })
        },
        Kind::Newtype(_) => quote! {
            core::result::Result::Ok(Self::#ident(#args::finish(state, __getargs_env)?))
        },
    }
}

pub(crate) fn expand(input: &DeriveInput, variants: &[Variant]) -> TokenStream {
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

    let mut items = Vec::new();
    for variant in variants {
        if let Some((args_input, model)) = &variant.args {
            items.push(hidden_struct(args_input));
            items.push(variant_args(args_input, model));
        }
    }

    let types: Vec<Type> = variants.iter().map(variant_type).collect();
    let args: Vec<TokenStream> = types
        .iter()
        .map(|ty| quote! { <#ty as getargs_derive::__private::Args<#lifetime>> })
        .collect();
    let entries = variants
        .iter()
        .zip(&types)
        .map(|(variant, ty)| entry(variant, ty));
    let infos = variants
        .iter()
        .zip(&types)
        .map(|(variant, ty)| info(variant, ty));
    let names: Vec<&str> = variants
        .iter()
        .filter(|variant| !variant.hidden)
        .flat_map(|variant| std::iter::once(variant.name.as_str()).chain(variant.visible_aliases()))
        .collect();
    let visible: Vec<&str> = variants
        .iter()
        .filter(|variant| !variant.hidden)
        .map(|variant| variant.name.as_str())
        .collect();
    let mut env = quote! { getargs_derive::__private::NoEnv };
    for args in &args {
        env = quote! { <#env as getargs_derive::__private::EnvFlag>::Or<#args::Env> };
    }
    let states: Vec<_> = (0..variants.len())
        .map(|index| format_ident!("__state_{}", index))
        .collect();
    let arms = variants.iter().enumerate().map(|(index, variant)| {
        let primary = &variant.name;
        let aliases = variant.aliases.iter().map(|alias| &alias.name);
        let ty = &types[index];
        let values = (0..variants.len()).map(|position| {
            if position == index {
                quote! { core::option::Option::Some(__state) }
            } else {
                quote! { core::option::Option::None }
            }
        });
        quote! {
            #primary #(| #aliases)* => {
                let __context = __context.command(getargs_derive::__private::node(
                    <Self as getargs_derive::__private::Subcommand<#lifetime>>::COMMANDS,
                    __context.node,
                    #index,
                ));
                let __state = getargs_derive::__private::command::<#ty, __I>(
                    __options,
                    __context,
                    __globals,
                )?;
                core::result::Result::Ok(core::option::Option::Some((#(#values,)*)))
            }
        }
    });
    let finishes = variants
        .iter()
        .zip(&types)
        .zip(&states)
        .map(|((variant, ty), state_ident)| {
            let build = build(variant, ty);
            quote! {
                if let core::option::Option::Some(state) = #state_ident {
                    #build
                } else
            }
        });

    quote! {
        #(#items)*

        impl #impl_generics getargs_derive::__private::Subcommand<#lifetime> for #name #ty_generics #where_clause {
            type State = (#(core::option::Option<#args::State>,)*);
            type Env = #env;

            const COMMANDS: &'static [getargs_derive::__private::CommandEntry] = &[#(#entries),*];
            const NAMES: &'static [&'static str] = &[#(#names),*];
            const VISIBLE: &'static [&'static str] = &[#(#visible),*];
            const INFOS: &'static [getargs_derive::CommandInfo] = &[#(#infos),*];

            fn try_command<__I: Iterator<Item = &#lifetime str>>(
                __name: &#lifetime str,
                __options: &mut getargs::Options<&#lifetime str, __I>,
                __context: getargs_derive::__private::Context<#lifetime, '_>,
                __globals: &mut getargs_derive::__private::Globals<'_, #lifetime, __I>,
            ) -> core::result::Result<
                core::option::Option<Self::State>,
                getargs_derive::Error<#lifetime>,
            > {
                match __name {
                    #(#arms)*
                    _ => core::result::Result::Ok(core::option::Option::None),
                }
            }

            fn finish<__E: FnMut(&str) -> core::option::Option<&#lifetime str>>(
                __state: Self::State,
                __getargs_env: &mut __E,
            ) -> core::result::Result<Self, getargs_derive::Error<#lifetime>> {
                let (#(#states,)*) = __state;
                #(#finishes)* {
                    core::unreachable!("a subcommand state is set when it is parsed")
                }
            }
        }
    }
}
