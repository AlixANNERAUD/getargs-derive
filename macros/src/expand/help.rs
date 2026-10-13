use crate::help::{Mode, has_long_help, help_text};
use crate::model::{ContainerModel, FieldModel};
use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn help_items(container: &ContainerModel, fields: &[FieldModel]) -> TokenStream {
    let help = help_text(container, fields, Mode::Short);
    let long_help = if has_long_help(container, fields) {
        help_text(container, fields, Mode::Long)
    } else {
        quote! { Self::HELP }
    };
    quote! {
        /// Help text generated at compile time from the struct, its fields and their documentation.
        ///
        /// Returned in [`Error::Help`](getargs_derive::Error::Help) when `-h` is given.
        pub const HELP: &'static str = #help;

        /// Long help text generated at compile time, using the full doc comments, `long_about`
        /// and `long_help`.
        ///
        /// Returned in [`Error::Help`](getargs_derive::Error::Help) when `--help` is given. It is
        /// the same string as [`HELP`](Self::HELP) when no long help text is defined.
        pub const LONG_HELP: &'static str = #long_help;
    }
}

pub(super) fn help_handler(container: &ContainerModel) -> TokenStream {
    if container.disable_help_flag {
        quote! {}
    } else {
        quote! {
            getargs::Arg::Short('h') => {
                return Err(getargs_derive::Error::Help(Self::HELP));
            }
            getargs::Arg::Long("help") => {
                return Err(getargs_derive::Error::Help(Self::LONG_HELP));
            }
        }
    }
}
