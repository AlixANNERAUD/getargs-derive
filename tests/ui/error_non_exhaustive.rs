use getargs_derive::Error;

fn describe(error: Error<'_>) -> &'static str {
    match error {
        Error::MissingArgument(_) => "missing argument",
        Error::UnknownOption { .. } => "unknown option",
        Error::UnexpectedArgument(_) => "unexpected argument",
        Error::UnexpectedValue(_) => "unexpected value",
        Error::MissingOptionValue(_) => "missing option value",
        Error::ArgumentConflict { .. } => "argument conflict",
        Error::MissingRequiredArgument { .. } => "missing required argument",
        Error::MissingOneOf(_) => "missing one of",
        Error::MissingSubcommand(_) => "missing subcommand",
        Error::UnknownSubcommand { .. } => "unknown subcommand",
        Error::InvalidArguments => "invalid arguments",
        Error::InvalidValue { .. } => "invalid value",
        Error::InvalidPossibleValue { .. } => "invalid possible value",
        Error::ValueOutOfRange { .. } => "value out of range",
        Error::TooManyValues { .. } => "too many values",
        Error::TooFewValues { .. } => "too few values",
        Error::InvalidUtf8 { .. } => "invalid UTF-8",
        Error::Help(_) => "help",
        Error::Version(_) => "version",
    }
}

fn main() {
    let _ = describe(Error::InvalidArguments);
}
