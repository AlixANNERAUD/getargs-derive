use super::flatten::static_self;
use crate::help::{Mode, after_text, command_name, has_long_help, help_text, usage_prefix};
use crate::model::{ContainerModel, Model};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

fn merged_help(
    input: &DeriveInput,
    container: &ContainerModel,
    mode: Mode,
    has_long: &TokenStream,
) -> TokenStream {
    let args = static_self(input);
    let long = mode == Mode::Long;
    let mut reserved = Vec::new();
    if !container.disable_help_flag {
        reserved.push(quote! {
            getargs_derive::__private::HelpEntry::reserved(
                "-h, --help",
                getargs_derive::__private::help_flag(false, #has_long),
                getargs_derive::__private::help_flag(true, #has_long),
            )
        });
    }
    if container.version.is_some() {
        reserved.push(quote! {
            getargs_derive::__private::HelpEntry::reserved(
                "-V, --version",
                "Print version",
                "Print version",
            )
        });
    }
    let prefix = usage_prefix(container, mode);
    let suffix = after_text(container);
    let name = command_name(container);
    let term_width = container.max_term_width.unwrap_or_default();
    quote! {
        {
            const HELP: getargs_derive::__private::Help<'static> = getargs_derive::__private::Help {
                prefix: ::core::concat!(#prefix, #name),
                suffix: #suffix,
                entries: #args::ENTRIES,
                reserved: &[#(#reserved),*],
                long: #long,
                term_width: #term_width,
            };
            const LENGTH: usize = getargs_derive::__private::help_len(&HELP);
            const BYTES: [u8; LENGTH] = getargs_derive::__private::help(&HELP);
            getargs_derive::__private::utf8(&BYTES)
        }
    }
}

pub(super) fn help_items(input: &DeriveInput, model: &Model) -> TokenStream {
    let container = &model.container;
    let (help, long_help) = if model.flattened.is_empty() {
        let help = help_text(container, &model.fields, Mode::Short);
        let long_help = if has_long_help(container, &model.fields) {
            help_text(container, &model.fields, Mode::Long)
        } else {
            quote! { Self::HELP }
        };
        (help, long_help)
    } else {
        let args = static_self(input);
        let long_about = container.long_about.is_some();
        let has_long = quote! { (#long_about || #args::HAS_LONG_HELP) };
        let help = merged_help(input, container, Mode::Short, &has_long);
        let long = merged_help(input, container, Mode::Long, &has_long);
        (help, quote! { if #has_long { #long } else { Self::HELP } })
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
