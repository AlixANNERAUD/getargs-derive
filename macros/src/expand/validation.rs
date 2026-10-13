use crate::help::expr_text;
use crate::model::{FieldModel, ValueParser};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ExprRange, RangeLimits};

fn range_text(range: &ExprRange) -> String {
    let start = range.start.as_deref().map(expr_text).unwrap_or_default();
    let end = range.end.as_deref().map(expr_text).unwrap_or_default();
    let limits = match range.limits {
        RangeLimits::HalfOpen(_) => "..",
        RangeLimits::Closed(_) => "..=",
    };
    format!("{start}{limits}{end}")
}

fn possible_values_check(field: &FieldModel, value: &TokenStream, error_name: &str) -> TokenStream {
    let values = &field.possible_values;
    if values.is_empty() {
        return quote! {};
    }
    quote! {
        if !::core::matches!(#value, #(#values)|*) {
            return Err(getargs_derive::Error::InvalidPossibleValue {
                name: #error_name,
                value: #value,
                possible_values: &[#(#values),*],
            });
        }
    }
}

fn conversion(field: &FieldModel, value: &TokenStream, error_name: &str) -> TokenStream {
    let invalid = quote! {
        |_| getargs_derive::Error::InvalidValue {
            name: #error_name,
            value: #value,
        }
    };
    let value_ty = &field.value_ty;
    match &field.value_parser {
        _ if field.value_enum => quote! {
            <#value_ty as getargs_derive::ValueEnum>::from_value(#value).ok_or(
                getargs_derive::Error::InvalidValue {
                    name: #error_name,
                    value: #value,
                },
            )?
        },
        Some(ValueParser::Function(path)) => quote! { #path(#value).map_err(#invalid)? },
        _ if field.is_str => quote! { #value },
        _ => quote! { core::str::FromStr::from_str(#value).map_err(#invalid)? },
    }
}

pub(super) fn convert(field: &FieldModel, value: TokenStream, error_name: &str) -> TokenStream {
    let check = possible_values_check(field, &value, error_name);
    let converted = conversion(field, &value, error_name);
    match &field.value_parser {
        Some(ValueParser::Range(range)) => {
            let value_type = &field.value_ty;
            let text = range_text(range);
            quote! {
                {
                    #check
                    let converted: #value_type = #converted;
                    if !(#range).contains(&converted) {
                        return Err(getargs_derive::Error::ValueOutOfRange {
                            name: #error_name,
                            value: #value,
                            range: #text,
                        });
                    }
                    converted
                }
            }
        }
        _ if field.possible_values.is_empty() => converted,
        _ => quote! {
            {
                #check
                #converted
            }
        },
    }
}
