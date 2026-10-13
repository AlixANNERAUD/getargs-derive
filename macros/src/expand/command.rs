use super::flatten::{Merged, static_self};
use super::version::version_value;
use crate::help::{command_name, has_long_help};
use crate::model::{FieldKind, FieldModel, FlagAction, Model};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

fn arg_info(field: &FieldModel) -> TokenStream {
    let id = &field.name;
    let kind = match field.kind {
        FieldKind::Positional => quote! { Positional },
        FieldKind::Option => quote! { Option },
        FieldKind::Flag(_) => quote! { Flag },
    };
    let mut calls = Vec::new();
    if let Some(short) = field.short {
        calls.push(quote! { .short(#short) });
    }
    if let Some(long) = &field.long {
        calls.push(quote! { .long(#long) });
    }
    let (visible_shorts, shorts): (Vec<_>, Vec<_>) =
        field.short_aliases.iter().partition(|alias| alias.visible);
    let (visible_longs, longs): (Vec<_>, Vec<_>) =
        field.long_aliases.iter().partition(|alias| alias.visible);
    let lists = [
        (quote! { visible_short_aliases }, names(&visible_shorts)),
        (quote! { short_aliases }, names(&shorts)),
        (quote! { visible_aliases }, names(&visible_longs)),
        (quote! { aliases }, names(&longs)),
    ];
    for (method, names) in lists {
        if !names.is_empty() {
            calls.push(quote! { .#method(&[#(#names),*]) });
        }
    }
    if !matches!(field.kind, FieldKind::Flag(_)) {
        let value_name = &field.value_name;
        calls.push(quote! { .value_name(#value_name) });
    }
    if let Some(help) = field.help.as_deref().filter(|help| !help.is_empty()) {
        calls.push(quote! { .help(#help) });
    }
    if let Some(long_help) = field.long_help.as_deref().filter(|help| !help.is_empty()) {
        calls.push(quote! { .long_help(#long_help) });
    }
    if field.value_enum {
        let value_ty = &field.value_ty;
        calls.push(quote! {
            .possible_values(<#value_ty as getargs_derive::ValueEnum>::POSSIBLE_VALUES)
        });
    } else if !field.possible_values.is_empty() {
        let values = &field.possible_values;
        calls.push(quote! { .possible_values(&[#(#values),*]) });
    }
    if let Some(env) = &field.env {
        calls.push(quote! { .env(#env) });
    }
    if field.hide {
        calls.push(quote! { .hidden(true) });
    }
    if !matches!(field.kind, FieldKind::Flag(_)) && !field.is_optional() {
        calls.push(quote! { .required(true) });
    }
    if matches!(field.kind, FieldKind::Flag(FlagAction::Count)) || field.collection.is_some() {
        calls.push(quote! { .multiple(true) });
    }
    if field.parsing.last {
        calls.push(quote! { .last(true) });
    }
    quote! {
        getargs_derive::ArgInfo::new(#id, getargs_derive::ArgKind::#kind) #(#calls)*
    }
}

fn names<T: quote::ToTokens>(aliases: &[&crate::model::Alias<T>]) -> Vec<TokenStream> {
    aliases
        .iter()
        .map(|alias| {
            let name = &alias.name;
            quote! { #name }
        })
        .collect()
}

pub(super) fn constants(input: &DeriveInput, model: &Model) -> TokenStream {
    let container = &model.container;
    let args = Merged {
        element: quote! { getargs_derive::ArgInfo },
        fill: quote! { getargs_derive::ArgInfo::new("", getargs_derive::ArgKind::Flag) },
        constant: "ARGS",
    }
    .tokens(model, |field| Some(vec![arg_info(field)]));
    let has_long = if model.flattened.is_empty() {
        let has_long = has_long_help(container, &model.fields);
        quote! { #has_long }
    } else {
        let args = static_self(input);
        let long_about = container.long_about.is_some();
        quote! { (#long_about || #args::HAS_LONG_HELP) }
    };
    let mut reserved = Vec::new();
    if !container.disable_help_flag {
        reserved.push(quote! { getargs_derive::__private::help_arg(#has_long) });
    }
    if container.version.is_some() {
        reserved.push(quote! { getargs_derive::__private::version_arg() });
    }
    let command_args = if model.flattened.is_empty() {
        let items = model.fields.iter().map(arg_info);
        quote! { &[#(#items,)* #(#reserved),*] }
    } else {
        let args = static_self(input);
        quote! {
            {
                const PARTS: &[&[getargs_derive::ArgInfo]] = &[#args::ARGS, &[#(#reserved),*]];
                const LENGTH: usize = getargs_derive::__private::count(PARTS);
                const MERGED: [getargs_derive::ArgInfo; LENGTH] = getargs_derive::__private::concat(
                    PARTS,
                    getargs_derive::ArgInfo::new("", getargs_derive::ArgKind::Flag),
                );
                &MERGED
            }
        }
    };
    quote! {
        const ARGS: &'static [getargs_derive::ArgInfo] = #args;
        const COMMAND_ARGS: &'static [getargs_derive::ArgInfo] = #command_args;
    }
}

pub(super) fn command_impl(input: &DeriveInput, model: &Model) -> TokenStream {
    let container = &model.container;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let command_name = command_name(container);
    let mut calls = Vec::new();
    if let Some(about) = container.about.as_deref().filter(|about| !about.is_empty()) {
        calls.push(quote! { .about(#about) });
    }
    if let Some(long_about) = container
        .long_about
        .as_deref()
        .filter(|about| !about.is_empty())
    {
        calls.push(quote! { .long_about(#long_about) });
    }
    if let Some(version) = version_value(container) {
        calls.push(quote! { .version(#version) });
    }
    quote! {
        impl #impl_generics getargs_derive::Command for #name #ty_generics #where_clause {
            const COMMAND: getargs_derive::CommandInfo = getargs_derive::CommandInfo::new(#command_name)
                #(#calls)*
                .args(<Self as getargs_derive::__private::Args<'static>>::COMMAND_ARGS);
        }
    }
}
