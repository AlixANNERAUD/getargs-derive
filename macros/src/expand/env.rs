use super::validation::convert;
use crate::model::{FieldModel, Model};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

fn has_env(fields: &[FieldModel]) -> bool {
    fields.iter().any(|field| field.env.is_some())
}

pub(super) fn env_lookup(field: &FieldModel) -> Option<TokenStream> {
    let env = field.env.as_deref()?;
    let value_ident = format_ident!("__value_{}", field.name);
    let error_name = format!("{} [env: {env}]", field.error_name());
    let parsed = convert(field, quote! { value }, &error_name);
    Some(quote! {
        let #value_ident = match #value_ident {
            core::option::Option::Some(value) => core::option::Option::Some(value),
            core::option::Option::None => match __getargs_env(#env) {
                core::option::Option::Some(value) => core::option::Option::Some(#parsed),
                core::option::Option::None => core::option::Option::None,
            },
        };
    })
}

fn hidden_parse(lifetime_params: &[syn::Lifetime], body: TokenStream) -> TokenStream {
    quote! {
        #[doc(hidden)]
        pub fn __parse<'__getargs, I, E>(
            options: &mut getargs::Options<&'__getargs str, I>,
            __raw: core::option::Option<&core::cell::Cell<core::option::Option<&'__getargs str>>>,
            mut __getargs_env: E,
        ) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
        where
            I: Iterator<Item = &'__getargs str>,
            E: FnMut(&str) -> core::option::Option<&'__getargs str>,
            #('__getargs: #lifetime_params,)*
        {
            #body
        }
    }
}

fn with_env_methods(lifetime_params: &[syn::Lifetime]) -> TokenStream {
    quote! {
        /// Parse CLI arguments from an iterator, using `getargs::Options`, reading environment
        /// variable fallbacks with `env`.
        ///
        /// `env` is called with the name of an environment variable and returns its value, or
        /// `None` when it is not set. It is only called for arguments absent from the command line.
        pub fn parse_with_env<'__getargs, I, E>(options: &mut getargs::Options<&'__getargs str, I>, env: E) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
        where
            I: Iterator<Item = &'__getargs str>,
            E: FnMut(&str) -> core::option::Option<&'__getargs str>,
            #('__getargs: #lifetime_params,)*
        {
            Self::__parse(options, core::option::Option::None, env)
        }

        /// Parse CLI arguments directly from an iterator, reading environment variable fallbacks
        /// with `env`.
        ///
        /// Convenience wrapper that creates a `getargs::Options` internally
        /// and calls [`parse_with_env`](Self::parse_with_env).
        pub fn from_args_with_env<'__getargs, I, E>(iter: I, env: E) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
        where
            I: Iterator<Item = &'__getargs str>,
            E: FnMut(&str) -> core::option::Option<&'__getargs str>,
            #('__getargs: #lifetime_params,)*
        {
            let raw = core::cell::Cell::new(core::option::Option::None);
            let mut options = getargs::Options::new(getargs_derive::__RawArgs::new(iter, &raw));
            Self::__parse(&mut options, core::option::Option::Some(&raw), env)
        }
    }
}

pub(super) fn parse_methods(
    model: &Model,
    lifetime_params: &[syn::Lifetime],
    body: TokenStream,
) -> TokenStream {
    let hidden = hidden_parse(lifetime_params, body);
    if has_env(&model.fields) {
        let with_env = with_env_methods(lifetime_params);
        return quote! {
            /// Parse CLI arguments from an iterator, using `getargs::Options`, reading environment
            /// variable fallbacks from the process environment.
            ///
            /// This requires the `std` feature of `getargs-derive`. Without it, use
            /// [`parse_with_env`](Self::parse_with_env) with your own lookup function.
            /// Prefer [`from_args`](Self::from_args) for convenience.
            pub fn parse<'__getargs, I>(options: &mut getargs::Options<&'__getargs str, I>) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
            where
                I: Iterator<Item = &'__getargs str> + getargs_derive::__EnvSupport,
                #('__getargs: #lifetime_params,)*
            {
                Self::__parse(options, core::option::Option::None, getargs_derive::__env)
            }

            #with_env

            /// Parse CLI arguments directly from an iterator, reading environment variable fallbacks
            /// from the process environment.
            ///
            /// Convenience wrapper that creates a `getargs::Options` internally
            /// and calls [`parse`](Self::parse). This requires the `std` feature of `getargs-derive`.
            /// Without it, use [`from_args_with_env`](Self::from_args_with_env).
            pub fn from_args<'__getargs, I>(iter: I) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
            where
                I: Iterator<Item = &'__getargs str> + getargs_derive::__EnvSupport,
                #('__getargs: #lifetime_params,)*
            {
                let raw = core::cell::Cell::new(core::option::Option::None);
                let mut options = getargs::Options::new(getargs_derive::__RawArgs::new(iter, &raw));
                Self::__parse(&mut options, core::option::Option::Some(&raw), getargs_derive::__env)
            }

            #hidden
        };
    }

    let (lookup, bound, with_env) = if model.flattened.is_empty() {
        (
            quote! { getargs_derive::__private::no_env },
            quote! {},
            quote! {},
        )
    } else {
        (
            quote! { getargs_derive::__env },
            quote! {
                + getargs_derive::__private::EnvSupport<
                    <Self as getargs_derive::__private::Args<'__getargs>>::Env,
                >
            },
            with_env_methods(lifetime_params),
        )
    };
    quote! {
        /// Parse CLI arguments from an iterator, using `getargs::Options`.
        ///
        /// This is the low-level method. Prefer [`from_args`](Self::from_args) for convenience.
        pub fn parse<'__getargs, I>(options: &mut getargs::Options<&'__getargs str, I>) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
        where
            I: Iterator<Item = &'__getargs str> #bound,
            #('__getargs: #lifetime_params,)*
        {
            Self::__parse(options, core::option::Option::None, #lookup)
        }

        #with_env

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
        pub fn from_args<'__getargs, I>(iter: I) -> core::result::Result<Self, getargs_derive::Error<'__getargs>>
        where
            I: Iterator<Item = &'__getargs str> #bound,
            #('__getargs: #lifetime_params,)*
        {
            let raw = core::cell::Cell::new(core::option::Option::None);
            let mut options = getargs::Options::new(getargs_derive::__RawArgs::new(iter, &raw));
            Self::__parse(&mut options, core::option::Option::Some(&raw), #lookup)
        }

        #hidden
    }
}
