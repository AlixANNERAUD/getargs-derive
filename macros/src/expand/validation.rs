use crate::help::expr_text;
use crate::model::types::ValueKind;
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
        _ if field.value_kind == ValueKind::Str => quote! { #value },
        _ => quote! { core::str::FromStr::from_str(#value).map_err(#invalid)? },
    }
}

fn raw_conversion(
    field: &FieldModel,
    value: &TokenStream,
    error_name: &str,
) -> Option<TokenStream> {
    if field.value_enum || field.value_parser.is_some() {
        return None;
    }
    let value_ty = &field.value_ty;
    Some(match field.value_kind {
        ValueKind::Bytes => quote! { getargs_derive::__private::bytes(#value) },
        ValueKind::OsStr => quote! { getargs_derive::__private::os_str(#value, #error_name)? },
        ValueKind::Path => quote! { getargs_derive::__private::path(#value, #error_name)? },
        ValueKind::OsOwned => quote! {
            <#value_ty as core::convert::From<&getargs_derive::__private::OsStr>>::from(
                getargs_derive::__private::os_str(#value, #error_name)?,
            )
        },
        ValueKind::Str | ValueKind::Parsed => return None,
    })
}

pub(super) fn convert(field: &FieldModel, value: TokenStream, error_name: &str) -> TokenStream {
    let text = quote! { getargs_derive::__private::text(#value, #error_name)? };
    let Some(raw) = raw_conversion(field, &value, error_name) else {
        let converted = convert_text(field, quote! { __text }, error_name);
        return quote! {
            {
                let __text = #text;
                #converted
            }
        };
    };
    if field.possible_values.is_empty() {
        return raw;
    }
    let check = possible_values_check(field, &quote! { __text }, error_name);
    quote! {
        {
            let __text = #text;
            #check
            #raw
        }
    }
}

fn convert_text(field: &FieldModel, value: TokenStream, error_name: &str) -> TokenStream {
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
