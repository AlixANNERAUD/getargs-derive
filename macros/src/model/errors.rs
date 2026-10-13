#[derive(Default)]
pub(super) struct Errors(pub(super) Option<syn::Error>);

impl Errors {
    pub(super) fn push(&mut self, error: syn::Error) {
        match &mut self.0 {
            Some(existing) => existing.combine(error),
            None => self.0 = Some(error),
        }
    }

    pub(super) fn finish<T>(self, value: T) -> syn::Result<T> {
        match self.0 {
            Some(error) => Err(error),
            None => Ok(value),
        }
    }
}
