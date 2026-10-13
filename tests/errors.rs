#![allow(dead_code)]

use getargs_derive::{Error, GetArgs};

#[derive(Debug, GetArgs)]
struct RequiredOption<'a> {
    #[arg(long)]
    name: &'a str,
}

#[derive(Debug, GetArgs)]
struct OnePositional<'a> {
    first: &'a str,
}

#[derive(Debug, GetArgs)]
struct ParseInt {
    #[arg(short = 'n')]
    num: u32,
}

#[derive(Debug, GetArgs)]
struct NoOptions<'a> {
    input: &'a str,
}

#[test]
fn missing_required_positional() {
    let args: [&str; 0] = [];
    let mut options = getargs::Options::new(args.into_iter());
    let err = OnePositional::parse(&mut options).unwrap_err();
    assert_eq!(err, Error::MissingPositionalArgument("first"));
}

#[test]
fn missing_required_option() {
    let args: [&str; 0] = [];
    let mut options = getargs::Options::new(args.into_iter());
    let err = RequiredOption::parse(&mut options).unwrap_err();
    assert_eq!(err, Error::MissingPositionalArgument("name"));
}

#[test]
fn too_many_positionals() {
    let args = ["one", "two"];
    let mut options = getargs::Options::new(args.into_iter());
    let err = OnePositional::parse(&mut options).unwrap_err();
    assert_eq!(err, Error::InvalidNumberOfArguments);
}

#[test]
fn invalid_option_parse_failure() {
    let args = ["-n", "not-a-number"];
    let mut options = getargs::Options::new(args.into_iter());
    let err = ParseInt::parse(&mut options).unwrap_err();
    assert_eq!(err, Error::ParseError("num"));
}

#[test]
fn unknown_flag() {
    let args = ["input", "--bogus"];
    let mut options = getargs::Options::new(args.into_iter());
    let err = NoOptions::parse(&mut options).unwrap_err();
    assert_eq!(err, Error::UnknownOption);
}

#[test]
fn unknown_short_flag() {
    let args = ["input", "-z"];
    let mut options = getargs::Options::new(args.into_iter());
    let err = NoOptions::parse(&mut options).unwrap_err();
    assert_eq!(err, Error::UnknownOption);
}

#[test]
fn too_many_positionals_when_no_positionals_defined() {
    #[derive(Debug, GetArgs)]
    struct FlagsOnly {
        #[arg(long)]
        verbose: bool,
    }

    let args = ["extra"];
    let mut options = getargs::Options::new(args.into_iter());
    let err = FlagsOnly::parse(&mut options).unwrap_err();
    assert_eq!(err, Error::InvalidNumberOfArguments);
}

#[test]
fn is_help_false_for_other_errors() {
    assert!(!Error::UnknownOption.is_help());
    assert!(!Error::MissingPositionalArgument("x").is_help());
}

#[test]
fn error_display_missing_positional() {
    assert_eq!(
        format!("{}", Error::MissingPositionalArgument("output")),
        "Missing required argument: 'output'"
    );
}

#[test]
fn error_display_unknown_option() {
    assert_eq!(format!("{}", Error::UnknownOption), "Unknown option");
}

#[test]
fn error_display_missing_option_value() {
    assert_eq!(
        format!("{}", Error::MissingOptionValue("output")),
        "Option 'output' requires a value"
    );
}

#[test]
fn error_display_parse_error() {
    assert_eq!(
        format!("{}", Error::ParseError("port")),
        "Failed to parse value for 'port'"
    );
}

#[test]
fn error_display_invalid_number() {
    assert_eq!(
        format!("{}", Error::InvalidNumberOfArguments),
        "Invalid number of arguments provided"
    );
}

#[test]
fn error_debug_and_clone_and_eq() {
    let a = Error::MissingPositionalArgument("x");
    let b = Error::MissingPositionalArgument("x");
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
}
