use getargs_derive::{Error, GetArgs};

#[derive(GetArgs)]
struct WithLegacyDefault {
    #[arg(long)]
    #[default(42)]
    value: u32,
}

#[derive(Debug, GetArgs)]
struct ExplicitPositional<'a> {
    #[arg(positional)]
    name: &'a str,
}

#[derive(Debug, GetArgs)]
struct LegacyForms {
    #[arg(long, default = 5)]
    arg_default: u32,
    #[arg(long)]
    #[default = 7]
    name_value_default: u32,
    #[arg(long, flag)]
    legacy_flag: bool,
}

#[derive(Debug, GetArgs)]
struct LegacyPositionalDefault {
    #[default(9)]
    count: u32,
}

#[test]
fn legacy_default() {
    let args: [&str; 0] = [];
    let mut options = getargs::Options::new(args.into_iter());
    let params = WithLegacyDefault::parse(&mut options).unwrap();
    assert_eq!(params.value, 42);
}

#[test]
fn explicit_positional() {
    let args = ["hello"];
    let mut options = getargs::Options::new(args.into_iter());
    let params = ExplicitPositional::parse(&mut options).unwrap();
    assert_eq!(params.name, "hello");
}

#[test]
fn extra_positional_with_explicit_marker() {
    let args = ["hello", "world"];
    let mut options = getargs::Options::new(args.into_iter());
    let err = ExplicitPositional::parse(&mut options).unwrap_err();
    assert_eq!(err, Error::InvalidNumberOfArguments);
}

#[test]
fn legacy_forms() {
    let params = LegacyForms::from_args([].into_iter()).unwrap();
    assert_eq!(params.arg_default, 5);
    assert_eq!(params.name_value_default, 7);
    assert!(!params.legacy_flag);

    let args = [
        "--arg-default",
        "1",
        "--name-value-default",
        "2",
        "--legacy-flag",
    ];
    let params = LegacyForms::from_args(args.into_iter()).unwrap();
    assert_eq!(params.arg_default, 1);
    assert_eq!(params.name_value_default, 2);
    assert!(params.legacy_flag);
}

#[test]
fn legacy_default_without_name_is_optional_positional() {
    let params = LegacyPositionalDefault::from_args([].into_iter()).unwrap();
    assert_eq!(params.count, 9);

    let params = LegacyPositionalDefault::from_args(["3"].into_iter()).unwrap();
    assert_eq!(params.count, 3);

    let args = ["--count", "3"];
    let err = LegacyPositionalDefault::from_args(args.into_iter()).unwrap_err();
    assert_eq!(err, Error::UnknownOption);
}
