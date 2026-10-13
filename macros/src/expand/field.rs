use super::env::env_lookup;
use super::parsing;
use super::validation::convert;
use crate::model::{DefaultValue, FieldKind, FieldModel, FlagAction};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

#[derive(Default)]
pub(super) struct FieldTokens {
    pub(super) initializers: Vec<TokenStream>,
    pub(super) option_match_arms: Vec<TokenStream>,
    pub(super) positional_match_arms: Vec<TokenStream>,
    pub(super) finalizers: Vec<TokenStream>,
    pub(super) field_builds: Vec<TokenStream>,
    last_positional: Option<usize>,
}

fn option_pattern(field: &FieldModel) -> TokenStream {
    let shorts = field
        .short
        .into_iter()
        .chain(field.short_aliases.iter().map(|alias| alias.name));
    let longs = field
        .long
        .iter()
        .chain(field.long_aliases.iter().map(|alias| &alias.name));
    let patterns: Vec<TokenStream> = shorts
        .map(|short| quote! { getargs::Arg::Short(#short) })
        .chain(longs.map(|long| quote! { getargs::Arg::Long(#long) }))
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

impl FieldTokens {
    pub(super) fn push(&mut self, field: &FieldModel) {
        let field_ident = &field.ident;
        let field_type = &field.ty;
        let value_type = &field.value_ty;
        let value_ident = format_ident!("__value_{}", field.name);

        self.initializers.push(quote! {
            let mut #value_ident: core::option::Option<#value_type> = core::option::Option::None;
        });

        match field.kind {
            FieldKind::Flag(action) => {
                let pattern = option_pattern(field);
                let update = match action {
                    FlagAction::SetTrue => quote! { true },
                    FlagAction::SetFalse => quote! { false },
                    FlagAction::Count => quote! {
                        match #value_ident {
                            core::option::Option::Some(count) => count.saturating_add(1),
                            core::option::Option::None => 1,
                        }
                    },
                };
                self.option_match_arms.push(quote! {
                    #pattern => {
                        #value_ident = core::option::Option::Some(#update);
                    }
                });
            }
            FieldKind::Option => {
                let pattern = option_pattern(field);
                let error_name = field.error_name();
                let parsed = convert(field, quote! { value }, &error_name);
                let value = parsing::option_value(
                    field,
                    quote! { getargs_derive::Error::MissingOptionValue(#error_name) },
                );
                self.option_match_arms.push(quote! {
                    #pattern => {
                        let value = #value;
                        #value_ident = core::option::Option::Some(#parsed);
                    }
                });
            }
            FieldKind::Positional => {
                let index = self.positional_match_arms.len();
                if field.parsing.last {
                    self.last_positional = Some(index);
                }
                let parsed = convert(field, quote! { value }, &field.error_name());
                self.positional_match_arms.push(quote! {
                    #index => {
                        #value_ident = core::option::Option::Some(#parsed);
                    }
                });
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
    }

    pub(super) fn positional_handler(&self) -> TokenStream {
        let positional_match_arms = &self.positional_match_arms;
        let last_positional = parsing::last_positional(self.last_positional);
        if positional_match_arms.is_empty() {
            quote! {
                getargs::Arg::Positional(value) => {
                    return Err(getargs_derive::Error::UnexpectedArgument(value));
                }
            }
        } else {
            quote! {
                getargs::Arg::Positional(value) => {
                    #last_positional
                    match __positional_index {
                        #(#positional_match_arms)*
                        _ => return Err(getargs_derive::Error::UnexpectedArgument(value)),
                    }
                    __positional_index += 1;
                }
            }
        }
    }
}
