#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "reading `env` fallbacks from the process environment requires the `std` feature of `getargs-derive`",
    label = "`parse` and `from_args` read the process environment",
    note = "enable the `std` feature of `getargs-derive`, or call `parse_with_env` or `from_args_with_env` with a lookup function"
)]
pub trait __EnvSupport {}

#[cfg(feature = "std")]
impl<T: ?Sized> __EnvSupport for T {}

#[doc(hidden)]
#[cfg(feature = "std")]
pub fn __env<'a>(name: &str) -> Option<&'a str> {
    let value = std::env::var(name).ok()?;
    Some(std::boxed::Box::leak(value.into_boxed_str()))
}

#[doc(hidden)]
#[cfg(not(feature = "std"))]
pub fn __env<'a>(_name: &str) -> Option<&'a str> {
    None
}
