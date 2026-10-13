use getargs_derive::{Error, GetArgs, Opt};

#[derive(GetArgs)]
struct BasicPositionalFlag<'a> {
    positional_field: &'a str,
    #[arg(long)]
    bool_field: bool,
}

#[derive(GetArgs)]
struct TwoPositionals<'a> {
    first: &'a str,
    second: &'a str,
}

#[derive(GetArgs)]
struct TwoPosNoOptions<'a> {
    a: &'a str,
    b: &'a str,
}

#[derive(GetArgs)]
struct Mixed<'a> {
    input: &'a str,
    #[arg(short, long)]
    verbose: bool,
    #[arg(short, long, default_value = "out")]
    output: &'a str,
}

#[derive(Debug, GetArgs)]
struct OptionalPositionals<'a> {
    input: &'a str,
    #[arg(default_value = "out.txt")]
    output: &'a str,
    #[arg(default_value_t = 2)]
    level: u8,
}

#[test]
fn positional_and_flag() {
    let args = ["value1", "--bool-field"];
    let mut options = getargs::Options::new(args.into_iter());
    let params = BasicPositionalFlag::parse(&mut options).unwrap();
    assert_eq!(params.positional_field, "value1");
    assert!(params.bool_field);
}

#[test]
fn two_positionals() {
    let args = ["hello", "world"];
    let mut options = getargs::Options::new(args.into_iter());
    let params = TwoPositionals::parse(&mut options).unwrap();
    assert_eq!(params.first, "hello");
    assert_eq!(params.second, "world");
}

#[test]
fn all_positionals() {
    let args = ["x", "y"];
    let mut options = getargs::Options::new(args.into_iter());
    let params = TwoPosNoOptions::parse(&mut options).unwrap();
    assert_eq!(params.a, "x");
    assert_eq!(params.b, "y");
}

#[test]
fn mixed_positionals_and_options() {
    let args = ["in.txt", "--verbose"];
    let mut options = getargs::Options::new(args.into_iter());
    let params = Mixed::parse(&mut options).unwrap();
    assert_eq!(params.input, "in.txt");
    assert!(params.verbose);
    assert_eq!(params.output, "out");
}

#[test]
fn options_can_appear_before_positionals() {
    let args = ["--verbose", "in.txt"];
    let mut options = getargs::Options::new(args.into_iter());
    let params = Mixed::parse(&mut options).unwrap();
    assert_eq!(params.input, "in.txt");
    assert!(params.verbose);
}

#[test]
fn optional_positionals() {
    let params = OptionalPositionals::from_args(["in.txt"].into_iter()).unwrap();
    assert_eq!(params.input, "in.txt");
    assert_eq!(params.output, "out.txt");
    assert_eq!(params.level, 2);

    let args = ["in.txt", "res.txt"];
    let params = OptionalPositionals::from_args(args.into_iter()).unwrap();
    assert_eq!(params.output, "res.txt");
    assert_eq!(params.level, 2);

    let args = ["in.txt", "res.txt", "7"];
    let params = OptionalPositionals::from_args(args.into_iter()).unwrap();
    assert_eq!(params.level, 7);
}

#[test]
fn optional_positionals_still_require_leading_positional() {
    let err = OptionalPositionals::from_args([].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingArgument("<INPUT>"));
}

#[test]
fn optional_positionals_reject_extra_arguments() {
    let args = ["a", "b", "1", "extra"];
    let err = OptionalPositionals::from_args(args.into_iter()).unwrap_err();
    assert_eq!(err, Error::UnexpectedArgument("extra"));
}

#[test]
fn optional_positionals_are_not_named_options() {
    let args = ["in.txt", "--level", "3"];
    let err = OptionalPositionals::from_args(args.into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::UnknownOption {
            option: Opt::Long("level"),
            suggestion: None,
        }
    );
}
