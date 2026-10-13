#![allow(dead_code)]

use core::num::ParseIntError;
use getargs_derive::{Error, GetArgs};

#[derive(Debug, PartialEq)]
struct Port(u16);

fn parse_port(value: &str) -> Result<Port, ParseIntError> {
    value.parse().map(Port)
}

fn non_empty(value: &str) -> Result<&str, ()> {
    match value.trim() {
        "" => Err(()),
        trimmed => Ok(trimmed),
    }
}

mod parsers {
    pub fn even(value: &str) -> Result<u32, ()> {
        match value.parse::<u32>() {
            Ok(number) if number % 2 == 0 => Ok(number),
            _ => Err(()),
        }
    }
}

#[derive(Debug, GetArgs)]
#[command(name = "validation")]
struct Validated<'a> {
    #[arg(value_parser = non_empty)]
    name: &'a str,
    #[arg(short, long, value_parser = parse_port, default_value = "8080")]
    port: Port,
    #[arg(short, long, value_parser = 1..=64, default_value_t = 4)]
    jobs: u8,
    #[arg(long, value_parser = parsers::even)]
    even: Option<u32>,
    #[arg(short, long, possible_values = ["fast", "slow"], default_value = "fast")]
    mode: &'a str,
    #[arg(long, possible_values = ["1", "2", "4"])]
    level: Option<u8>,
}

#[test]
fn defaults_go_through_validation() {
    let params = Validated::from_args(["app"].into_iter()).unwrap();
    assert_eq!(params.name, "app");
    assert_eq!(params.port, Port(8080));
    assert_eq!(params.jobs, 4);
    assert_eq!(params.even, None);
    assert_eq!(params.mode, "fast");
    assert_eq!(params.level, None);
}

#[test]
fn value_parser_functions() {
    let args = [" app ", "-p", "3000", "--even", "6"];
    let params = Validated::from_args(args.into_iter()).unwrap();
    assert_eq!(params.name, "app");
    assert_eq!(params.port, Port(3000));
    assert_eq!(params.even, Some(6));

    let err = Validated::from_args(["  "].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "<NAME>",
            value: "  "
        }
    );

    let err = Validated::from_args(["app", "--port", "http"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "--port <PORT>",
            value: "http"
        }
    );

    let err = Validated::from_args(["app", "--even", "3"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "--even <EVEN>",
            value: "3"
        }
    );
}

#[test]
fn ranges() {
    let params = Validated::from_args(["app", "-j", "1"].into_iter()).unwrap();
    assert_eq!(params.jobs, 1);
    let params = Validated::from_args(["app", "-j", "64"].into_iter()).unwrap();
    assert_eq!(params.jobs, 64);

    let err = Validated::from_args(["app", "-j", "0"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::ValueOutOfRange {
            name: "--jobs <JOBS>",
            value: "0",
            range: "1..=64"
        }
    );
    assert_eq!(
        err.to_string(),
        "invalid value '0' for '--jobs <JOBS>': 0 is not in 1..=64"
    );

    let err = Validated::from_args(["app", "-j", "65"].into_iter()).unwrap_err();
    assert!(matches!(err, Error::ValueOutOfRange { value: "65", .. }));

    let err = Validated::from_args(["app", "-j", "x"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "--jobs <JOBS>",
            value: "x"
        }
    );
}

#[test]
fn possible_values() {
    let params = Validated::from_args(["app", "--mode=slow", "--level", "4"].into_iter()).unwrap();
    assert_eq!(params.mode, "slow");
    assert_eq!(params.level, Some(4));

    let err = Validated::from_args(["app", "-m", "turbo"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidPossibleValue {
            name: "--mode <MODE>",
            value: "turbo",
            possible_values: &["fast", "slow"],
        }
    );
    assert_eq!(
        err.to_string(),
        "invalid value 'turbo' for '--mode <MODE>'\n  [possible values: fast, slow]"
    );

    let err = Validated::from_args(["app", "--level", "04"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidPossibleValue {
            name: "--level <LEVEL>",
            value: "04",
            possible_values: &["1", "2", "4"],
        }
    );
}

#[derive(Debug, GetArgs)]
struct Ranges {
    #[arg(long, value_parser = 1..)]
    from: Option<u32>,
    #[arg(long, value_parser = ..10)]
    to: Option<u32>,
    #[arg(long, value_parser = ..=-1)]
    negative: Option<i8>,
    #[arg(long, value_parser = -5..5)]
    window: Option<i64>,
    #[arg(long, value_parser = 1..=8, default_value = "9")]
    invalid_default: u8,
}

#[test]
fn range_forms() {
    let args = [
        "--from",
        "7",
        "--to",
        "9",
        "--negative",
        "-3",
        "--window=-5",
        "--invalid-default",
        "8",
    ];
    let params = Ranges::from_args(args.into_iter()).unwrap();
    assert_eq!(params.from, Some(7));
    assert_eq!(params.to, Some(9));
    assert_eq!(params.negative, Some(-3));
    assert_eq!(params.window, Some(-5));
    assert_eq!(params.invalid_default, 8);

    let cases: [(&[&str], &str); 4] = [
        (&["--from", "0"], "1.."),
        (&["--to", "10"], "..10"),
        (&["--negative", "0"], "..=-1"),
        (&["--window", "5"], "-5..5"),
    ];
    for (args, expected) in cases {
        let args = args.iter().copied().chain(["--invalid-default", "1"]);
        match Ranges::from_args(args).unwrap_err() {
            Error::ValueOutOfRange { range, .. } => assert_eq!(range, expected),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    let err = Ranges::from_args([].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::ValueOutOfRange {
            name: "--invalid-default <INVALID_DEFAULT>",
            value: "9",
            range: "1..=8"
        }
    );
}

#[derive(Debug, GetArgs)]
#[command(name = "validation")]
struct Combined<'a> {
    #[arg(possible_values = ["2", "3", "20"], value_parser = 1..=10)]
    count: u32,
    #[arg(long, possible_values = ["a b", "c"], value_parser = non_empty)]
    spaced: Option<&'a str>,
}

#[test]
fn combined_validation_order() {
    let params = Combined::from_args(["3", "--spaced", "a b"].into_iter()).unwrap();
    assert_eq!(params.count, 3);
    assert_eq!(params.spaced, Some("a b"));

    let err = Combined::from_args(["5"].into_iter()).unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidPossibleValue { value: "5", .. }
    ));

    let err = Combined::from_args(["20"].into_iter()).unwrap_err();
    assert!(matches!(err, Error::ValueOutOfRange { value: "20", .. }));

    let err = Combined::from_args(["2", "--spaced", "d"].into_iter()).unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid value 'd' for '--spaced <SPACED>'\n  [possible values: \"a b\", c]"
    );
}

#[test]
fn help_lists_possible_values() {
    assert_eq!(
        Validated::HELP,
        "Usage: validation [OPTIONS] <NAME>

Arguments:
  <NAME>

Options:
  -p, --port <PORT>    [default: 8080]
  -j, --jobs <JOBS>    [default: 4]
      --even <EVEN>
  -m, --mode <MODE>    [default: fast] [possible values: fast, slow]
      --level <LEVEL>  [possible values: 1, 2, 4]
  -h, --help           Print help"
    );
    assert_eq!(
        Combined::HELP,
        "Usage: validation [OPTIONS] <COUNT>

Arguments:
  <COUNT>  [possible values: 2, 3, 20]

Options:
      --spaced <SPACED>  [possible values: \"a b\", c]
  -h, --help             Print help"
    );
}

#[derive(Debug, GetArgs)]
struct FromEnv<'a> {
    #[arg(long, env = "JOBS", value_parser = 1..=64)]
    jobs: Option<u8>,
    #[arg(long, env = "MODE", possible_values = ["fast", "slow"])]
    mode: Option<&'a str>,
}

#[test]
fn env_values_are_validated() {
    let vars = [("JOBS", "8"), ("MODE", "slow")];
    let lookup = |name: &str| {
        vars.iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| *value)
    };
    let cli = FromEnv::from_args_with_env([].into_iter(), lookup).unwrap();
    assert_eq!(cli.jobs, Some(8));
    assert_eq!(cli.mode, Some("slow"));

    let err = FromEnv::from_args_with_env([].into_iter(), |name: &str| {
        (name == "JOBS").then_some("65")
    })
    .unwrap_err();
    assert_eq!(
        err,
        Error::ValueOutOfRange {
            name: "--jobs <JOBS> [env: JOBS]",
            value: "65",
            range: "1..=64"
        }
    );

    let err = FromEnv::from_args_with_env([].into_iter(), |name: &str| {
        (name == "MODE").then_some("turbo")
    })
    .unwrap_err();
    assert_eq!(
        err,
        Error::InvalidPossibleValue {
            name: "--mode <MODE> [env: MODE]",
            value: "turbo",
            possible_values: &["fast", "slow"]
        }
    );
}
