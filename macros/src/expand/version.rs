use crate::help::command_name;
use crate::model::{ContainerModel, Version};
use proc_macro2::TokenStream;
use quote::quote;

fn version_text(container: &ContainerModel) -> Option<(TokenStream, TokenStream)> {
    let version = match container.version.as_ref()? {
        Version::Package => quote! { ::core::env!("CARGO_PKG_VERSION") },
        Version::Literal(version) => quote! { #version },
    };
    let name = command_name(container);
    let display = quote! { ::core::concat!(#name, " ", #version) };
    Some((version, display))
}

pub(super) fn version_tokens(container: &ContainerModel) -> (TokenStream, TokenStream) {
    match version_text(container) {
        Some((version, display)) => (
            quote! {
                /// Version string set with `#[command(version)]`.
                pub const VERSION: &'static str = #version;

                /// Version line returned in [`Error::Version`](getargs_derive::Error::Version) when `-V` or `--version` is given.
                ///
                /// It is the command name followed by a space and [`VERSION`](Self::VERSION).
                pub const VERSION_TEXT: &'static str = #display;
            },
            quote! {
                getargs::Arg::Short('V') | getargs::Arg::Long("version") => {
                    return Err(getargs_derive::Error::Version(Self::VERSION_TEXT));
                }
            },
        ),
        None => (quote! {}, quote! {}),
    }
}
