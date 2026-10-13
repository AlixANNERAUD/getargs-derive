#![allow(dead_code)]

use getargs_derive::{Error, GetArgs, Opt};

#[derive(Debug, PartialEq, GetArgs)]
struct Verbosity {
    /// More output.
    #[arg(short, long, action = Count)]
    verbose: u8,
    /// Less output.
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,
}

#[derive(Debug, PartialEq, GetArgs)]
struct Network<'a> {
    /// Host to connect to.
    #[arg(long, default_value = "localhost")]
    host: &'a str,
    /// Port to connect to.
    #[arg(short, long)]
    port: u16,
}

/// Talk to a server.
#[derive(Debug, PartialEq, GetArgs)]
#[command(name = "client")]
struct Cli<'a> {
    /// Request to send.
    request: &'a str,
    #[command(flatten)]
    network: Network<'a>,
    /// Output file.
    #[arg(short, long)]
    output: Option<&'a str>,
    #[command(flatten)]
    verbosity: Verbosity,
}

/// Talk to a server.
#[derive(GetArgs)]
#[command(name = "client")]
struct FlatCli<'a> {
    /// Request to send.
    request: &'a str,
    /// Host to connect to.
    #[arg(long, default_value = "localhost")]
    host: &'a str,
    /// Port to connect to.
    #[arg(short, long)]
    port: u16,
    /// Output file.
    #[arg(short, long)]
    output: Option<&'a str>,
    /// More output.
    #[arg(short, long, action = Count)]
    verbose: u8,
    /// Less output.
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,
}

#[test]
fn flattened_fields_are_parsed() {
    let cli =
        Cli::from_args(["-p", "80", "get", "-vv", "--host", "example.com"].into_iter()).unwrap();
    assert_eq!(
        cli,
        Cli {
            request: "get",
            network: Network {
                host: "example.com",
                port: 80,
            },
            output: None,
            verbosity: Verbosity {
                verbose: 2,
                quiet: false,
            },
        }
    );
}

#[test]
fn flattened_defaults_apply() {
    let cli = Cli::from_args(["get", "--port", "8080", "-o", "out.txt"].into_iter()).unwrap();
    assert_eq!(cli.network.host, "localhost");
    assert_eq!(cli.output, Some("out.txt"));
    assert_eq!(
        cli.verbosity,
        Verbosity {
            verbose: 0,
            quiet: false
        }
    );
}

#[test]
fn flattened_required_arguments_are_checked() {
    let err = Cli::from_args(["get"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingArgument("--port <PORT>"));
}

#[test]
fn flattened_values_are_validated() {
    let err = Cli::from_args(["get", "--port", "http"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "--port <PORT>",
            value: "http",
        }
    );
    let err = Cli::from_args(["get", "--port"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOptionValue("--port <PORT>"));
}

#[test]
fn flattened_relations_are_checked() {
    let err = Cli::from_args(["get", "-p", "1", "-q", "-v"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::ArgumentConflict {
            name: "--quiet",
            other: "--verbose",
        }
    );
}

#[test]
fn unknown_options_suggest_flattened_names() {
    let err = Cli::from_args(["get", "--prot", "1"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::UnknownOption {
            option: Opt::Long("prot"),
            suggestion: Some("port"),
        }
    );
    let err = Cli::from_args(["get", "--hepl"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::UnknownOption {
            option: Opt::Long("hepl"),
            suggestion: Some("help"),
        }
    );
}

#[test]
fn help_flag_is_handled_by_the_parent() {
    let err = Cli::from_args(["--help"].into_iter()).unwrap_err();
    assert_eq!(err, Error::Help(Cli::HELP));
    assert_eq!(err.to_string(), Cli::HELP);
}

#[test]
fn help_merges_flattened_arguments() {
    assert_eq!(Cli::HELP, FlatCli::HELP);
    assert_eq!(
        Cli::HELP,
        "\
Talk to a server

Usage: client [OPTIONS] <REQUEST>

Arguments:
  <REQUEST>  Request to send

Options:
      --host <HOST>      Host to connect to [default: localhost]
  -p, --port <PORT>      Port to connect to
  -o, --output <OUTPUT>  Output file
  -v, --verbose...       More output
  -q, --quiet            Less output
  -h, --help             Print help"
    );
}

#[derive(Debug, PartialEq, GetArgs)]
struct Source<'a> {
    /// Source file.
    source: &'a str,
}

#[derive(Debug, PartialEq, GetArgs)]
struct Target<'a> {
    /// Target file.
    #[arg(default_value = "out")]
    target: &'a str,
}

#[derive(Debug, PartialEq, GetArgs)]
#[command(name = "copy")]
struct Copy<'a> {
    #[command(flatten)]
    source: Source<'a>,
    /// Mode.
    mode: &'a str,
    #[command(flatten)]
    target: Target<'a>,
}

#[test]
fn positionals_follow_field_order() {
    let copy = Copy::from_args(["a", "b", "c"].into_iter()).unwrap();
    assert_eq!(copy.source.source, "a");
    assert_eq!(copy.mode, "b");
    assert_eq!(copy.target.target, "c");

    let copy = Copy::from_args(["a", "b"].into_iter()).unwrap();
    assert_eq!(copy.target.target, "out");

    let err = Copy::from_args(["a", "b", "c", "d"].into_iter()).unwrap_err();
    assert_eq!(err, Error::UnexpectedArgument("d"));

    let err = Copy::from_args(["a"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingArgument("<MODE>"));
}

#[test]
fn positional_help_follows_field_order() {
    assert_eq!(
        Copy::HELP,
        "\
Usage: copy [OPTIONS] <SOURCE> <MODE> [TARGET]

Arguments:
  <SOURCE>  Source file
  <MODE>    Mode
  [TARGET]  Target file [default: out]

Options:
  -h, --help  Print help"
    );
}

#[derive(Debug, PartialEq, GetArgs)]
struct Inner {
    #[arg(short = 'x', long)]
    extra: bool,
}

#[derive(Debug, PartialEq, GetArgs)]
struct Middle {
    #[command(flatten)]
    inner: Inner,
    #[arg(short, long)]
    middle: bool,
}

#[derive(Debug, PartialEq, GetArgs)]
#[command(name = "outer", version = "1.0", disable_help_flag)]
struct Outer {
    #[command(flatten)]
    middle: Middle,
    #[arg(short = 'h', long)]
    human: bool,
}

#[test]
fn nested_flatten() {
    let outer = Outer::from_args(["-x", "-h", "--middle"].into_iter()).unwrap();
    assert_eq!(
        outer,
        Outer {
            middle: Middle {
                inner: Inner { extra: true },
                middle: true,
            },
            human: true,
        }
    );
    let err = Outer::from_args(["--version"].into_iter()).unwrap_err();
    assert_eq!(err, Error::Version("outer 1.0"));
    assert_eq!(
        Outer::HELP,
        "\
Usage: outer [OPTIONS]

Options:
  -x, --extra
  -m, --middle
  -h, --human
  -V, --version  Print version"
    );
}

#[derive(Debug, PartialEq, GetArgs)]
#[command(name = "empty")]
struct OnlyFlatten {
    #[command(flatten)]
    inner: Inner,
}

#[test]
fn struct_with_only_flattened_fields() {
    let parsed = OnlyFlatten::from_args([].into_iter()).unwrap();
    assert_eq!(parsed.inner, Inner { extra: false });
    let err = OnlyFlatten::from_args(["value"].into_iter()).unwrap_err();
    assert_eq!(err, Error::UnexpectedArgument("value"));
    assert_eq!(
        OnlyFlatten::HELP,
        "\
Usage: empty [OPTIONS]

Options:
  -x, --extra
  -h, --help   Print help"
    );
}

#[derive(GetArgs)]
struct Described<'a> {
    #[arg(
        long,
        value_name = "ÉTAT",
        help = "First line\nsecond line\n\nAfter a blank line"
    )]
    state: Option<&'a str>,
}

#[derive(GetArgs)]
#[command(name = "described")]
struct FlattenedDescription<'a> {
    #[command(flatten)]
    described: Described<'a>,
    #[arg(short, long)]
    all: bool,
}

#[derive(GetArgs)]
#[command(name = "described")]
struct FlatDescription<'a> {
    #[arg(
        long,
        value_name = "ÉTAT",
        help = "First line\nsecond line\n\nAfter a blank line"
    )]
    state: Option<&'a str>,
    #[arg(short, long)]
    all: bool,
}

#[test]
fn merged_help_matches_layout_of_flat_struct() {
    assert_eq!(FlattenedDescription::HELP, FlatDescription::HELP);
}

mod features {
    use getargs_derive::{Error, GetArgs, ValueEnum};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
    pub enum Color {
        Auto,
        Always,
        Never,
    }

    #[derive(Debug, PartialEq, GetArgs)]
    pub struct Output<'a> {
        /// Output file.
        #[arg(short, long, visible_alias = "out", short_alias = 'f')]
        pub output: Option<&'a str>,
        /// When to color the output.
        #[arg(long, value_enum, default_value_t = Color::Auto)]
        pub color: Color,
        /// Worker threads.
        #[arg(short, long, env = "FEATURES_JOBS", value_parser = 1..=64, default_value_t = 4)]
        pub jobs: u8,
        /// Compression level.
        #[arg(long, possible_values = ["low", "high"])]
        pub level: Option<&'a str>,
    }

    #[derive(Debug, PartialEq, GetArgs)]
    #[command(name = "tool")]
    pub struct Cli<'a> {
        /// Input file.
        pub input: &'a str,
        #[command(flatten)]
        pub output: Output<'a>,
    }

    #[derive(GetArgs)]
    #[command(name = "tool")]
    pub struct FlatCli<'a> {
        /// Input file.
        input: &'a str,
        /// Output file.
        #[arg(short, long, visible_alias = "out", short_alias = 'f')]
        output: Option<&'a str>,
        /// When to color the output.
        #[arg(long, value_enum, default_value_t = Color::Auto)]
        color: Color,
        /// Worker threads.
        #[arg(short, long, env = "FEATURES_JOBS", value_parser = 1..=64, default_value_t = 4)]
        jobs: u8,
        /// Compression level.
        #[arg(long, possible_values = ["low", "high"])]
        level: Option<&'a str>,
    }

    fn no_env(_: &str) -> Option<&'static str> {
        None
    }

    #[test]
    fn aliases_inside_flattened_structs() {
        let cli = Cli::from_args_with_env(["in", "--out", "a"].into_iter(), no_env).unwrap();
        assert_eq!(cli.output.output, Some("a"));
        let cli = Cli::from_args_with_env(["in", "-f", "b"].into_iter(), no_env).unwrap();
        assert_eq!(cli.output.output, Some("b"));
        let error = Cli::from_args_with_env(["in", "--ou"].into_iter(), no_env).unwrap_err();
        assert!(matches!(
            error,
            Error::UnknownOption {
                suggestion: Some("out"),
                ..
            }
        ));
    }

    #[test]
    fn env_inside_flattened_structs() {
        let lookup = |name: &str| (name == "FEATURES_JOBS").then_some("8");
        let cli = Cli::from_args_with_env(["in"].into_iter(), lookup).unwrap();
        assert_eq!(cli.output.jobs, 8);
        let cli = Cli::from_args_with_env(["in", "-j", "2"].into_iter(), lookup).unwrap();
        assert_eq!(cli.output.jobs, 2);

        let lookup = |name: &str| (name == "FEATURES_JOBS").then_some("99");
        let error = Cli::from_args_with_env(["in"].into_iter(), lookup).unwrap_err();
        assert_eq!(
            error,
            Error::ValueOutOfRange {
                name: "--jobs <JOBS> [env: FEATURES_JOBS]",
                value: "99",
                range: "1..=64",
            }
        );
    }

    #[test]
    fn validation_inside_flattened_structs() {
        let error = Cli::from_args_with_env(["in", "-j", "0"].into_iter(), no_env).unwrap_err();
        assert_eq!(
            error,
            Error::ValueOutOfRange {
                name: "--jobs <JOBS>",
                value: "0",
                range: "1..=64",
            }
        );
        let error =
            Cli::from_args_with_env(["in", "--level", "max"].into_iter(), no_env).unwrap_err();
        assert_eq!(
            error,
            Error::InvalidPossibleValue {
                name: "--level <LEVEL>",
                value: "max",
                possible_values: &["low", "high"],
            }
        );
    }

    #[test]
    fn value_enums_inside_flattened_structs() {
        let cli = Cli::from_args_with_env(["in"].into_iter(), no_env).unwrap();
        assert_eq!(cli.output.color, Color::Auto);
        let cli = Cli::from_args_with_env(["in", "--color", "never"].into_iter(), no_env).unwrap();
        assert_eq!(cli.output.color, Color::Never);
        let error =
            Cli::from_args_with_env(["in", "--color", "blue"].into_iter(), no_env).unwrap_err();
        assert_eq!(
            error,
            Error::InvalidValue {
                name: "--color <COLOR>",
                value: "blue",
            }
        );
    }

    #[test]
    fn help_matches_flat_struct() {
        assert_eq!(Cli::HELP, FlatCli::HELP);
        assert_eq!(Cli::LONG_HELP, FlatCli::LONG_HELP);
        assert!(Cli::HELP.contains("[env: FEATURES_JOBS=] [default: 4]"));
        assert!(Cli::HELP.contains("[default: auto] [possible values: auto, always, never]"));
        assert!(Cli::HELP.contains("[aliases: --out]"));
        assert!(Cli::HELP.contains("[possible values: low, high]"));
    }
}

mod layout {
    use getargs_derive::GetArgs;

    #[derive(GetArgs)]
    pub struct Network<'a> {
        /// Remote host.
        ///
        /// The host name or address of the server, resolved when the program starts.
        #[arg(long, help_heading = "Network")]
        pub host: Option<&'a str>,
        /// Remote port.
        #[arg(long, help_heading = "Network", display_order = 0)]
        pub port: Option<u16>,
        /// Secret token.
        #[arg(long, hide)]
        pub token: Option<&'a str>,
        /// Target directory.
        pub target: Option<&'a str>,
    }

    /// Fetch files from a server.
    #[derive(GetArgs)]
    #[command(
        name = "fetch",
        max_term_width = 50,
        after_help = "See the manual for more."
    )]
    pub struct Cli<'a> {
        /// Quiet output.
        #[arg(short, long, display_order = 5)]
        pub quiet: bool,
        #[command(flatten)]
        pub network: Network<'a>,
        /// Log file.
        #[arg(long, help_heading = "Logging")]
        pub log: Option<&'a str>,
        /// Extra arguments passed to the server.
        #[arg(last = true)]
        pub rest: Option<&'a str>,
    }

    /// Fetch files from a server.
    #[derive(GetArgs)]
    #[command(
        name = "fetch",
        max_term_width = 50,
        after_help = "See the manual for more."
    )]
    pub struct FlatCli<'a> {
        /// Quiet output.
        #[arg(short, long, display_order = 5)]
        quiet: bool,
        /// Remote host.
        ///
        /// The host name or address of the server, resolved when the program starts.
        #[arg(long, help_heading = "Network")]
        host: Option<&'a str>,
        /// Remote port.
        #[arg(long, help_heading = "Network", display_order = 0)]
        port: Option<u16>,
        /// Secret token.
        #[arg(long, hide)]
        token: Option<&'a str>,
        /// Target directory.
        target: Option<&'a str>,
        /// Log file.
        #[arg(long, help_heading = "Logging")]
        log: Option<&'a str>,
        /// Extra arguments passed to the server.
        #[arg(last = true)]
        rest: Option<&'a str>,
    }

    #[test]
    fn customized_help_matches_flat_struct() {
        assert_eq!(Cli::HELP, FlatCli::HELP);
        assert_eq!(Cli::LONG_HELP, FlatCli::LONG_HELP);
        assert!(Cli::HELP.contains("Usage: fetch [OPTIONS] [TARGET] [-- <REST>]\n"));
        assert!(!Cli::HELP.contains("token"));
        assert!(Cli::HELP.lines().all(|line| line.chars().count() <= 50));
    }

    #[test]
    fn last_positional_skips_flattened_positionals() {
        let cli = Cli::from_args(["--", "x"].into_iter()).unwrap();
        assert_eq!(cli.network.target, None);
        assert_eq!(cli.rest, Some("x"));
        let cli = Cli::from_args(["dir", "--", "x"].into_iter()).unwrap();
        assert_eq!(cli.network.target, Some("dir"));
        assert_eq!(cli.rest, Some("x"));
        assert!(Cli::from_args(["dir", "x"].into_iter()).is_err());
    }

    #[test]
    fn hidden_flattened_options_are_parsed() {
        let cli = Cli::from_args(["--token", "t"].into_iter()).unwrap();
        assert_eq!(cli.network.token, Some("t"));
    }
}

mod parsing {
    use getargs_derive::{Error, GetArgs};

    #[derive(Debug, GetArgs)]
    pub struct Shift<'a> {
        #[arg(short, long, allow_hyphen_values = true)]
        pub pattern: Option<&'a str>,
        #[arg(short, long)]
        pub offset: Option<i32>,
        #[arg(allow_negative_numbers)]
        pub start: Option<i32>,
    }

    #[derive(Debug, GetArgs)]
    pub struct Cli<'a> {
        pub name: Option<&'a str>,
        #[command(flatten)]
        pub shift: Shift<'a>,
    }

    #[derive(Debug, GetArgs)]
    #[command(allow_negative_numbers)]
    pub struct Negative<'a> {
        #[command(flatten)]
        pub shift: Shift<'a>,
    }

    #[test]
    fn parsing_modes_inside_flattened_structs() {
        let cli = Cli::from_args(["-p", "-v", "a", "-5"].into_iter()).unwrap();
        assert_eq!(cli.shift.pattern, Some("-v"));
        assert_eq!(cli.name, Some("a"));
        assert_eq!(cli.shift.start, Some(-5));

        let error = Cli::from_args(["-5"].into_iter()).unwrap_err();
        assert!(matches!(error, Error::UnknownOption { .. }));

        let error = Cli::from_args(["-o", "-3"].into_iter()).unwrap_err();
        assert_eq!(error, Error::MissingOptionValue("--offset <OFFSET>"));
    }

    #[test]
    fn parent_allow_negative_numbers_applies_to_flattened_structs() {
        let cli = Negative::from_args(["-o", "-3"].into_iter()).unwrap();
        assert_eq!(cli.shift.offset, Some(-3));
    }
}
