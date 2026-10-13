#![allow(dead_code)]

use getargs_derive::{Error, GetArgs};

#[derive(GetArgs)]
struct BasicPositionalFlag<'a> {
    positional_field: &'a str,
    #[arg(long)]
    bool_field: bool,
}

#[derive(GetArgs)]
struct WithDefault<'a> {
    #[arg(long, default_value = "fallback")]
    name: &'a str,
}

#[derive(Debug, GetArgs)]
struct ParseInt {
    #[arg(short = 'n')]
    num: u32,
}

#[test]
fn from_args_convenience_method() {
    let args = ["value1", "--bool-field"];
    let params = BasicPositionalFlag::from_args(args.into_iter()).unwrap();
    assert_eq!(params.positional_field, "value1");
    assert!(params.bool_field);
}

#[test]
fn from_args_empty_defaults() {
    let args: [&str; 0] = [];
    let params = WithDefault::from_args(args.into_iter()).unwrap();
    assert_eq!(params.name, "fallback");
}

#[test]
fn from_args_parse_error() {
    let args = ["-n", "not-a-number"];
    let err = ParseInt::from_args(args.into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "-n <NUM>",
            value: "not-a-number",
        }
    );
}
