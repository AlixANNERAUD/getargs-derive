use crate::model::{FieldKind, FieldModel};
use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn option_value(field: &FieldModel, missing: TokenStream) -> TokenStream {
    let next = quote! { options.value().map_err(|_| #missing)? };
    if field.parsing.allow_hyphen_values {
        return quote! { #next };
    }
    let allow_negative_numbers = field.parsing.allow_negative_numbers;
    quote! {
        match options.value_opt() {
            core::option::Option::Some(value) => value,
            core::option::Option::None => {
                let value = #next;
                if !getargs_derive::__accepts_value(value, #allow_negative_numbers) {
                    return Err(#missing);
                }
                value
            }
        }
    }
}

pub(super) fn negative_numbers(fields: &[FieldModel]) -> TokenStream {
    let indices: Vec<usize> = fields
        .iter()
        .filter(|field| matches!(field.kind, FieldKind::Positional))
        .enumerate()
        .filter(|(_, field)| field.parsing.allow_negative_numbers)
        .map(|(index, _)| index)
        .collect();
    if indices.is_empty() {
        return quote! {};
    }
    quote! {
        let argument = match argument {
            getargs::Arg::Short(short) if matches!(__positional_index, #(#indices)|*) => {
                match getargs_derive::__negative_number(__raw, short) {
                    core::option::Option::Some(number) => {
                        let _ = options.value_opt();
                        getargs::Arg::Positional(number)
                    }
                    core::option::Option::None => argument,
                }
            }
            argument => argument,
        };
    }
}

pub(super) fn last_positional(index: Option<usize>) -> TokenStream {
    let Some(index) = index else {
        return quote! {};
    };
    quote! {
        if options.opts_ended() {
            __positional_index = __positional_index.max(#index);
        } else if __positional_index == #index {
            return Err(getargs_derive::Error::UnexpectedArgument(value));
        }
    }
}
