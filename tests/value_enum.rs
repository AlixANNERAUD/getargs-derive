#![allow(dead_code)]

use getargs_derive::{Error, GetArgs, ParseValueEnumError, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Mode {
    Fast,
    #[value(alias = "careful")]
    SlowAndSteady,
    #[value(name = "turbo", alias = "t", alias = "max")]
    HTTPTurbo,
    r#Raw,
}

#[derive(Debug, PartialEq, Eq, ValueEnum)]
enum Level {
    Low,
    High,
}

#[test]
fn value_enum_exposes_variants_and_values() {
    assert_eq!(
        Mode::POSSIBLE_VALUES,
        &["fast", "slow-and-steady", "turbo", "raw"]
    );
    assert_eq!(
        Mode::VARIANTS,
        &[Mode::Fast, Mode::SlowAndSteady, Mode::HTTPTurbo, Mode::Raw]
    );
    assert_eq!(Mode::SlowAndSteady.value_name(), "slow-and-steady");
    assert_eq!(Mode::HTTPTurbo.value_name(), "turbo");
}

#[test]
fn value_enum_parses_names_and_aliases() {
    assert_eq!("fast".parse(), Ok(Mode::Fast));
    assert_eq!("careful".parse(), Ok(Mode::SlowAndSteady));
    assert_eq!("t".parse(), Ok(Mode::HTTPTurbo));
    assert_eq!("max".parse(), Ok(Mode::HTTPTurbo));
    assert_eq!("raw".parse(), Ok(Mode::Raw));
    assert_eq!("Fast".parse::<Mode>(), Err(ParseValueEnumError));
    assert_eq!("http-turbo".parse::<Mode>(), Err(ParseValueEnumError));
    assert_eq!(
        Mode::from_value("slow-and-steady"),
        Some(Mode::SlowAndSteady)
    );
}

#[derive(Debug, GetArgs)]
#[command(name = "run")]
struct Cli {
    /// Execution mode.
    #[arg(short, long, value_enum, default_value_t = Mode::SlowAndSteady)]
    mode: Mode,
    /// Fallback mode.
    #[arg(long, value_enum, default_value = "t")]
    fallback: Mode,
    #[arg(long, value_enum)]
    level: Option<Level>,
    /// Target level.
    #[arg(value_enum)]
    target: Level,
}

#[test]
fn value_enum_fields_parse() {
    let cli = Cli::from_args(["-m", "fast", "--level=high", "low"].into_iter()).unwrap();
    assert_eq!(cli.mode, Mode::Fast);
    assert_eq!(cli.fallback, Mode::HTTPTurbo);
    assert_eq!(cli.level, Some(Level::High));
    assert_eq!(cli.target, Level::Low);

    let cli = Cli::from_args(["high"].into_iter()).unwrap();
    assert_eq!(cli.mode, Mode::SlowAndSteady);
    assert_eq!(cli.level, None);
    assert_eq!(cli.target, Level::High);
}

#[test]
fn value_enum_fields_reject_unknown_values() {
    let err = Cli::from_args(["--mode", "slow", "low"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "--mode <MODE>",
            value: "slow",
        }
    );
    let err = Cli::from_args(["medium"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "<TARGET>",
            value: "medium",
        }
    );
}

#[test]
fn value_enum_help_lists_possible_values() {
    assert_eq!(
        Cli::HELP,
        "\
Usage: run [OPTIONS] <TARGET>

Arguments:
  <TARGET>  Target level [possible values: low, high]

Options:
  -m, --mode <MODE>          Execution mode [default: slow-and-steady] [possible values: fast, slow-and-steady, turbo, raw]
      --fallback <FALLBACK>  Fallback mode [default: t] [possible values: fast, slow-and-steady, turbo, raw]
      --level <LEVEL>        [possible values: low, high]
  -h, --help                 Print help"
    );
}

const HELP: &str = Cli::HELP;

#[test]
fn value_enum_help_is_const() {
    assert!(HELP.ends_with("Print help"));
}

#[derive(GetArgs)]
#[command(name = "lifetimes")]
struct Borrowed<'a> {
    #[arg(long)]
    name: &'a str,
    #[arg(long, value_enum, default_value_t = Level::High)]
    level: Level,
    #[arg(long, value_enum, default_value_t = Mode::r#Raw)]
    mode: Mode,
}

#[test]
fn value_enum_with_borrowed_fields() {
    let cli = Borrowed::from_args(["--name", "x"].into_iter()).unwrap();
    assert_eq!(cli.name, "x");
    assert_eq!(cli.level, Level::High);
    assert_eq!(cli.mode, Mode::Raw);
    assert!(Borrowed::HELP.contains("[default: high] [possible values: low, high]"));
    assert!(Borrowed::HELP.contains("[default: raw] [possible values: fast,"));
}

struct Manual(u8);

impl ValueEnum for Manual {
    const VARIANTS: &'static [Self] = &[Manual(1), Manual(2)];
    const POSSIBLE_VALUES: &'static [&'static str] = &["one", "two"];

    fn value_name(&self) -> &'static str {
        Self::POSSIBLE_VALUES[usize::from(self.0) - 1]
    }

    fn from_value(value: &str) -> Option<Self> {
        match value {
            "one" => Some(Manual(1)),
            "two" => Some(Manual(2)),
            _ => None,
        }
    }
}

const TWO: Manual = Manual(2);

#[derive(GetArgs)]
#[command(name = "manual")]
struct ManualCli {
    #[arg(long, value_enum, default_value_t = TWO)]
    count: Manual,
}

#[test]
fn manual_value_enum_implementation() {
    let cli = ManualCli::from_args(["--count", "one"].into_iter()).unwrap();
    assert_eq!(cli.count.0, 1);
    assert_eq!(
        ManualCli::HELP,
        "Usage: manual [OPTIONS]\n\nOptions:\n      --count <COUNT>  [default: TWO] [possible values: one, two]\n  -h, --help           Print help"
    );
}

#[derive(GetArgs)]
struct ManualEnvCli {
    #[arg(long, value_enum, env = "MANUAL_COUNT")]
    count: Manual,
}

#[test]
fn manual_value_enum_reads_env() {
    let cli = ManualEnvCli::from_args_with_env([].into_iter(), |_: &str| Some("two")).unwrap();
    assert_eq!(cli.count.0, 2);
    let error = ManualEnvCli::from_args_with_env([].into_iter(), |_: &str| Some("three"))
        .err()
        .unwrap();
    assert_eq!(
        error,
        Error::InvalidValue {
            name: "--count <COUNT> [env: MANUAL_COUNT]",
            value: "three",
        }
    );
}
