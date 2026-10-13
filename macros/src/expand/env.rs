use super::validation::convert;
use crate::model::FieldModel;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(super) fn has_env(fields: &[FieldModel]) -> bool {
    fields.iter().any(|field| field.env.is_some())
}

pub(super) fn env_lookup(field: &FieldModel) -> Option<TokenStream> {
    let env = field.env.as_deref()?;
    let value_ident = format_ident!("__value_{}", field.name);
    let error_name = format!("{} [env: {env}]", field.error_name());
    let parsed = convert(field, quote! { value }, &error_name);
    Some(quote! {
        if #value_ident.is_none() {
            if let core::option::Option::Some(value) = __getargs_env(#env) {
                #value_ident = core::option::Option::Some(#parsed);
            }
        }
    })
}

pub(super) fn parse_methods(lifetime_params: &[syn::Lifetime], body: TokenStream) -> TokenStream {
    quote! {
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
