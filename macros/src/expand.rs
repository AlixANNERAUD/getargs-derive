mod field;
mod help;
mod version;

use crate::model::{ContainerModel, FieldModel};
use field::FieldTokens;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, GenericParam};

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

    let mut tokens = FieldTokens::default();
    for field in fields {
        tokens.push(field);
    }
    let positional_handler = tokens.positional_handler();
    let FieldTokens {
        initializers,
        option_match_arms,
        finalizers,
        field_builds,
        ..
    } = tokens;

    let help_items = help::help_items(container, fields);
    let help_handler = help::help_handler(container);
    let (version_items, version_handler) = version::version_tokens(container);

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #help_items

            #version_items

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
                        #version_handler
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
