use crate::model::FieldModel;
use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn allows_negative_numbers(field: &FieldModel) -> TokenStream {
    if field.parsing.allow_negative_numbers {
        quote! { true }
    } else {
        quote! { __context.negative_numbers }
    }
}

pub(super) fn option_value(field: &FieldModel, missing: TokenStream) -> TokenStream {
    let next = quote! { __options.value().map_err(|_| #missing)? };
    if field.parsing.allow_hyphen_values {
        return quote! { #next };
    }
    let allow_negative_numbers = allows_negative_numbers(field);
    quote! {
        match __options.value_opt() {
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

pub(super) fn negative_positional(
    field: &FieldModel,
    slot: &TokenStream,
    earlier: &TokenStream,
    parsed: &TokenStream,
) -> TokenStream {
    let allow_negative_numbers = allows_negative_numbers(field);
    let store = if field.parsing.last {
        quote! { return Err(getargs_derive::Error::UnexpectedArgument(value)); }
    } else {
        quote! {
            #slot = core::option::Option::Some(#parsed);
            return core::result::Result::Ok(true);
        }
    };
    quote! {
        if let getargs::Arg::Short(short) = __argument {
            if #allow_negative_numbers
                && !__context.blocked
                && #earlier
                && #slot.is_none()
            {
                if let core::option::Option::Some(value) =
                    getargs_derive::__negative_number(__context.raw, short)
                {
                    let _ = __options.value_opt();
                    #store
                }
            }
        }
    }
}

pub(super) fn positional(
    field: &FieldModel,
    slot: &TokenStream,
    parsed: &TokenStream,
) -> TokenStream {
    let accepts = if field.parsing.last {
        quote! { __options.opts_ended() }
    } else {
        quote! { !(__last && __options.opts_ended()) }
    };
    quote! {
        if let getargs::Arg::Positional(value) = __argument {
            if #slot.is_none() && #accepts {
                #slot = core::option::Option::Some(#parsed);
                return core::result::Result::Ok(true);
            }
        }
    }
}
