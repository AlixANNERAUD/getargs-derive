#![allow(dead_code)]

use getargs_derive::{Error, GetArgs};

#[derive(Debug, GetArgs)]
struct FlagActions {
    #[arg(short, long, action = Count)]
    verbose: u8,
    #[arg(short = 'q', action = ArgAction::Count)]
    quiet: u32,
    #[arg(long, action = SetFalse)]
    no_color: bool,
    #[arg(long = "fast", action = SetTrue)]
    fast: bool,
    #[arg(long, action = Count, default_value_t = 3)]
    level: i64,
}

#[test]
fn flag_actions_defaults() {
    let params = FlagActions::from_args([].into_iter()).unwrap();
    assert_eq!(params.verbose, 0);
    assert_eq!(params.quiet, 0);
    assert!(params.no_color);
    assert!(!params.fast);
    assert_eq!(params.level, 3);
}

#[test]
fn count_action_counts_occurrences() {
    let args = ["-vvv", "--verbose", "-q", "-vq", "--level", "--level"];
    let params = FlagActions::from_args(args.into_iter()).unwrap();
    assert_eq!(params.verbose, 5);
    assert_eq!(params.quiet, 2);
    assert_eq!(params.level, 2);
}

#[test]
fn count_action_saturates() {
    let args = ["-v"; 300];
    let params = FlagActions::from_args(args.into_iter()).unwrap();
    assert_eq!(params.verbose, u8::MAX);
}

#[test]
fn set_false_and_set_true_actions() {
    let params = FlagActions::from_args(["--no-color", "--fast"].into_iter()).unwrap();
    assert!(!params.no_color);
    assert!(params.fast);
}

#[test]
fn count_action_rejects_value() {
    let err = FlagActions::from_args(["--verbose=2"].into_iter()).unwrap_err();
    assert_eq!(err, Error::UnknownOption);
}

#[derive(Debug, GetArgs)]
#[command(name = "actions")]
struct HelpActions {
    /// More output
    #[arg(short, long, action = Count)]
    verbose: u8,
    #[arg(short = 'q', action = Count, default_value_t = 1)]
    quiet: u8,
    /// Disable colors
    #[arg(long, action = SetFalse)]
    color: bool,
}

#[test]
fn help_actions_snapshot() {
    assert_eq!(
        HelpActions::HELP,
        "Usage: actions [OPTIONS]

Options:
  -v, --verbose...  More output
  -q...
      --color       Disable colors
  -h, --help        Print help"
    );
}
