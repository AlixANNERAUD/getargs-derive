#![allow(dead_code)]

use getargs_derive::{__suggest, Error, GetArgs, Opt};

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

#[derive(Debug, GetArgs)]
struct InferredNames {
    #[arg(short, long)]
    verbose: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(short, default_value_t = 1)]
    jobs: u32,
}

#[derive(Debug, GetArgs)]
struct OptionalPositionals<'a> {
    input: &'a str,
    #[arg(default_value = "out.txt")]
    output: &'a str,
    #[arg(default_value_t = 2)]
    level: u8,
}

#[derive(Debug, GetArgs)]
#[command(disable_help_flag)]
struct HelpDisabled<'a> {
    input: &'a str,
}

#[derive(Debug, GetArgs)]
struct ValueName<'a> {
    #[arg(value_name = "FILE")]
    input: &'a str,
}

fn unknown_long(long: &str) -> Error<'_> {
    Error::UnknownOption {
        option: Opt::Long(long),
        suggestion: None,
    }
}

fn unknown_short(short: char) -> Error<'static> {
    Error::UnknownOption {
        option: Opt::Short(short),
        suggestion: None,
    }
}

#[test]
fn missing_required_positional() {
    let args: [&str; 0] = [];
    let mut options = getargs::Options::new(args.into_iter());
    let err = OnePositional::parse(&mut options).unwrap_err();
    assert_eq!(err, Error::MissingArgument("<FIRST>"));
}

#[test]
fn missing_required_option() {
    let args: [&str; 0] = [];
    let mut options = getargs::Options::new(args.into_iter());
    let err = RequiredOption::parse(&mut options).unwrap_err();
    assert_eq!(err, Error::MissingArgument("--name <NAME>"));
}

#[test]
fn missing_argument_uses_value_name() {
    let err = ValueName::from_args([].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingArgument("<FILE>"));
}

#[test]
fn too_many_positionals() {
    let args = ["one", "two"];
    let mut options = getargs::Options::new(args.into_iter());
    let err = OnePositional::parse(&mut options).unwrap_err();
    assert_eq!(err, Error::UnexpectedArgument("two"));
}

#[test]
fn invalid_option_parse_failure() {
    let args = ["-n", "not-a-number"];
    let mut options = getargs::Options::new(args.into_iter());
    let err = ParseInt::parse(&mut options).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "-n <NUM>",
            value: "not-a-number",
        }
    );
}

#[test]
fn invalid_positional_value() {
    let err = OptionalPositionals::from_args(["in", "out", "high"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "<LEVEL>",
            value: "high",
        }
    );
}

#[test]
fn unknown_flag() {
    let args = ["input", "--bogus"];
    let mut options = getargs::Options::new(args.into_iter());
    let err = NoOptions::parse(&mut options).unwrap_err();
    assert_eq!(err, unknown_long("bogus"));
}

#[test]
fn unknown_short_flag() {
    let args = ["input", "-z"];
    let mut options = getargs::Options::new(args.into_iter());
    let err = NoOptions::parse(&mut options).unwrap_err();
    assert_eq!(err, unknown_short('z'));
}

#[test]
fn unknown_long_option_suggests_closest_name() {
    let err = InferredNames::from_args(["--verbos"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::UnknownOption {
            option: Opt::Long("verbos"),
            suggestion: Some("verbose"),
        }
    );

    let err = InferredNames::from_args(["--dry_run"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::UnknownOption {
            option: Opt::Long("dry_run"),
            suggestion: Some("dry-run"),
        }
    );

    let err = InferredNames::from_args(["--hlep"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::UnknownOption {
            option: Opt::Long("hlep"),
            suggestion: Some("help"),
        }
    );
}

#[test]
fn unknown_long_option_with_value_reports_name_only() {
    let err = InferredNames::from_args(["--bogus=1"].into_iter()).unwrap_err();
    assert_eq!(err, unknown_long("bogus"));
}

#[test]
fn unknown_short_option_inside_cluster() {
    let err = InferredNames::from_args(["-vz"].into_iter()).unwrap_err();
    assert_eq!(err, unknown_short('z'));
}

#[test]
fn help_disabled_does_not_suggest_help() {
    let err = HelpDisabled::from_args(["in", "--hel"].into_iter()).unwrap_err();
    assert_eq!(err, unknown_long("hel"));
}

#[test]
fn value_given_to_flag() {
    let err = InferredNames::from_args(["--verbose=yes"].into_iter()).unwrap_err();
    assert_eq!(err, Error::UnexpectedValue(Opt::Long("verbose")));
}

#[test]
fn missing_value_for_short_only_option() {
    let err = InferredNames::from_args(["-j"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOptionValue("-j <JOBS>"));
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
    assert_eq!(err, Error::UnexpectedArgument("extra"));
}

#[test]
fn suggest_ignores_distant_names() {
    assert_eq!(__suggest("bogus", &[&["verbose", "output"]]), None);
    assert_eq!(
        __suggest("outptu", &[&["verbose", "output"]]),
        Some("output")
    );
    assert_eq!(__suggest("x", &[&["y"]]), None);
    assert_eq!(__suggest("anything", &[&[]]), None);
}

#[test]
fn suggest_prefers_closest_name() {
    assert_eq!(__suggest("colr", &[&["colour", "color"]]), Some("color"));
}

#[test]
fn is_help_false_for_other_errors() {
    assert!(!unknown_short('z').is_help());
    assert!(!Error::MissingArgument("<X>").is_help());
}

#[test]
fn error_display_missing_argument() {
    assert_eq!(
        format!("{}", Error::MissingArgument("<OUTPUT>")),
        "the following required argument was not provided: <OUTPUT>"
    );
}

#[test]
fn error_display_unknown_option() {
    assert_eq!(
        format!("{}", unknown_long("bogus")),
        "unexpected argument '--bogus' found"
    );
    assert_eq!(
        format!("{}", unknown_short('z')),
        "unexpected argument '-z' found"
    );
}

#[test]
fn error_display_unknown_option_with_suggestion() {
    let error = Error::UnknownOption {
        option: Opt::Long("verbos"),
        suggestion: Some("verbose"),
    };
    assert_eq!(
        format!("{error}"),
        "unexpected argument '--verbos' found\n\n  tip: a similar argument exists: '--verbose'"
    );
}

#[test]
fn error_display_unexpected_argument() {
    assert_eq!(
        format!("{}", Error::UnexpectedArgument("extra")),
        "unexpected argument 'extra' found"
    );
}

#[test]
fn error_display_unexpected_value() {
    assert_eq!(
        format!("{}", Error::UnexpectedValue(Opt::Long("verbose"))),
        "unexpected value for '--verbose': it does not take a value"
    );
    assert_eq!(
        format!("{}", Error::UnexpectedValue(Opt::Short('v'))),
        "unexpected value for '-v': it does not take a value"
    );
}

#[test]
fn error_display_missing_option_value() {
    assert_eq!(
        format!("{}", Error::MissingOptionValue("--output <OUTPUT>")),
        "a value is required for '--output <OUTPUT>' but none was supplied"
    );
}

#[test]
fn error_display_invalid_arguments() {
    assert_eq!(
        format!("{}", Error::InvalidArguments),
        "the arguments could not be parsed"
    );
    assert!(!Error::InvalidArguments.is_help());
    assert!(!Error::InvalidArguments.is_version());
}

#[test]
fn error_display_invalid_value() {
    let error = Error::InvalidValue {
        name: "--port <PORT>",
        value: "abc",
    };
    assert_eq!(
        format!("{error}"),
        "invalid value 'abc' for '--port <PORT>'"
    );
}

#[test]
fn error_debug_and_clone_and_eq() {
    let a = Error::MissingArgument("<X>");
    let b = Error::MissingArgument("<X>");
    let c = a;
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_ne!(a, unknown_short('x'));
    assert!(format!("{a:?}").contains("MissingArgument"));
}
