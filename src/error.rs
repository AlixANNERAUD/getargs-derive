/// An option as it was written on the command line, without its leading dashes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opt<'a> {
    /// A short option, like `-z` (stored as `'z'`).
    Short(char),
    /// A long option, like `--bogus` (stored as `"bogus"`).
    Long(&'a str),
}

impl core::fmt::Display for Opt<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Opt::Short(short) => write!(f, "-{short}"),
            Opt::Long(long) => write!(f, "--{long}"),
        }
    }
}

/// Errors that can occur when parsing CLI arguments with [`GetArgs`](crate::GetArgs).
///
/// The lifetime `'a` is the lifetime of the parsed arguments: variants describing what the
/// user typed borrow it from the command line instead of copying it, so no allocation is needed.
///
/// Names of the struct's own arguments (`&'static str`) are written as in the generated help:
/// `<INPUT>` for a positional argument, `--output <OUTPUT>` for an option taking a value
/// (`-o <OUTPUT>` when it has no long name).
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<'a> {
    /// A required positional argument or named option was not provided on the command line.
    ///
    /// The string is the argument name, e.g. `"<INPUT>"` or `"--output <OUTPUT>"`.
    MissingArgument(&'static str),

    /// An option that the struct does not define was given, e.g. `--bogus` or `-z`.
    UnknownOption {
        /// The unknown option, as written by the user.
        option: Opt<'a>,
        /// The closest long option name (without `--`) when an unknown long option looks like a
        /// typo of a known one, e.g. `Some("verbose")` for `--verbos`.
        suggestion: Option<&'static str>,
    },

    /// A positional argument was given but the struct expects no more of them.
    ///
    /// The string is the extra argument, as written by the user.
    UnexpectedArgument(&'a str),

    /// A value was attached to an option that does not take one, e.g. `--verbose=yes` for a flag.
    ///
    /// The option is the one written by the user.
    UnexpectedValue(Opt<'a>),

    /// An option taking a value was given without one, e.g. `--output` at the end of the
    /// command line.
    ///
    /// The string is the option name, e.g. `"--output <OUTPUT>"`.
    MissingOptionValue(&'static str),

    /// Two arguments that cannot be used together were both given, because of `conflicts_with`
    /// or because they belong to the same group.
    ArgumentConflict {
        /// The argument declaring the conflict (or the first member of the group), e.g.
        /// `"--quiet"`.
        name: &'static str,
        /// The other argument, e.g. `"--verbose"`.
        other: &'static str,
    },

    /// An argument was given without another argument it `requires`.
    MissingRequiredArgument {
        /// The missing argument, e.g. `"--output <OUTPUT>"`.
        name: &'static str,
        /// The given argument that requires it, e.g. `"--compress"`.
        required_by: &'static str,
    },

    /// None of a set of arguments was given although one of them is required, because of
    /// `required_unless_present` or a `required` group.
    ///
    /// The slice lists the arguments, e.g. `["--config <CONFIG>", "--stdin"]`.
    MissingOneOf(&'static [&'static str]),

    /// The underlying `getargs` parser reported an error that this crate does not recognise.
    ///
    /// `getargs` may add error kinds in a compatible release; they are reported as this variant
    /// instead of panicking. It is never returned with the `getargs` 0.5 releases known to this
    /// crate.
    InvalidArguments,

    /// A value could not be parsed into the target field type, e.g. `abc` for a `u32` field.
    ///
    /// This is also returned when a `default_value` string does not parse.
    InvalidValue {
        /// The argument name, e.g. `"--port <PORT>"` or `"<COUNT>"`.
        name: &'static str,
        /// The value that failed to parse.
        value: &'a str,
    },

    /// A value is not one of the argument's `possible_values`, e.g. `turbo` for
    /// `possible_values = ["fast", "slow"]`.
    ///
    /// This is also returned when a `default_value` string is not one of them.
    InvalidPossibleValue {
        /// The argument name, e.g. `"--mode <MODE>"`.
        name: &'static str,
        /// The value that was given.
        value: &'a str,
        /// The accepted values, in declaration order, e.g. `["fast", "slow"]`.
        possible_values: &'static [&'static str],
    },

    /// A value parsed into the field type but is outside the argument's `value_parser` range,
    /// e.g. `0` for `value_parser = 1..=64`.
    ///
    /// This is also returned when a `default_value` string is outside the range.
    ValueOutOfRange {
        /// The argument name, e.g. `"--jobs <JOBS>"`.
        name: &'static str,
        /// The value that was given.
        value: &'a str,
        /// The range as written in the attribute, e.g. `"1..=64"`.
        range: &'static str,
    },

    /// Help was requested with `-h` or `--help`.
    ///
    /// This is not a parse failure: the string is the generated help text (the struct's `HELP`
    /// constant). Applications should print it to standard output and exit successfully.
    /// It is never returned when help is disabled with `#[command(disable_help_flag)]`.
    Help(&'static str),

    /// The version was requested with `-V` or `--version`.
    ///
    /// This is not a parse failure: the string is the version line, the command name followed by
    /// the version (the struct's `VERSION_TEXT` constant). Applications should print it to standard
    /// output and exit successfully. It is only returned when `#[command(version)]` is set.
    Version(&'static str),
}

impl Error<'_> {
    /// Returns `true` if this is [`Error::Help`], i.e. the user asked for help rather than
    /// passing invalid arguments.
    #[must_use]
    pub fn is_help(&self) -> bool {
        matches!(self, Error::Help(_))
    }

    /// Returns `true` if this is [`Error::Version`], i.e. the user asked for the version rather
    /// than passing invalid arguments.
    #[must_use]
    pub fn is_version(&self) -> bool {
        matches!(self, Error::Version(_))
    }
}

impl core::fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::MissingArgument(name) => {
                write!(
                    f,
                    "the following required argument was not provided: {name}"
                )
            }
            Error::UnknownOption { option, suggestion } => {
                write!(f, "unexpected argument '{option}' found")?;
                match suggestion {
                    Some(suggestion) => {
                        write!(f, "\n\n  tip: a similar argument exists: '--{suggestion}'")
                    }
                    None => Ok(()),
                }
            }
            Error::UnexpectedArgument(argument) => {
                write!(f, "unexpected argument '{argument}' found")
            }
            Error::UnexpectedValue(option) => {
                write!(
                    f,
                    "unexpected value for '{option}': it does not take a value"
                )
            }
            Error::ArgumentConflict { name, other } => {
                write!(f, "the argument '{name}' cannot be used with '{other}'")
            }
            Error::MissingRequiredArgument { name, required_by } => {
                write!(
                    f,
                    "the following required argument was not provided: {name} (required by '{required_by}')"
                )
            }
            Error::MissingOneOf(names) => {
                f.write_str("the following required arguments were not provided: <")?;
                for (index, name) in names.iter().enumerate() {
                    if index > 0 {
                        f.write_str("|")?;
                    }
                    f.write_str(name)?;
                }
                f.write_str(">")
            }
            Error::InvalidArguments => f.write_str("the arguments could not be parsed"),
            Error::MissingOptionValue(name) => {
                write!(f, "a value is required for '{name}' but none was supplied")
            }
            Error::InvalidValue { name, value } => {
                write!(f, "invalid value '{value}' for '{name}'")
            }
            Error::InvalidPossibleValue {
                name,
                value,
                possible_values,
            } => {
                write!(
                    f,
                    "invalid value '{value}' for '{name}'\n  [possible values: "
                )?;
                for (index, possible) in possible_values.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    if possible.is_empty() || possible.contains(char::is_whitespace) {
                        write!(f, "{possible:?}")?;
                    } else {
                        f.write_str(possible)?;
                    }
                }
                f.write_str("]")
            }
            Error::ValueOutOfRange { name, value, range } => {
                write!(
                    f,
                    "invalid value '{value}' for '{name}': {value} is not in {range}"
                )
            }
            Error::Help(help) => f.write_str(help),
            Error::Version(version) => f.write_str(version),
        }
    }
}

impl core::error::Error for Error<'_> {}
