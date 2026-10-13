use crate::help::help_text;
use crate::model::{ContainerModel, DefaultValue, FieldKind, FieldModel};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, GenericParam};

fn option_pattern(field: &FieldModel) -> TokenStream {
    match (field.short, &field.long) {
        (Some(short), Some(long)) => {
            quote! { getargs::Arg::Short(#short) | getargs::Arg::Long(#long) }
        }
        (Some(short), None) => quote! { getargs::Arg::Short(#short) },
        (None, Some(long)) => quote! { getargs::Arg::Long(#long) },
        (None, None) => unreachable!("named options always have a short or long name"),
    }
}

fn parse_value(field: &FieldModel, value: TokenStream) -> TokenStream {
    let error_name = field.error_name();
    if field.is_str {
        value
    } else {
        quote! {
            #value.parse().map_err(|_| getargs_derive::Error::ParseError(#error_name))?
        }
    }
}

fn fallback(field: &FieldModel) -> TokenStream {
    let error_name = field.error_name();
    match (&field.default, &field.kind) {
        (Some(DefaultValue::Literal(literal)), _) if field.is_str => quote! { #literal },
        (Some(DefaultValue::Literal(literal)), _) => quote! {
            core::str::FromStr::from_str(#literal)
                .map_err(|_| getargs_derive::Error::ParseError(#error_name))?
        },
        (Some(DefaultValue::Expr(expr)), _) => quote! { #expr },
        (None, FieldKind::Flag) => quote! { false },
        (None, _) => quote! {
            return Err(getargs_derive::Error::MissingPositionalArgument(#error_name))
        },
    }
}

pub(crate) fn expand(
    input: &DeriveInput,
    container: &ContainerModel,
    fields: &[FieldModel],
) -> TokenStream {
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

    let mut initializers = Vec::new();
    let mut option_match_arms = Vec::new();
    let mut positional_match_arms = Vec::new();
    let mut finalizers = Vec::new();
    let mut field_builds = Vec::new();

    for field in fields {
        let field_ident = &field.ident;
        let field_type = &field.ty;
        let value_ident = format_ident!("__value_{}", field.name);

        initializers.push(quote! {
            let mut #value_ident: core::option::Option<#field_type> = core::option::Option::None;
        });

        match field.kind {
            FieldKind::Flag => {
                let pattern = option_pattern(field);
                option_match_arms.push(quote! {
                    #pattern => {
                        #value_ident = core::option::Option::Some(true);
                    }
                });
            }
            FieldKind::Option => {
                let pattern = option_pattern(field);
                let error_name = field.error_name();
                let parsed = parse_value(field, quote! { value });
                option_match_arms.push(quote! {
                    #pattern => {
                        let value = options
                            .value()
                            .map_err(|_| getargs_derive::Error::MissingOptionValue(#error_name))?;
                        #value_ident = core::option::Option::Some(#parsed);
                    }
                });
            }
            FieldKind::Positional => {
                let index = positional_match_arms.len();
                let parsed = parse_value(field, quote! { value });
                positional_match_arms.push(quote! {
                    #index => {
                        #value_ident = core::option::Option::Some(#parsed);
                    }
                });
            }
        }

        let fallback = fallback(field);
        finalizers.push(quote! {
            let #value_ident: #field_type = match #value_ident {
                core::option::Option::Some(value) => value,
                core::option::Option::None => #fallback,
            };
        });

        field_builds.push(quote! { #field_ident: #value_ident });
    }

    let positional_handler = if positional_match_arms.is_empty() {
        quote! {
            getargs::Arg::Positional(_) => {
                return Err(getargs_derive::Error::InvalidNumberOfArguments);
            }
        }
    } else {
        quote! {
            getargs::Arg::Positional(value) => {
                match __positional_index {
                    #(#positional_match_arms)*
                    _ => return Err(getargs_derive::Error::InvalidNumberOfArguments),
                }
                __positional_index += 1;
            }
        }
    };

    let help = help_text(container, fields);
    let help_handler = if container.disable_help_flag {
        quote! {}
    } else {
        quote! {
            getargs::Arg::Short('h') | getargs::Arg::Long("help") => {
                return Err(getargs_derive::Error::Help(Self::HELP));
            }
        }
    };

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Help text generated at compile time from the struct, its fields and their documentation.
            ///
            /// Returned in [`Error::Help`](getargs_derive::Error::Help) when `-h` or `--help` is given.
            pub const HELP: &'static str = #help;

            /// Parse CLI arguments from an iterator, using `getargs::Options`.
            ///
            /// This is the low-level method. Prefer [`from_args`](Self::from_args) for convenience.
            pub fn parse<'__getargs, I>(options: &mut getargs::Options<&'__getargs str, I>) -> core::result::Result<Self, getargs_derive::Error>
            where
                I: Iterator<Item = &'__getargs str>,
                #('__getargs: #lifetime_params,)*
            {
                #(#initializers)*
                let mut __positional_index = 0_usize;

                while let Some(argument) = options.next_arg().map_err(|_| getargs_derive::Error::UnknownOption)? {
                    match argument {
                        #help_handler
                        #(#option_match_arms)*
                        #positional_handler
                        _ => return Err(getargs_derive::Error::UnknownOption),
                    }
                }

                #(#finalizers)*

                Ok(Self {
                    #(#field_builds),*
                })
            }

            /// Parse CLI arguments directly from an iterator.
            ///
            /// Convenience wrapper that creates a `getargs::Options` internally
            /// and calls [`parse`](Self::parse).
            ///
            /// # Example
            ///
            /// ```ignore
            /// let params = MyStruct::from_args(std::env::args().skip(1))?;
            /// ```
            pub fn from_args<'__getargs, I>(iter: I) -> core::result::Result<Self, getargs_derive::Error>
            where
                I: Iterator<Item = &'__getargs str>,
                #('__getargs: #lifetime_params,)*
            {
                let mut options = getargs::Options::new(iter);
                Self::parse(&mut options)
            }
        }
    }
}
