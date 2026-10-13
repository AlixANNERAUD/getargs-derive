use core::num::NonZeroU32;
use getargs_derive::{Error, GetArgs};

#[derive(GetArgs)]
struct WithDefault<'a> {
    #[arg(long, default_value = "fallback")]
    name: &'a str,
}

#[derive(Debug, GetArgs)]
struct DefaultForms<'a> {
    #[arg(long, default_value = "8")]
    parsed: u32,
    #[arg(long, default_value_t = 3)]
    typed: u32,
    #[arg(long, default_value_t = "typed")]
    typed_str: &'a str,
    #[arg(long, default_value_t = NonZeroU32::new(5).unwrap())]
    non_zero: NonZeroU32,
}

#[derive(Debug, GetArgs)]
struct InvalidDefaultValue {
    #[arg(long, default_value = "abc")]
    jobs: u32,
}

#[test]
fn default_value_fallback() {
    let args: [&str; 0] = [];
    let mut options = getargs::Options::new(args.into_iter());
    let params = WithDefault::parse(&mut options).unwrap();
    assert_eq!(params.name, "fallback");
}

#[test]
fn default_value_and_default_value_t() {
    let params = DefaultForms::from_args([].into_iter()).unwrap();
    assert_eq!(params.parsed, 8);
    assert_eq!(params.typed, 3);
    assert_eq!(params.typed_str, "typed");
    assert_eq!(params.non_zero.get(), 5);

    let args = [
        "--parsed",
        "1",
        "--typed",
        "2",
        "--typed-str",
        "given",
        "--non-zero",
        "9",
    ];
    let params = DefaultForms::from_args(args.into_iter()).unwrap();
    assert_eq!(params.parsed, 1);
    assert_eq!(params.typed, 2);
    assert_eq!(params.typed_str, "given");
    assert_eq!(params.non_zero.get(), 9);
}

#[test]
fn invalid_default_value_reports_parse_error() {
    let err = InvalidDefaultValue::from_args([].into_iter()).unwrap_err();
    assert_eq!(err, Error::ParseError("jobs"));
}

#[test]
fn invalid_default_value_unused_when_provided() {
    let params = InvalidDefaultValue::from_args(["--jobs", "2"].into_iter()).unwrap();
    assert_eq!(params.jobs, 2);
}
