use proc_macro2::TokenStream;
use quote::quote;

pub(super) struct Flavor {
    bytes: bool,
    pub(super) parse: &'static str,
    pub(super) parse_with_env: &'static str,
    pub(super) from_args: &'static str,
    pub(super) from_args_with_env: &'static str,
    pub(super) example: &'static str,
}

pub(super) const STR: Flavor = Flavor {
    bytes: false,
    parse: "parse",
    parse_with_env: "parse_with_env",
    from_args: "from_args",
    from_args_with_env: "from_args_with_env",
    example: "\n\n# Example\n\n```ignore\nlet params = MyStruct::from_args(std::env::args().skip(1))?;\n```",
};

pub(super) const BYTES: Flavor = Flavor {
    bytes: true,
    parse: "parse_bytes",
    parse_with_env: "parse_bytes_with_env",
    from_args: "from_bytes",
    from_args_with_env: "from_bytes_with_env",
    example: "",
};

const BYTES_NOTE: &str = "The arguments are raw bytes that need not be valid UTF-8: `&[u8]` fields borrow them\nas they are, and the other fields return\n[`Error::InvalidUtf8`](getargs_derive::Error::InvalidUtf8) for a value that is not valid\nUTF-8. See the [non-UTF-8 arguments guide](getargs_derive::guide::non_utf8).";

impl Flavor {
    pub(super) fn input(&self) -> TokenStream {
        if self.bytes {
            quote! { &'__getargs [u8] }
        } else {
            quote! { &'__getargs str }
        }
    }

    pub(super) fn doc(&self, text: &str) -> TokenStream {
        let text = if self.bytes {
            format!("{text}\n\n{BYTES_NOTE}")
        } else {
            text.to_owned()
        };
        let lines = text.lines().map(|line| format!(" {line}"));
        quote! { #(#[doc = #lines])* }
    }
}

pub(super) fn os_args_method() -> TokenStream {
    if !cfg!(feature = "std") {
        return quote! {};
    }
    quote! {
        /// Parse the arguments of the process, read with `std::env::args_os` without the program
        /// name, reading environment variable fallbacks from the process environment.
        ///
        /// The arguments are read once and leaked, so that the struct can borrow them for
        /// `'static`; later calls reuse them. On Unix they are parsed as raw bytes like
        /// [`from_bytes`](Self::from_bytes), so `&[u8]`, `&OsStr`, `&Path`, `OsString` and
        /// `PathBuf` fields accept any argument. On other platforms every argument must be valid
        /// UTF-8, or [`Error::InvalidUtf8`](getargs_derive::Error::InvalidUtf8) is returned. See
        /// the [non-UTF-8 arguments guide](getargs_derive::guide::non_utf8).
        pub fn from_args_os() -> core::result::Result<Self, getargs_derive::Error<'static>> {
            let args = getargs_derive::__private::args_os()?;
            let raw = core::cell::Cell::new(core::option::Option::None);
            let mut options = getargs::Options::new(getargs_derive::__RawArgs::new(
                args.iter().copied(),
                &raw,
            ));
            Self::__parse(&mut options, core::option::Option::Some(&raw), getargs_derive::__env)
        }
    }
}
