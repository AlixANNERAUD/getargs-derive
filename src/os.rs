use crate::Error;
use crate::input::Input;
use std::boxed::Box;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::sync::OnceLock;
use std::vec::Vec;

/// The argument type of `from_args_os`: raw bytes on Unix, strings elsewhere.
#[cfg(unix)]
pub type OsArg = &'static [u8];

/// The argument type of `from_args_os`: raw bytes on Unix, strings elsewhere.
#[cfg(not(unix))]
pub type OsArg = &'static str;

static ARGS: OnceLock<Result<&'static [OsArg], Error<'static>>> = OnceLock::new();

fn leak<T>(items: Vec<T>) -> &'static [T] {
    Box::leak(items.into_boxed_slice())
}

#[cfg(unix)]
fn convert(arguments: &'static [OsString]) -> Result<&'static [OsArg], Error<'static>> {
    use std::os::unix::ffi::OsStrExt;
    Ok(leak(
        arguments
            .iter()
            .map(|argument| argument.as_bytes())
            .collect(),
    ))
}

#[cfg(not(unix))]
fn convert(arguments: &'static [OsString]) -> Result<&'static [OsArg], Error<'static>> {
    arguments
        .iter()
        .map(|argument| {
            argument.to_str().ok_or(Error::InvalidUtf8 {
                name: None,
                value: argument.as_encoded_bytes(),
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(leak)
}

/// The arguments of the process without the program name, read once with
/// [`std::env::args_os`] and leaked so that the parsed struct can borrow them for `'static`.
pub fn args_os() -> Result<&'static [OsArg], Error<'static>> {
    *ARGS.get_or_init(|| convert(leak(std::env::args_os().skip(1).collect())))
}

/// Converts a value to an [`OsStr`] for the argument `name`: losslessly on Unix, requiring UTF-8
/// elsewhere.
pub fn os_str<'a, V: Input<'a>>(value: V, name: &'static str) -> Result<&'a OsStr, Error<'a>> {
    #[cfg(unix)]
    {
        let _ = name;
        Ok(<OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(
            value.bytes(),
        ))
    }
    #[cfg(not(unix))]
    {
        crate::input::text(value, name).map(OsStr::new)
    }
}

/// Converts a value to a [`Path`] for the argument `name`, like [`os_str`].
pub fn path<'a, V: Input<'a>>(value: V, name: &'static str) -> Result<&'a Path, Error<'a>> {
    os_str(value, name).map(Path::new)
}
