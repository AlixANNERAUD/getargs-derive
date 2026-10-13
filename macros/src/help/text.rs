use proc_macro2::TokenStream;
use quote::quote;

pub(crate) enum Segment {
    Text(String),
    Str(TokenStream),
    List(TokenStream),
}

#[derive(Default)]
pub(crate) struct Text(Vec<Segment>);

impl Text {
    pub(crate) fn push_str(&mut self, text: &str) {
        match self.0.last_mut() {
            Some(Segment::Text(last)) => last.push_str(text),
            _ => self.0.push(Segment::Text(text.to_owned())),
        }
    }

    pub(crate) fn push(&mut self, segment: Segment) {
        match segment {
            Segment::Text(text) => self.push_str(&text),
            other => self.0.push(other),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn segments(self) -> Vec<Segment> {
        self.0
    }

    pub(crate) fn into_tokens(self, head: TokenStream) -> TokenStream {
        match self.0.as_slice() {
            [] => quote! { #head },
            [Segment::Text(text)] => quote! { ::core::concat!(#head, #text) },
            _ => {
                let parts = self.0.iter().map(|segment| match segment {
                    Segment::Text(text) => quote! { getargs_derive::__HelpPart::Text(#text) },
                    Segment::Str(tokens) => quote! { getargs_derive::__HelpPart::Text(#tokens) },
                    Segment::List(tokens) => quote! { getargs_derive::__HelpPart::List(#tokens) },
                });
                quote! {
                    {
                        const __PARTS: &[getargs_derive::__HelpPart] = &[
                            getargs_derive::__HelpPart::Text(#head),
                            #(#parts),*
                        ];
                        const __LENGTH: usize = getargs_derive::__help_len(__PARTS);
                        const __BYTES: [u8; __LENGTH] = getargs_derive::__help_bytes::<__LENGTH>(__PARTS);
                        getargs_derive::__help_str(&__BYTES)
                    }
                }
            }
        }
    }
}
