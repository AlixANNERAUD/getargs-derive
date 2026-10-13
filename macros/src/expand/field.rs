use super::env::env_lookup;
use super::parsing;
use super::validation::convert;
use crate::model::{
    DefaultValue, FieldKind, FieldModel, FlagAction, FlattenModel, Slot, SubcommandModel,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Index, Type};

#[derive(Default)]
pub(super) struct FieldTokens {
    pub(super) state_types: Vec<TokenStream>,
    pub(super) initializers: Vec<TokenStream>,
    pub(super) value_idents: Vec<Ident>,
    pub(super) prologue: Vec<TokenStream>,
    pub(super) option_match_arms: Vec<TokenStream>,
    pub(super) steps: Vec<TokenStream>,
    pub(super) filled: Vec<TokenStream>,
    pub(super) finalizers: Vec<TokenStream>,
    pub(super) field_builds: Vec<TokenStream>,
    pub(super) global_arms: Vec<TokenStream>,
    pub(super) global_steps: Vec<TokenStream>,
    pub(super) subcommand: Option<Index>,
}

pub(super) fn subcommand_trait(ty: &Type) -> TokenStream {
    quote! { <#ty as getargs_derive::__private::Subcommand<'__getargs>> }
}

pub(super) fn option_pattern(field: &FieldModel) -> TokenStream {
    let shorts = field
        .short
        .into_iter()
        .chain(field.short_aliases.iter().map(|alias| alias.name));
    let longs = field
        .long
        .iter()
        .chain(field.long_aliases.iter().map(|alias| &alias.name));
    let patterns: Vec<TokenStream> = shorts
        .map(|short| quote! { getargs_derive::__private::Arg::Short(#short) })
        .chain(longs.map(|long| quote! { getargs_derive::__private::Arg::Long(#long) }))
        .collect();
    quote! { #(#patterns)|* }
}

fn fallback(field: &FieldModel) -> TokenStream {
    let error_name = field.error_name();
    match (&field.default, &field.kind) {
        (Some(DefaultValue::Literal(literal)), _) => {
            convert(field, quote! { #literal }, &error_name)
        }
        (Some(DefaultValue::Expr(expr)), _) => quote! { #expr },
        (None, FieldKind::Flag(FlagAction::SetTrue)) => quote! { false },
        (None, FieldKind::Flag(FlagAction::SetFalse)) => quote! { true },
        (None, FieldKind::Flag(FlagAction::Count)) => quote! { 0 },
        (None, _) => quote! {
            return Err(getargs_derive::Error::MissingArgument(#error_name))
        },
    }
}

pub(super) fn args_trait(ty: &Type) -> TokenStream {
    quote! { <#ty as getargs_derive::__private::Args<'__getargs>> }
}

pub(super) fn all(conditions: &[TokenStream]) -> TokenStream {
    if conditions.is_empty() {
        quote! { true }
    } else {
        quote! { #(#conditions)&&* }
    }
}

impl FieldTokens {
    pub(super) fn push(&mut self, slot: &Slot, negative_numbers: bool) {
        match slot {
            Slot::Arg(field) => self.push_arg(field),
            Slot::Flatten(flatten) => self.push_flatten(flatten, negative_numbers),
        }
    }

    fn push_flatten(&mut self, flatten: &FlattenModel, negative_numbers: bool) {
        let index = Index::from(self.state_types.len());
        let field_ident = &flatten.ident;
        let value_ident = format_ident!("__value_{}", flatten.name);
        let args = args_trait(&flatten.ty);
        let earlier = all(&self.filled);

        self.state_types.push(quote! { #args::State });
        self.initializers.push(quote! { #args::init() });
        self.steps.push(quote! {
            if #args::try_arg(
                &mut __state.#index,
                __argument,
                __options,
                __context.child(__last, #negative_numbers, !(#earlier)),
                __globals,
            )? {
                return core::result::Result::Ok(true);
            }
        });
        self.global_steps.push(quote! {
            if #args::try_global(
                &mut __state.#index,
                __argument,
                __options,
                __context.child(false, #negative_numbers, false),
            )? {
                return core::result::Result::Ok(true);
            }
        });
        self.filled.push(quote! { #args::filled(&__state.#index) });
        self.finalizers.push(quote! {
            let #value_ident = #args::finish(#value_ident, __getargs_env)?;
        });
        self.field_builds
            .push(quote! { #field_ident: #value_ident });
        self.value_idents.push(value_ident);
    }

    fn push_option_arm(&mut self, field: &FieldModel, arm: TokenStream) {
        if field.parsing.global {
            self.global_arms.push(arm.clone());
        }
        self.option_match_arms.push(arm);
    }

    pub(super) fn push_subcommand(&mut self, subcommand: &SubcommandModel) {
        let index = Index::from(self.state_types.len());
        let field_ident = &subcommand.ident;
        let value_ident = format_ident!("__value_{}", subcommand.name);
        let commands = subcommand_trait(&subcommand.ty);

        self.state_types
            .push(quote! { core::option::Option<#commands::State> });
        self.initializers
            .push(quote! { core::option::Option::None });
        let missing = if subcommand.optional {
            quote! { core::option::Option::None }
        } else {
            quote! {
                return Err(getargs_derive::Error::MissingSubcommand(#commands::VISIBLE))
            }
        };
        let found = if subcommand.optional {
            quote! { core::option::Option::Some(#commands::finish(state, __getargs_env)?) }
        } else {
            quote! { #commands::finish(state, __getargs_env)? }
        };
        self.finalizers.push(quote! {
            let #value_ident = match #value_ident {
                core::option::Option::Some(state) => #found,
                core::option::Option::None => #missing,
            };
        });
        self.field_builds
            .push(quote! { #field_ident: #value_ident });
        self.value_idents.push(value_ident);
        self.subcommand = Some(index);
    }

    fn push_arg(&mut self, field: &FieldModel) {
        if let Some(collection) = &field.collection {
            self.push_collection(field, collection);
            return;
        }
        let index = Index::from(self.state_types.len());
        let slot = quote! { __state.#index };
        let field_ident = &field.ident;
        let field_type = &field.ty;
        let value_type = &field.value_ty;
        let value_ident = format_ident!("__value_{}", field.name);

        self.state_types
            .push(quote! { core::option::Option<#value_type> });
        self.initializers
            .push(quote! { core::option::Option::None });

        match field.kind {
            FieldKind::Flag(action) => {
                let pattern = option_pattern(field);
                let update = match action {
                    FlagAction::SetTrue => quote! { true },
                    FlagAction::SetFalse => quote! { false },
                    FlagAction::Count => quote! {
                        match #slot {
                            core::option::Option::Some(count) => count.saturating_add(1),
                            core::option::Option::None => 1,
                        }
                    },
                };
                self.push_option_arm(
                    field,
                    quote! {
                        #pattern => {
                            #slot = core::option::Option::Some(#update);
                            return core::result::Result::Ok(true);
                        }
                    },
                );
            }
            FieldKind::Option => {
                let pattern = option_pattern(field);
                let error_name = field.error_name();
                let parsed = convert(field, quote! { value }, &error_name);
                let value = parsing::option_value(
                    field,
                    quote! { getargs_derive::Error::MissingOptionValue(#error_name) },
                );
                self.push_option_arm(
                    field,
                    quote! {
                        #pattern => {
                            let value = #value;
                            #slot = core::option::Option::Some(#parsed);
                            return core::result::Result::Ok(true);
                        }
                    },
                );
            }
            FieldKind::Positional => {
                let parsed = convert(field, quote! { value }, &field.error_name());
                let earlier = all(&self.filled);
                self.prologue.push(parsing::negative_positional(
                    field, &slot, &earlier, &parsed,
                ));
                self.steps.push(parsing::positional(field, &slot, &parsed));
                self.filled.push(quote! { #slot.is_some() });
            }
        }

        self.finalizers.extend(env_lookup(field));

        if !field.optional {
            let fallback = fallback(field);
            self.finalizers.push(quote! {
                let #value_ident: #field_type = match #value_ident {
                    core::option::Option::Some(value) => value,
                    core::option::Option::None => #fallback,
                };
            });
        }

        self.field_builds
            .push(quote! { #field_ident: #value_ident });
        self.value_idents.push(value_ident);
    }
}
