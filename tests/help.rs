#![allow(dead_code)]

use core::num::NonZeroU32;
use getargs_derive::{Error, GetArgs, Opt};

#[derive(Debug, GetArgs)]
struct RequiredOption<'a> {
    #[arg(long)]
    name: &'a str,
}

#[derive(Debug, GetArgs)]
struct OnePositional<'a> {
    first: &'a str,
}

/// Copy files around.
///
/// This second paragraph is only shown in long help, which is not generated.
#[derive(Debug, GetArgs)]
#[command(name = "copy")]
struct HelpFull<'a> {
    /// Input file.
    input: &'a str,
    /// Output file
    #[arg(default_value = "out.txt")]
    output: &'a str,
    /// Verbose
    /// output.
    #[arg(short, long)]
    verbose: bool,
    /// Number of jobs
    #[arg(short, long, default_value_t = 4)]
    jobs: u32,
    /// Ignored doc comment
    #[arg(long, help = "Mode", value_name = "KIND")]
    mode: Mode<'a>,
    #[arg(short = 'x')]
    extract: bool,
    #[arg(short = 'C', default_value = "a dir")]
    directory: &'a str,
    #[arg(long, default_value_t = -3)]
    offset: i32,
    #[arg(long, default_value_t = NonZeroU32::new(5).unwrap())]
    threads: NonZeroU32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mode<'a>(&'a str);

impl<'a> core::str::FromStr for Mode<'a> {
    type Err = ();

    fn from_str(_: &str) -> Result<Self, ()> {
        Ok(Mode("parsed"))
    }
}

#[derive(Debug, GetArgs)]
#[command(name = "tool", about = "Overrides the doc comment")]
/// Doc comment about text
struct HelpCommand<'a> {
    #[arg(value_name = "FILE")]
    input: &'a str,
}

#[derive(Debug, GetArgs)]
struct HelpDefaultName {
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, GetArgs)]
#[command(name = "bare", disable_help_flag)]
struct HelpDisabled<'a> {
    input: &'a str,
}

#[derive(Debug, GetArgs)]
#[command(disable_help_flag)]
#[command(name = "host")]
struct HelpDisabledWithShortH<'a> {
    /// Host to connect to
    #[arg(short, long = "host")]
    host: &'a str,
}

#[derive(Debug, GetArgs)]
#[command(name = "empty")]
struct HelpEmpty {}

#[test]
fn help_full_snapshot() {
    assert_eq!(
        HelpFull::HELP,
        "Copy files around

Usage: copy [OPTIONS] <INPUT> [OUTPUT]

Arguments:
  <INPUT>   Input file
  [OUTPUT]  Output file [default: out.txt]

Options:
  -v, --verbose            Verbose output
  -j, --jobs <JOBS>        Number of jobs [default: 4]
      --mode <KIND>        Mode
  -x
  -C <DIRECTORY>           [default: \"a dir\"]
      --offset <OFFSET>    [default: -3]
      --threads <THREADS>  [default: NonZeroU32 :: new(5).unwrap()]
  -h, --help               Print help"
    );
}

#[test]
fn help_command_name_and_about() {
    assert_eq!(
        HelpCommand::HELP,
        "Overrides the doc comment

Usage: tool [OPTIONS] <FILE>

Arguments:
  <FILE>

Options:
  -h, --help  Print help"
    );
}

#[test]
fn help_default_name_is_package_name() {
    assert_eq!(
        HelpDefaultName::HELP,
        "Usage: getargs-derive [OPTIONS]

Options:
      --dry-run
  -h, --help     Print help"
    );
}

#[test]
fn help_disabled_snapshot() {
    assert_eq!(
        HelpDisabled::HELP,
        "Usage: bare <INPUT>\n\nArguments:\n  <INPUT>"
    );
}

#[test]
fn help_empty_struct() {
    assert_eq!(
        HelpEmpty::HELP,
        "Usage: empty [OPTIONS]\n\nOptions:\n  -h, --help  Print help"
    );
}

#[test]
fn help_long_and_short_flags() {
    for flag in ["-h", "--help"] {
        let err = HelpFull::from_args([flag].into_iter()).unwrap_err();
        assert_eq!(err, Error::Help(HelpFull::HELP));
        assert!(err.is_help());
    }
}

#[test]
fn help_takes_priority_over_missing_required() {
    let err = HelpFull::from_args(["-v", "--help"].into_iter()).unwrap_err();
    assert_eq!(err, Error::Help(HelpFull::HELP));
    let err = RequiredOption::from_args(["--help"].into_iter()).unwrap_err();
    assert!(err.is_help());
}

#[test]
fn help_after_double_dash_is_positional() {
    let params = OnePositional::from_args(["--", "--help"].into_iter()).unwrap();
    assert_eq!(params.first, "--help");
}

#[test]
fn help_disabled_treats_help_as_unknown() {
    for flag in ["-h", "--help"] {
        let err = HelpDisabled::from_args(["in", flag].into_iter()).unwrap_err();
        let option = match flag {
            "-h" => Opt::Short('h'),
            _ => Opt::Long("help"),
        };
        assert_eq!(
            err,
            Error::UnknownOption {
                option,
                suggestion: None,
            }
        );
        assert!(!err.is_help());
    }
}

#[test]
fn help_disabled_allows_short_h() {
    let params = HelpDisabledWithShortH::from_args(["-h", "example.com"].into_iter()).unwrap();
    assert_eq!(params.host, "example.com");
    assert_eq!(
        HelpDisabledWithShortH::HELP,
        "Usage: host [OPTIONS]\n\nOptions:\n  -h, --host <HOST>  Host to connect to"
    );
}

#[test]
fn is_version_false_for_other_errors() {
    assert!(!Error::MissingArgument("<INPUT>").is_version());
    assert!(!Error::Help(HelpFull::HELP).is_version());
}

#[test]
fn error_display_help() {
    assert_eq!(
        format!("{}", Error::Help(HelpCommand::HELP)),
        HelpCommand::HELP
    );
}
