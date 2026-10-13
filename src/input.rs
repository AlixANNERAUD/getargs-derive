use crate::{Error, Opt};
use core::fmt;

/// Displays bytes that may not be valid UTF-8, without allocating.
///
/// Valid UTF-8 sequences are written as they are and every byte of an invalid sequence is written
/// as a `\xNN` escape, e.g. `caf\xE9` for the Latin-1 encoding of `café`. It is used by the
/// [`Display`](fmt::Display) implementation of [`Error::InvalidUtf8`].
///
/// ```
/// use getargs_derive::Lossy;
///
/// assert_eq!(Lossy(b"caf\xE9.txt").to_string(), "caf\\xE9.txt");
/// assert_eq!(Lossy("café".as_bytes()).to_string(), "café");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lossy<'a>(pub &'a [u8]);

impl fmt::Display for Lossy<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for chunk in self.0.utf8_chunks() {
            f.write_str(chunk.valid())?;
            for byte in chunk.invalid() {
                write!(f, "\\x{byte:02X}")?;
            }
        }
        Ok(())
    }
}

/// An argument type accepted by the generated parsers: `&str` or `&[u8]`.
pub trait Input<'a>: getargs::Argument + 'a {
    /// The raw bytes of the argument.
    fn bytes(self) -> &'a [u8];

    /// The argument as a string, or its bytes when they are not valid UTF-8.
    fn text(self) -> Result<&'a str, &'a [u8]>;

    /// A short option name as a character.
    fn short(short: Self::ShortOpt) -> char;

    /// Splits the argument at the first `delimiter`.
    fn split_once(self, delimiter: char) -> Option<(Self, Self)>;
}

impl<'a, 'b: 'a> Input<'a> for &'b str {
    fn bytes(self) -> &'a [u8] {
        self.as_bytes()
    }

    fn text(self) -> Result<&'a str, &'a [u8]> {
        Ok(self)
    }

    fn short(short: char) -> char {
        short
    }

    fn split_once(self, delimiter: char) -> Option<(Self, Self)> {
        str::split_once(self, delimiter)
    }
}

impl<'a, 'b: 'a> Input<'a> for &'b [u8] {
    fn bytes(self) -> &'a [u8] {
        self
    }

    fn text(self) -> Result<&'a str, &'a [u8]> {
        core::str::from_utf8(self).map_err(|_| self)
    }

    fn short(short: u8) -> char {
        if short.is_ascii() {
            char::from(short)
        } else {
            char::REPLACEMENT_CHARACTER
        }
    }

    fn split_once(self, delimiter: char) -> Option<(Self, Self)> {
        let mut buffer = [0; 4];
        let delimiter = delimiter.encode_utf8(&mut buffer).as_bytes();
        let index = self
            .windows(delimiter.len())
            .position(|window| window == delimiter)?;
        Some((&self[..index], &self[index + delimiter.len()..]))
    }
}

/// An argument read from the command line, with option names converted to UTF-8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arg<'a, A> {
    /// A short option, like `-v`.
    Short(char),
    /// A long option, like `--verbose`, without its value.
    Long(&'a str),
    /// A positional argument, or every argument after `--`.
    Positional(A),
}

/// Converts a value to a string for the argument `name`.
pub fn text<'a, V: Input<'a>>(value: V, name: &'static str) -> Result<&'a str, Error<'a>> {
    value.text().map_err(|value| Error::InvalidUtf8 {
        name: Some(name),
        value,
    })
}

/// Converts an argument that matches no field to a string.
pub fn unnamed<'a, V: Input<'a>>(value: V) -> Result<&'a str, Error<'a>> {
    value
        .text()
        .map_err(|value| Error::InvalidUtf8 { name: None, value })
}

/// The raw bytes of a value, for `&[u8]` fields.
pub fn bytes<'a, V: Input<'a>>(value: V) -> &'a [u8] {
    value.bytes()
}

/// Converts an argument read by `getargs`.
pub fn arg<'a, A: Input<'a>>(argument: getargs::Arg<A>) -> Result<Arg<'a, A>, Error<'a>> {
    Ok(match argument {
        getargs::Arg::Short(short) => Arg::Short(A::short(short)),
        getargs::Arg::Long(long) => Arg::Long(unnamed(long)?),
        getargs::Arg::Positional(value) => Arg::Positional(value),
    })
}

/// Converts an option named by `getargs`.
pub fn opt<'a, A: Input<'a>>(option: getargs::Opt<A>) -> Result<Opt<'a>, Error<'a>> {
    Ok(match option {
        getargs::Opt::Short(short) => Opt::Short(A::short(short)),
        getargs::Opt::Long(long) => Opt::Long(unnamed(long)?),
    })
}

/// The error for a positional argument given where the struct expects no more of them.
pub fn unexpected_argument<'a, V: Input<'a>>(value: V) -> Error<'a> {
    match value.text() {
        Ok(value) => Error::UnexpectedArgument(value),
        Err(value) => Error::InvalidUtf8 { name: None, value },
    }
}

/// The error for a positional argument given where a subcommand is expected, suggesting one of
/// `names`.
pub fn unknown_subcommand<'a, V: Input<'a>>(value: V, names: &[&[&'static str]]) -> Error<'a> {
    match value.text() {
        Ok(name) => Error::UnknownSubcommand {
            name,
            suggestion: crate::suggest::suggest(name, names),
        },
        Err(value) => Error::InvalidUtf8 { name: None, value },
    }
}
