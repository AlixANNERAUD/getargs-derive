use crate::model::{FieldKind, FlattenModel, Model, Slot};
use proc_macro2::{Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{ToTokens, quote, quote_spanned};
use syn::{DeriveInput, GenericParam};

fn static_lifetimes(tokens: TokenStream) -> TokenStream {
    let mut output = Vec::new();
    let mut tokens = tokens.into_iter().peekable();
    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Punct(punct) if punct.as_char() == '\'' => {
                if let Some(TokenTree::Ident(ident)) = tokens.peek() {
                    let span = ident.span();
                    tokens.next();
                    let mut quote = Punct::new('\'', Spacing::Joint);
                    quote.set_span(punct.span());
                    output.push(TokenTree::Punct(quote));
                    output.push(TokenTree::Ident(Ident::new("static", span)));
                } else {
                    output.push(TokenTree::Punct(punct));
                }
            }
            TokenTree::Group(group) => {
                let mut replaced = Group::new(group.delimiter(), static_lifetimes(group.stream()));
                replaced.set_span(group.span());
                output.push(TokenTree::Group(replaced));
            }
            other => output.push(other),
        }
    }
    output.into_iter().collect()
}

pub(crate) fn static_args(ty: impl ToTokens) -> TokenStream {
    let ty = static_lifetimes(ty.to_token_stream());
    quote! { <#ty as getargs_derive::__private::Args<'static>> }
}

pub(super) fn static_commands(ty: impl ToTokens) -> TokenStream {
    let ty = static_lifetimes(ty.to_token_stream());
    quote! { <#ty as getargs_derive::__private::Subcommand<'static>> }
}

pub(super) fn static_type(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let lifetimes: Vec<_> = input
        .generics
        .params
        .iter()
        .filter(|parameter| matches!(parameter, GenericParam::Lifetime(_)))
        .map(|_| quote! { 'static })
        .collect();
    if lifetimes.is_empty() {
        quote! { #name }
    } else {
        quote! { #name<#(#lifetimes),*> }
    }
}

pub(super) fn static_self(input: &DeriveInput) -> TokenStream {
    static_args(static_type(input))
}

pub(super) struct Merged<'a> {
    pub(super) element: TokenStream,
    pub(super) fill: TokenStream,
    pub(super) constant: &'a str,
}

impl Merged<'_> {
    pub(super) fn tokens(
        &self,
        model: &Model,
        item: impl Fn(&crate::model::FieldModel) -> Option<Vec<TokenStream>>,
    ) -> TokenStream {
        let element = &self.element;
        if model.flattened.is_empty() {
            let items = model.fields.iter().filter_map(item).flatten();
            return quote! { &[#(#items),*] };
        }
        let constant = Ident::new(self.constant, Span::call_site());
        let parts = model.slots().into_iter().filter_map(|slot| match slot {
            Slot::Arg(field) => item(field).map(|items| quote! { &[#(#items),*] }),
            Slot::Flatten(flatten) => {
                let args = static_args(&flatten.ty);
                Some(quote! { #args::#constant })
            }
        });
        let fill = &self.fill;
        quote! {
            {
                const PARTS: &[&[#element]] = &[#(#parts),*];
                const LENGTH: usize = getargs_derive::__private::count(PARTS);
                const MERGED: [#element; LENGTH] = getargs_derive::__private::concat(PARTS, #fill);
                &MERGED
            }
        }
    }
}

fn check(flatten: &FlattenModel, condition: TokenStream, message: String) -> TokenStream {
    quote_spanned! {flatten.span=>
        const _: () = ::core::assert!(#condition, #message);
    }
}

fn own_checks(model: &Model, flatten: &FlattenModel) -> Vec<TokenStream> {
    let args = static_args(&flatten.ty);
    let target = &flatten.name;
    let mut checks = Vec::new();
    let mut shorts: Vec<(char, String)> = model
        .fields
        .iter()
        .flat_map(|field| {
            field
                .short
                .into_iter()
                .chain(field.short_aliases.iter().map(|alias| alias.name))
                .map(|short| (short, format!("field `{}`", field.name)))
        })
        .collect();
    let mut longs: Vec<(String, String)> = model
        .fields
        .iter()
        .flat_map(|field| {
            field
                .long
                .iter()
                .chain(field.long_aliases.iter().map(|alias| &alias.name))
                .map(|long| (long.clone(), format!("field `{}`", field.name)))
        })
        .collect();
    let container = &model.container;
    if !container.disable_help_flag {
        shorts.push(('h', "the generated help flag".to_owned()));
        longs.push(("help".to_owned(), "the generated help flag".to_owned()));
    }
    if container.version.is_some() {
        shorts.push(('V', "the generated version flag".to_owned()));
        longs.push((
            "version".to_owned(),
            "the generated version flag".to_owned(),
        ));
    }
    for (short, owner) in shorts {
        checks.push(check(
            flatten,
            quote! { !getargs_derive::__private::contains_short(#args::SHORTS, #short) },
            format!(
                "short option `-{short}` of flattened field `{target}` is already used by {owner}"
            ),
        ));
    }
    for (long, owner) in longs {
        checks.push(check(
            flatten,
            quote! { !getargs_derive::__private::contains_long(#args::LONGS, #long) },
            format!(
                "long option `--{long}` of flattened field `{target}` is already used by {owner}"
            ),
        ));
    }
    checks
}

fn pair_checks(first: &FlattenModel, second: &FlattenModel) -> Vec<TokenStream> {
    let first_args = static_args(&first.ty);
    let second_args = static_args(&second.ty);
    let (first_name, second_name) = (&first.name, &second.name);
    vec![
        check(
            second,
            quote! { !getargs_derive::__private::shares_short(#first_args::SHORTS, #second_args::SHORTS) },
            format!(
                "flattened fields `{first_name}` and `{second_name}` use the same short option"
            ),
        ),
        check(
            second,
            quote! { !getargs_derive::__private::shares_long(#first_args::LONGS, #second_args::LONGS) },
            format!("flattened fields `{first_name}` and `{second_name}` use the same long option"),
        ),
    ]
}

fn positional_checks(model: &Model) -> Vec<TokenStream> {
    let mut parts = Vec::new();
    let mut checks = Vec::new();
    for slot in model.slots() {
        let (part, span) = match slot {
            Slot::Arg(field) => match field.kind {
                FieldKind::Positional => {
                    let optional = field.is_optional();
                    (
                        quote! { &[#optional] },
                        (!optional).then(|| field.ident.span()),
                    )
                }
                _ => continue,
            },
            Slot::Flatten(flatten) => {
                let args = static_args(&flatten.ty);
                (quote! { #args::POSITIONALS }, Some(flatten.span))
            }
        };
        let previous = parts.clone();
        parts.push(part);
        let Some(span) = span else {
            continue;
        };
        if model.flattened.is_empty() {
            continue;
        }
        checks.push(quote_spanned! {span=>
            const _: () = ::core::assert!(
                !getargs_derive::__private::positionals_ordered(&[#(#previous),*])
                    || getargs_derive::__private::positionals_ordered(&[#(#parts),*]),
                "a required positional argument cannot follow an optional positional argument, flattened fields included",
            );
        });
    }
    checks
}

fn last_checks(model: &Model) -> Vec<TokenStream> {
    let slots = model.slots();
    let mut checks = Vec::new();
    for (index, slot) in slots.iter().enumerate() {
        let (span, last) = match slot {
            Slot::Arg(field) if field.parsing.last => (field.ident.span(), None),
            Slot::Flatten(flatten) => {
                let args = static_args(&flatten.ty);
                (flatten.span, Some(quote! { #args::LAST }))
            }
            Slot::Arg(_) => continue,
        };
        let later = &slots[index + 1..];
        let own_later = later.iter().any(
            |slot| matches!(slot, Slot::Arg(field) if matches!(field.kind, FieldKind::Positional)),
        );
        let flattened_later: Vec<TokenStream> = later
            .iter()
            .filter_map(|slot| match slot {
                Slot::Flatten(flatten) => {
                    let args = static_args(&flatten.ty);
                    Some(quote! { #args::POSITIONALS.is_empty() })
                }
                Slot::Arg(_) => None,
            })
            .collect();
        let condition = match (last, own_later, flattened_later.is_empty()) {
            (None, true, _) | (_, false, true) => continue,
            (Some(last), true, _) => quote! { !#last },
            (None, false, false) => quote! { #(#flattened_later)&&* },
            (Some(last), false, false) => quote! { !#last || (#(#flattened_later)&&*) },
        };
        checks.push(quote_spanned! {span=>
            const _: () = ::core::assert!(
                #condition,
                "a `last` positional argument must be the final positional argument, flattened fields included",
            );
        });
    }
    checks
}

fn global_checks(model: &Model) -> Vec<TokenStream> {
    let Some(subcommand) = &model.subcommand else {
        return Vec::new();
    };
    let commands = static_commands(&subcommand.ty);
    let mut checks = Vec::new();
    for field in model.fields.iter().filter(|field| field.parsing.global) {
        let span = field.ident.span();
        let shorts = field
            .short
            .into_iter()
            .chain(field.short_aliases.iter().map(|alias| alias.name));
        for short in shorts {
            let message = format!(
                "global option `-{short}` of field `{}` is also used by a subcommand",
                field.name
            );
            checks.push(quote_spanned! {span=>
                const _: () = ::core::assert!(
                    !getargs_derive::__private::commands_use_short(#commands::COMMANDS, #short),
                    #message,
                );
            });
        }
        let longs = field
            .long
            .iter()
            .chain(field.long_aliases.iter().map(|alias| &alias.name));
        for long in longs {
            let message = format!(
                "global option `--{long}` of field `{}` is also used by a subcommand",
                field.name
            );
            checks.push(quote_spanned! {span=>
                const _: () = ::core::assert!(
                    !getargs_derive::__private::commands_use_long(#commands::COMMANDS, #long),
                    #message,
                );
            });
        }
    }
    checks
}

pub(super) fn checks(model: &Model) -> TokenStream {
    let mut checks = global_checks(model);
    if model.flattened.is_empty() {
        return quote! { #(#checks)* };
    }
    for (index, (_, flatten)) in model.flattened.iter().enumerate() {
        let args = static_args(&flatten.ty);
        let message = format!(
            "flattened field `{}` has a `#[command(subcommand)]` field, which cannot be flattened",
            flatten.name
        );
        checks.push(check(
            flatten,
            quote! { #args::COMMAND.commands.is_empty() },
            message,
        ));
        checks.extend(own_checks(model, flatten));
        for (_, previous) in &model.flattened[..index] {
            checks.extend(pair_checks(previous, flatten));
        }
    }
    checks.extend(positional_checks(model));
    checks.extend(last_checks(model));
    quote! { #(#checks)* }
}
