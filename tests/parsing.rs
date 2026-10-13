use getargs_derive::{Error, GetArgs, Opt};

#[derive(Debug, GetArgs)]
#[command(name = "tool")]
struct Strict<'a> {
    #[arg(short, long)]
    pattern: Option<&'a str>,
    #[arg(short, long)]
    offset: Option<i32>,
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, GetArgs)]
struct Hyphen<'a> {
    #[arg(short, long, allow_hyphen_values = true)]
    pattern: Option<&'a str>,
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, GetArgs)]
struct NegativeFields<'a> {
    #[arg(short, long, allow_negative_numbers = true)]
    offset: Option<f64>,
    #[arg(allow_negative_numbers)]
    first: Option<i32>,
    second: Option<&'a str>,
    #[arg(short = '1')]
    one: bool,
}

#[derive(Debug, GetArgs)]
#[command(allow_negative_numbers)]
struct NegativeContainer<'a> {
    #[arg(short, long)]
    offset: Option<i32>,
    #[arg(short, long)]
    verbose: bool,
    value: Option<&'a str>,
}

#[derive(Debug, GetArgs)]
#[command(name = "run")]
struct Last<'a> {
    #[arg(short, long)]
    verbose: bool,
    program: Option<&'a str>,
    #[arg(last = true)]
    rest: Option<&'a str>,
}

#[derive(Debug, GetArgs)]
#[command(name = "exec")]
struct RequiredLast<'a> {
    #[arg(last)]
    command: &'a str,
}

#[derive(Debug, GetArgs)]
struct DisabledModes<'a> {
    #[arg(short, allow_hyphen_values = false, allow_negative_numbers = false)]
    value: Option<&'a str>,
    #[arg(last = false)]
    input: Option<&'a str>,
}

#[test]
fn values_starting_with_a_hyphen_are_rejected_by_default() {
    let err = Strict::from_args(["--pattern", "-v"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOptionValue("--pattern <PATTERN>"));
    let err = Strict::from_args(["-o", "-5"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOptionValue("--offset <OFFSET>"));
    let err = Strict::from_args(["--pattern", "--"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOptionValue("--pattern <PATTERN>"));
    let cli = Strict::from_args(["-p", "x", "-v"].into_iter()).unwrap();
    assert_eq!(cli.pattern, Some("x"));
    assert!(cli.verbose);
}

#[test]
fn attached_hyphen_values_are_always_accepted() {
    let cli = Strict::from_args(["--pattern=-v", "-o-5"].into_iter()).unwrap();
    assert_eq!(cli.pattern, Some("-v"));
    assert_eq!(cli.offset, Some(-5));
    let cli = Strict::from_args(["-p-x"].into_iter()).unwrap();
    assert_eq!(cli.pattern, Some("-x"));
}

#[test]
fn a_single_hyphen_is_a_value() {
    let cli = Strict::from_args(["--pattern", "-"].into_iter()).unwrap();
    assert_eq!(cli.pattern, Some("-"));
}

#[test]
fn allow_hyphen_values_accepts_the_next_argument() {
    let cli = Hyphen::from_args(["--pattern", "-v"].into_iter()).unwrap();
    assert_eq!(cli.pattern, Some("-v"));
    assert!(!cli.verbose);
    let cli = Hyphen::from_args(["-p", "--verbose", "-v"].into_iter()).unwrap();
    assert_eq!(cli.pattern, Some("--verbose"));
    assert!(cli.verbose);
    let cli = Hyphen::from_args(["-p", "--"].into_iter()).unwrap();
    assert_eq!(cli.pattern, Some("--"));
}

#[test]
fn allow_negative_numbers_on_an_option() {
    let cli = NegativeFields::from_args(["--offset", "-1.5"].into_iter()).unwrap();
    assert_eq!(cli.offset, Some(-1.5));
    let cli = NegativeFields::from_args(["-o", "-2e3"].into_iter()).unwrap();
    assert_eq!(cli.offset, Some(-2000.0));
    let err = NegativeFields::from_args(["--offset", "-x"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOptionValue("--offset <OFFSET>"));
    let err = NegativeFields::from_args(["--offset", "-.5"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOptionValue("--offset <OFFSET>"));
}

#[test]
fn allow_negative_numbers_on_a_positional() {
    let cli = NegativeFields::from_args(["-15", "-1"].into_iter()).unwrap();
    assert_eq!(cli.first, Some(-15));
    assert_eq!(cli.second, None);
    assert!(cli.one);

    let cli = NegativeFields::from_args(["-1"].into_iter()).unwrap();
    assert_eq!(cli.first, Some(-1));
    assert!(!cli.one);

    let err = NegativeFields::from_args(["-1.5"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "<FIRST>",
            value: "-1.5"
        }
    );
}

#[test]
fn negative_numbers_only_apply_to_enabled_positionals() {
    let err = NegativeFields::from_args(["3", "-4"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::UnknownOption {
            option: Opt::Short('4'),
            suggestion: None
        }
    );
    let cli = NegativeFields::from_args(["3", "--", "-4"].into_iter()).unwrap();
    assert_eq!(cli.first, Some(3));
    assert_eq!(cli.second, Some("-4"));
}

#[test]
fn allow_negative_numbers_on_the_container() {
    let cli = NegativeContainer::from_args(["-o", "-3", "-7", "-v"].into_iter()).unwrap();
    assert_eq!(cli.offset, Some(-3));
    assert_eq!(cli.value, Some("-7"));
    assert!(cli.verbose);

    let err = NegativeContainer::from_args(["-o", "-v"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOptionValue("--offset <OFFSET>"));
}

#[test]
fn negative_number_positionals_need_from_args() {
    let mut options = getargs::Options::new(["-7"].into_iter());
    let err = NegativeContainer::parse(&mut options).unwrap_err();
    assert_eq!(
        err,
        Error::UnknownOption {
            option: Opt::Short('7'),
            suggestion: None
        }
    );

    let mut options = getargs::Options::new(["-o", "-3"].into_iter());
    let cli = NegativeContainer::parse(&mut options).unwrap();
    assert_eq!(cli.offset, Some(-3));
}

#[test]
fn last_positional_requires_double_dash() {
    let cli = Last::from_args(["cargo", "--", "--release"].into_iter()).unwrap();
    assert_eq!(cli.program, Some("cargo"));
    assert_eq!(cli.rest, Some("--release"));

    let err = Last::from_args(["cargo", "build"].into_iter()).unwrap_err();
    assert_eq!(err, Error::UnexpectedArgument("build"));

    let cli = Last::from_args(["cargo", "-v"].into_iter()).unwrap();
    assert_eq!(cli.rest, None);
    assert!(cli.verbose);
}

#[test]
fn double_dash_jumps_to_the_last_positional() {
    let cli = Last::from_args(["--", "-v"].into_iter()).unwrap();
    assert_eq!(cli.program, None);
    assert_eq!(cli.rest, Some("-v"));
    assert!(!cli.verbose);

    let err = Last::from_args(["--", "a", "b"].into_iter()).unwrap_err();
    assert_eq!(err, Error::UnexpectedArgument("b"));
}

#[test]
fn last_positional_works_with_parse() {
    let mut options = getargs::Options::new(["--", "x"].into_iter());
    let cli = Last::parse(&mut options).unwrap();
    assert_eq!(cli.rest, Some("x"));
}

#[test]
fn required_last_positional() {
    let cli = RequiredLast::from_args(["--", "ls"].into_iter()).unwrap();
    assert_eq!(cli.command, "ls");
    let err = RequiredLast::from_args(["ls"].into_iter()).unwrap_err();
    assert_eq!(err, Error::UnexpectedArgument("ls"));
    let err = RequiredLast::from_args([].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingArgument("<COMMAND>"));
}

#[test]
fn last_positional_usage() {
    assert!(Last::HELP.contains("Usage: run [OPTIONS] [PROGRAM] [-- <REST>]\n"));
    assert!(RequiredLast::HELP.contains("Usage: exec [OPTIONS] -- <COMMAND>\n"));
}

#[test]
fn disabled_modes_keep_the_default_behaviour() {
    let err = DisabledModes::from_args(["-v", "-x"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOptionValue("-v <VALUE>"));
    let cli = DisabledModes::from_args(["input", "-v", "x"].into_iter()).unwrap();
    assert_eq!(cli.input, Some("input"));
    assert_eq!(cli.value, Some("x"));
}

#[derive(Debug, GetArgs)]
#[command(name = "shift")]
struct WithEnv<'a> {
    /// Offset to apply.
    ///
    /// Negative offsets shift backwards.
    #[arg(short, long, env = "SHIFT_OFFSET", allow_negative_numbers = true, value_parser = -10..=10)]
    offset: Option<i32>,
    #[arg(allow_negative_numbers, env = "SHIFT_START")]
    start: Option<i32>,
    #[arg(last = true)]
    rest: Option<&'a str>,
}

#[test]
fn env_variants_accept_negative_numbers() {
    let unset = |_: &str| None;
    let cli = WithEnv::from_args_with_env(["-5", "-o", "-3"].into_iter(), unset).unwrap();
    assert_eq!(cli.start, Some(-5));
    assert_eq!(cli.offset, Some(-3));

    let vars = |name: &str| (name == "SHIFT_START").then_some("-7");
    let cli = WithEnv::from_args_with_env(["--", "x"].into_iter(), vars).unwrap();
    assert_eq!(cli.start, Some(-7));
    assert_eq!(cli.rest, Some("x"));
}

#[test]
fn accepted_hyphen_values_are_validated() {
    let unset = |_: &str| None;
    let err = WithEnv::from_args_with_env(["-o", "-11"].into_iter(), unset).unwrap_err();
    assert_eq!(
        err,
        Error::ValueOutOfRange {
            name: "--offset <OFFSET>",
            value: "-11",
            range: "-10..=10"
        }
    );
}

#[test]
fn last_positional_usage_in_long_help() {
    assert!(WithEnv::HELP.contains("Usage: shift [OPTIONS] [START] [-- <REST>]\n"));
    assert!(WithEnv::LONG_HELP.contains("Usage: shift [OPTIONS] [START] [-- <REST>]\n"));
}
