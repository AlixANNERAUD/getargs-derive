/// Errors that can occur when parsing CLI arguments with [`GetArgs`](crate::GetArgs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A required positional or named argument was not provided on the command line.
    ///
    /// The string is the name of the missing argument (e.g. `"input"` for a positional,
    /// or `"verbose"` for a named option).
    MissingPositionalArgument(&'static str),

    /// An unrecognised option was given, or a value could not be parsed into the target field type.
    ///
    /// This covers both unknown flags like `--bogus` and parse failures such as
    /// providing `"abc"` for a `u32` field.
    InvalidOption,

    /// More positional arguments were provided than the struct defines.
    InvalidNumberOfArguments,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::MissingPositionalArgument(arg) => {
                write!(f, "Missing required argument: '{}'", arg)
            }
            Error::InvalidOption => {
                write!(f, "Invalid option or failed to parse option value")
            }
            Error::InvalidNumberOfArguments => {
                write!(f, "Invalid number of arguments provided")
            }
        }
    }
}

impl core::error::Error for Error {}
