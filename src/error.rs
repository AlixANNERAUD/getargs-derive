#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A required positional or optional argument was missing.
    MissingPositionalArgument(&'static str),
    /// An option was formatted incorrectly, or its value failed to parse.
    InvalidOption,
    /// The user provided more positional arguments than the struct accommodates.
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
