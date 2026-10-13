#![deny(missing_docs)]
//! Internal proc-macro implementation for `getargs-derive`.
//!
//! This crate is not meant to be used directly. Re-exported via `getargs_derive::GetArgs`.

mod expand;
mod help;
mod model;
mod value_enum;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derive macro for the [`getargs`](https://crates.io/crates/getargs) crate.
///
/// Generates a `parse` method on your struct that consumes CLI arguments via
/// `getargs::Options` and populates the struct's fields, along with a
/// `HELP` constant holding the generated help text.
///
/// # Attribute syntax
///
/// See the [crate-level documentation](getargs_derive) for a full reference of
/// supported `#[arg(...)]` and `#[command(...)]` attributes.
#[proc_macro_derive(GetArgs, attributes(arg, command, default))]
pub fn derive_get_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match model::parse(&input) {
        Ok((container, fields)) => expand::expand(&input, &container, &fields).into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Derive macro implementing `getargs_derive::ValueEnum` and `FromStr` for an enum with unit
/// variants.
///
/// Each variant is accepted under its kebab-cased name, which `#[value(name = "...")]` overrides,
/// and under every `#[value(alias = "...")]`.
///
/// See the [value enum guide](getargs_derive::guide::value_enum) for details.
#[proc_macro_derive(ValueEnum, attributes(value))]
pub fn derive_value_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match value_enum::parse(&input) {
        Ok(variants) => value_enum::expand(&input, &variants).into(),
        Err(error) => error.to_compile_error().into(),
    }
}
