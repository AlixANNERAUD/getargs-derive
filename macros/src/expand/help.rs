use crate::help::help_text;
use crate::model::{ContainerModel, FieldModel};
use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn help_items(container: &ContainerModel, fields: &[FieldModel]) -> TokenStream {
    let help = help_text(container, fields);
    quote! {
        /// Help text generated at compile time from the struct, its fields and their documentation.
        ///
        /// Returned in [`Error::Help`](getargs_derive::Error::Help) when `-h` or `--help` is given.
        pub const HELP: &'static str = #help;
    }
}

pub(super) fn help_handler(container: &ContainerModel) -> TokenStream {
    if container.disable_help_flag {
        quote! {}
    } else {
        quote! {
            getargs::Arg::Short('h') | getargs::Arg::Long("help") => {
                return Err(getargs_derive::Error::Help(Self::HELP));
            }
        }
    }
}
