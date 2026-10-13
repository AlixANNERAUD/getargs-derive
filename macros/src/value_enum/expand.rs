use super::model::Variant;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, ext::IdentExt};

pub(crate) fn expand(input: &DeriveInput, variants: &[Variant]) -> TokenStream {
    let name = &input.ident;
    let idents: Vec<_> = variants.iter().map(|variant| &variant.ident).collect();
    let ident_names: Vec<String> = variants
        .iter()
        .map(|variant| variant.ident.unraw().to_string())
        .collect();
    let names: Vec<&str> = variants
        .iter()
        .map(|variant| variant.name.as_str())
        .collect();
    let patterns = variants.iter().map(|variant| {
        let name = &variant.name;
        let aliases = &variant.aliases;
        quote! { #name #(| #aliases)* }
    });

    quote! {
        impl getargs_derive::ValueEnum for #name {
            const VARIANTS: &'static [Self] = &[#(Self::#idents),*];

            const POSSIBLE_VALUES: &'static [&'static str] = &[#(#names),*];

            const __VARIANT_IDENTS: &'static [&'static str] = &[#(#ident_names),*];

            fn value_name(&self) -> &'static str {
                match self {
                    #(Self::#idents => #names,)*
                }
            }

            fn from_value(value: &str) -> core::option::Option<Self> {
                match value {
                    #(#patterns => core::option::Option::Some(Self::#idents),)*
                    _ => core::option::Option::None,
                }
            }
        }

        impl core::str::FromStr for #name {
            type Err = getargs_derive::ParseValueEnumError;

            fn from_str(value: &str) -> core::result::Result<Self, Self::Err> {
                <Self as getargs_derive::ValueEnum>::from_value(value)
                    .ok_or(getargs_derive::ParseValueEnumError)
            }
        }
    }
}
