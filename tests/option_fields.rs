use core::num::NonZeroU32;
use getargs_derive::{Error, GetArgs};

#[derive(Debug, GetArgs)]
#[command(name = "opt")]
struct OptionFields<'a> {
    /// Input file
    input: &'a str,
    /// Output file
    output: Option<&'a str>,
    level: core::option::Option<u8>,
    /// Name to use
    #[arg(short, long)]
    name: Option<&'a str>,
    #[arg(short, long, value_name = "N")]
    jobs: Option<NonZeroU32>,
    #[arg(long)]
    port: Option<u16>,
}

#[test]
fn option_fields_default_to_none() {
    let params = OptionFields::from_args(["in.txt"].into_iter()).unwrap();
    assert_eq!(params.input, "in.txt");
    assert_eq!(params.output, None);
    assert_eq!(params.level, None);
    assert_eq!(params.name, None);
    assert_eq!(params.jobs, None);
    assert_eq!(params.port, None);
}

#[test]
fn option_fields_are_parsed_when_present() {
    let args = [
        "in.txt", "out.txt", "3", "-n", "bob", "--jobs=8", "--port", "80",
    ];
    let params = OptionFields::from_args(args.into_iter()).unwrap();
    assert_eq!(params.output, Some("out.txt"));
    assert_eq!(params.level, Some(3));
    assert_eq!(params.name, Some("bob"));
    assert_eq!(params.jobs, NonZeroU32::new(8));
    assert_eq!(params.port, Some(80));
}

#[test]
fn option_fields_report_parse_errors() {
    let err = OptionFields::from_args(["in.txt", "-j", "0"].into_iter()).unwrap_err();
    assert_eq!(err, Error::ParseError("jobs"));

    let err = OptionFields::from_args(["in.txt", "out.txt", "x"].into_iter()).unwrap_err();
    assert_eq!(err, Error::ParseError("level"));
}

#[test]
fn option_fields_require_a_value() {
    let err = OptionFields::from_args(["in.txt", "--name"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOptionValue("name"));
}

#[test]
fn option_fields_keep_required_positionals_and_limits() {
    let err = OptionFields::from_args([].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingPositionalArgument("input"));

    let err = OptionFields::from_args(["a", "b", "1", "c"].into_iter()).unwrap_err();
    assert_eq!(err, Error::InvalidNumberOfArguments);
}

#[test]
fn option_fields_help() {
    assert_eq!(
        OptionFields::HELP,
        "Usage: opt [OPTIONS] <INPUT> [OUTPUT] [LEVEL]

Arguments:
  <INPUT>   Input file
  [OUTPUT]  Output file
  [LEVEL]

Options:
  -n, --name <NAME>  Name to use
  -j, --jobs <N>
      --port <PORT>
  -h, --help         Print help"
    );
}
