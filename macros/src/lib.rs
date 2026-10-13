#![deny(missing_docs)]
//! Internal proc-macro implementation for `getargs-derive`.
//!
//! This crate is not meant to be used directly. Re-exported via `getargs_derive::GetArgs`.

mod expand;
mod model;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derive macro for the [`getargs`](https://crates.io/crates/getargs) crate.
///
/// Generates a `parse` method on your struct that consumes CLI arguments via
/// `getargs::Options` and populates the struct's fields.
///
/// # Attribute syntax
///
/// See the [crate-level documentation](getargs_derive) for a full reference of
/// supported `#[arg(...)]` attributes.
#[proc_macro_derive(GetArgs, attributes(arg, default))]
pub fn derive_get_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match model::parse_fields(&input) {
        Ok(fields) => expand::expand(&input, &fields).into(),
        Err(error) => error.to_compile_error().into(),
    }
}
