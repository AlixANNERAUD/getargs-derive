/// Errors that can occur when parsing CLI arguments with [`GetArgs`](crate::GetArgs).
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A required positional or named argument was not provided on the command line.
    ///
    /// The string is the name of the missing argument (e.g. `"input"` for a positional,
    /// or `"verbose"` for a named option).
    MissingPositionalArgument(&'static str),

    /// An unrecognised option was given on the command line.
    ///
    /// This includes both unknown short flags (`-z`) and long flags (`--bogus`).
    UnknownOption,

    /// A named option was given but no value was provided.
    ///
    /// The string is the option name (e.g. `"output"` or `"-j"`).
    /// For example, `--output` at the end of the command line without a following value.
    MissingOptionValue(&'static str),

    /// A value could not be parsed into the target field type.
    ///
    /// The string is the field or option name (e.g. `"num"` or `"port"`).
    /// For example, providing `"abc"` for a `u32` field.
    ParseError(&'static str),

    /// More positional arguments were provided than the struct defines.
    InvalidNumberOfArguments,

    /// Help was requested with `-h` or `--help`.
    ///
    /// This is not a parse failure: the string is the generated help text (the struct's `HELP`
    /// constant). Applications should print it to standard output and exit successfully.
    /// It is never returned when help is disabled with `#[command(disable_help_flag)]`.
    Help(&'static str),
}

impl Error {
    /// Returns `true` if this is [`Error::Help`], i.e. the user asked for help rather than
    /// passing invalid arguments.
    #[must_use]
    pub fn is_help(&self) -> bool {
        matches!(self, Error::Help(_))
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::MissingPositionalArgument(arg) => {
                write!(f, "Missing required argument: '{}'", arg)
            }
            Error::UnknownOption => write!(f, "Unknown option"),
            Error::MissingOptionValue(name) => {
                write!(f, "Option '{}' requires a value", name)
            }
            Error::ParseError(name) => write!(f, "Failed to parse value for '{}'", name),
            Error::InvalidNumberOfArguments => {
                write!(f, "Invalid number of arguments provided")
            }
            Error::Help(help) => f.write_str(help),
        }
    }
}

impl core::error::Error for Error {}
