#![allow(dead_code)]

use getargs_derive::{Error, GetArgs, Opt};

#[derive(Debug, GetArgs)]
#[command(name = "copy")]
struct Aliased<'a> {
    /// Output file
    #[arg(
        short,
        long,
        alias = "out",
        visible_aliases = ["destination", "dest"],
        short_alias = 'O',
        visible_short_alias = 'd'
    )]
    output: &'a str,
    /// Verbose output
    #[arg(
        long,
        aliases = ["loud", "chatty"],
        visible_alias = "talkative",
        short_aliases = ['l', 'L'],
        visible_short_aliases = ['t', 'T']
    )]
    verbose: bool,
    /// Number of jobs
    #[arg(short, default_value_t = 1, visible_alias = "jobs")]
    jobs: u32,
    #[arg(long, action = Count, short_alias = 'q')]
    quiet: u8,
}

#[derive(Debug, GetArgs)]
#[command(name = "plain", disable_help_flag)]
struct HelpAlias {
    #[arg(long, short_alias = 'h', alias = "help")]
    usage: bool,
}

#[derive(Debug, GetArgs)]
#[command(name = "plain")]
struct VersionAlias {
    #[arg(long, short_alias = 'V', alias = "version")]
    show_version: bool,
}

#[test]
fn primary_names_still_match() {
    let cli = Aliased::from_args(["-o", "a", "--verbose", "-j", "3"].into_iter()).unwrap();
    assert_eq!(cli.output, "a");
    assert!(cli.verbose);
    assert_eq!(cli.jobs, 3);
}

#[test]
fn long_aliases_match() {
    for name in ["--out", "--destination", "--dest"] {
        let cli = Aliased::from_args([name, "file"].into_iter()).unwrap();
        assert_eq!(cli.output, "file");
    }
    let cli = Aliased::from_args(["--out=file", "--chatty"].into_iter()).unwrap();
    assert_eq!(cli.output, "file");
    assert!(cli.verbose);
    let cli = Aliased::from_args(["--dest", "x", "--jobs", "5"].into_iter()).unwrap();
    assert_eq!(cli.jobs, 5);
    for name in ["--loud", "--talkative"] {
        let cli = Aliased::from_args(["-o", "x", name].into_iter()).unwrap();
        assert!(cli.verbose);
    }
}

#[test]
fn short_aliases_match() {
    for name in ["-O", "-d"] {
        let cli = Aliased::from_args([name, "file"].into_iter()).unwrap();
        assert_eq!(cli.output, "file");
    }
    for name in ["-l", "-L", "-t", "-T"] {
        let cli = Aliased::from_args(["-o", "x", name].into_iter()).unwrap();
        assert!(cli.verbose);
    }
    let cli = Aliased::from_args(["-dfile", "-tq", "--quiet", "-qq"].into_iter()).unwrap();
    assert_eq!(cli.output, "file");
    assert!(cli.verbose);
    assert_eq!(cli.quiet, 4);
}

#[test]
fn errors_use_primary_name() {
    let err = Aliased::from_args(["--dest"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOptionValue("--output <OUTPUT>"));
    let err = Aliased::from_args(["-O", "x", "--jobs", "many"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "-j <JOBS>",
            value: "many",
        }
    );
    let err = Aliased::from_args([].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingArgument("--output <OUTPUT>"));
}

#[test]
fn aliases_are_suggested() {
    let err = Aliased::from_args(["--destinaton"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::UnknownOption {
            option: Opt::Long("destinaton"),
            suggestion: Some("destination"),
        }
    );
    let err = Aliased::from_args(["--chaty"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::UnknownOption {
            option: Opt::Long("chaty"),
            suggestion: Some("chatty"),
        }
    );
}

#[test]
fn visible_aliases_appear_in_help() {
    assert_eq!(
        Aliased::HELP,
        "\
Usage: copy [OPTIONS]

Options:
  -o, --output <OUTPUT>  Output file [aliases: --destination, --dest] [short aliases: -d]
      --verbose          Verbose output [aliases: --talkative] [short aliases: -t, -T]
  -j <JOBS>              Number of jobs [default: 1] [aliases: --jobs]
      --quiet...
  -h, --help             Print help"
    );
}

#[test]
fn aliases_can_reuse_disabled_reserved_names() {
    for name in ["-h", "--help"] {
        let cli = HelpAlias::from_args([name].into_iter()).unwrap();
        assert!(cli.usage);
    }
    for name in ["-V", "--version"] {
        let cli = VersionAlias::from_args([name].into_iter()).unwrap();
        assert!(cli.show_version);
    }
}
