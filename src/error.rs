/// Errors that can occur when parsing CLI arguments with [`GetArgs`](crate::GetArgs).
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A required positional or named argument was not provided on the command line.
    ///
    /// The string is the name of the missing argument (e.g. `"input"` for a positional,
    /// or `"verbose"` for a named option).
    MissingPositionalArgument(&'static str),

    /// An option was unrecognised, missing a value, or its value could not be parsed.
    ///
    /// The string describes what went wrong (e.g. `"unknown option"`, `"option requires a value"`,
    /// or `"failed to parse value for --port"`).
    InvalidOption(&'static str),

    /// More positional arguments were provided than the struct defines.
    InvalidNumberOfArguments,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::MissingPositionalArgument(arg) => {
                write!(f, "Missing required argument: '{}'", arg)
            }
            Error::InvalidOption(msg) => {
                write!(f, "{}", msg)
            }
            Error::InvalidNumberOfArguments => {
                write!(f, "Invalid number of arguments provided")
            }
        }
    }
}

impl core::error::Error for Error {}
