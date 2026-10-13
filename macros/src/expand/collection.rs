use super::field::FieldTokens;
use super::parsing::{allows_negative_numbers, option_value};
use super::validation::convert;
use crate::model::{CollectionModel, FieldKind, FieldModel};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{Index, spanned::Spanned};

struct Push<'a> {
    field: &'a FieldModel,
    function: TokenStream,
    delimiter: TokenStream,
    max: TokenStream,
}

impl Push<'_> {
    fn tokens(
        &self,
        target: &TokenStream,
        value: TokenStream,
        delimiter: &TokenStream,
        error_name: &str,
    ) -> TokenStream {
        let parsed = convert(self.field, quote! { value }, error_name);
        let max = &self.max;
        let function = &self.function;
        quote! {
            #function(
                &mut #target,
                #value,
                #delimiter,
                #error_name,
                #max,
                |value| {
                    let converted = #parsed;
                    core::result::Result::Ok(converted)
                },
            )?;
        }
    }

    fn split(&self, target: &TokenStream, value: TokenStream, error_name: &str) -> TokenStream {
        self.tokens(target, value, &self.delimiter, error_name)
    }

    fn whole(&self, target: &TokenStream, value: TokenStream, error_name: &str) -> TokenStream {
        self.tokens(
            target,
            value,
            &quote! { core::option::Option::None },
            error_name,
        )
    }
}

impl FieldTokens {
    pub(super) fn push_collection(&mut self, field: &FieldModel, collection: &CollectionModel) {
        let index = Index::from(self.state_types.len());
        let slot = quote! { __state.#index };
        let field_ident = &field.ident;
        let field_type = &field.ty;
        let value_ident = format_ident!("__value_{}", field.name);
        let error_name = field.error_name();
        let span = field.ty.span();
        let push = Push {
            field,
            function: quote_spanned! { span=> getargs_derive::__push_values },
            delimiter: match collection.delimiter {
                Some(delimiter) => quote! { core::option::Option::Some(#delimiter) },
                None => quote! { core::option::Option::None },
            },
            max: match collection.max {
                Some(max) => quote! { #max },
                None => quote! { usize::MAX },
            },
        };

        self.state_types
            .push(quote! { core::option::Option<#field_type> });
        self.initializers
            .push(quote! { core::option::Option::None });

        match field.kind {
            FieldKind::Positional => {
                let earlier = super::field::all(&self.filled);
                let store = push.split(&slot, quote! { value }, &error_name);
                let allow_negative_numbers = allows_negative_numbers(field);
                let (accepts, negative) = if field.parsing.last {
                    (
                        quote! { __options.opts_ended() },
                        quote! { return Err(getargs_derive::Error::UnexpectedArgument(value)); },
                    )
                } else {
                    (
                        quote! { !(__last && __options.opts_ended()) },
                        quote! {
                            #store
                            return core::result::Result::Ok(true);
                        },
                    )
                };
                self.prologue.push(quote! {
                    if let getargs_derive::__private::Arg::Short(short) = __argument {
                        if #allow_negative_numbers && !__context.blocked && #earlier {
                            if let core::option::Option::Some(value) =
                                getargs_derive::__negative_number(__context.raw, short)
                            {
                                let _ = __options.value_opt();
                                #negative
                            }
                        }
                    }
                });
                self.steps.push(quote! {
                    if let getargs_derive::__private::Arg::Positional(value) = __argument {
                        if #accepts {
                            #store
                            return core::result::Result::Ok(true);
                        }
                    }
                });
                let min = collection.min;
                self.filled.push(if min == 0 {
                    quote! { true }
                } else {
                    quote! { getargs_derive::__values_len(&#slot) >= #min }
                });
            }
            FieldKind::Option | FieldKind::Flag(_) => {
                let pattern = super::field::option_pattern(field);
                let value = option_value(
                    field,
                    quote! { getargs_derive::Error::MissingOptionValue(#error_name) },
                );
                let store = push.split(&slot, quote! { value }, &error_name);
                self.option_match_arms.push(quote! {
                    #pattern => {
                        let value = #value;
                        #store
                        return core::result::Result::Ok(true);
                    }
                });
            }
        }

        let target = quote! { #value_ident };
        let mut fallbacks = Vec::new();
        if let Some(env) = field.env.as_deref() {
            let env_name = format!("{error_name} [env: {env}]");
            let store = push.split(&target, quote! { value }, &env_name);
            fallbacks.push(quote! {
                if #value_ident.is_none() {
                    if let core::option::Option::Some(value) = __getargs_env(#env) {
                        #store
                    }
                }
            });
        }
        let defaults: Vec<TokenStream> = match &field.default {
            Some(crate::model::DefaultValue::Literal(literal)) => {
                vec![push.split(&target, quote! { #literal }, &error_name)]
            }
            _ => collection
                .default_values
                .iter()
                .map(|literal| push.whole(&target, quote! { #literal }, &error_name))
                .collect(),
        };
        if !defaults.is_empty() {
            fallbacks.push(quote! {
                if #value_ident.is_none() {
                    #(#defaults)*
                }
            });
        }
        if !fallbacks.is_empty() {
            self.finalizers.push(quote! {
                let mut #value_ident = #value_ident;
                #(#fallbacks)*
            });
        }
        let min = collection.min;
        let finish = quote_spanned! { span=> getargs_derive::__finish_values };
        self.finalizers.push(quote! {
            let #value_ident: #field_type =
                #finish(#value_ident, #error_name, #min)?;
        });

        self.field_builds
            .push(quote! { #field_ident: #value_ident });
        self.value_idents.push(value_ident);
    }
}
