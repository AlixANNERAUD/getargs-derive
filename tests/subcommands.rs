#![allow(dead_code)]

use getargs_derive::{Command, Error, GetArgs, Opt, Subcommand, ValueEnum, Values};

#[derive(Debug, PartialEq, GetArgs)]
struct Verbosity {
    /// More output.
    #[arg(short, long, action = Count)]
    verbose: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum Mode {
    Fast,
    Slow,
}

/// Show the remotes.
#[derive(Debug, PartialEq, GetArgs)]
struct Show<'a> {
    /// Remote to show.
    name: Option<&'a str>,
}

#[derive(Debug, PartialEq, Subcommand)]
enum Remote<'a> {
    /// Add a remote.
    Add {
        /// Name of the remote.
        name: &'a str,
        /// URL of the remote.
        url: &'a str,
    },
    /// Remove a remote.
    #[command(visible_alias = "rm")]
    Remove {
        /// Name of the remote.
        name: &'a str,
    },
    Show(Show<'a>),
}

#[derive(Debug, PartialEq, Subcommand)]
enum Commands<'a> {
    /// Copy files.
    #[command(alias = "cp")]
    Copy {
        /// Source file.
        source: &'a str,
        /// Copy mode.
        #[arg(short, long, value_enum, default_value_t = Mode::Fast)]
        mode: Mode,
        /// Number of jobs.
        #[arg(short, long, env = "SUBCOMMANDS_JOBS", value_parser = 1..=8, default_value_t = 1)]
        jobs: u8,
        #[command(flatten)]
        verbosity: Verbosity,
    },
    /// Manage remotes.
    Remote {
        /// Do not touch the network.
        #[arg(long)]
        offline: bool,
        #[command(subcommand)]
        command: Remote<'a>,
    },
    /// Print the status.
    Status,
    #[command(hide)]
    Debug,
}

/// A version control tool.
#[derive(Debug, PartialEq, GetArgs)]
#[command(name = "vcs", version = "1.0.0")]
struct Cli<'a> {
    /// Print less output.
    #[arg(short, long, global = true)]
    quiet: bool,
    /// Configuration file.
    #[arg(short, long, global = true)]
    config: Option<&'a str>,
    #[command(subcommand)]
    command: Commands<'a>,
}

#[derive(Debug, PartialEq, GetArgs)]
#[command(name = "tool")]
struct Optional<'a> {
    #[command(subcommand)]
    command: Option<Commands<'a>>,
}

fn parse<'a>(args: &[&'a str]) -> Result<Cli<'a>, Error<'a>> {
    Cli::from_args_with_env(args.iter().copied(), |_| None)
}

#[test]
fn parses_unit_variant() {
    let cli = parse(&["status"]).unwrap();
    assert_eq!(cli.command, Commands::Status);
    assert!(!cli.quiet);
}

#[test]
fn parses_named_variant() {
    let cli = parse(&["copy", "a.txt", "-m", "slow", "-vv"]).unwrap();
    assert_eq!(
        cli.command,
        Commands::Copy {
            source: "a.txt",
            mode: Mode::Slow,
            jobs: 1,
            verbosity: Verbosity { verbose: 2 },
        }
    );
}

#[test]
fn accepts_aliases() {
    let cli = parse(&["cp", "a.txt"]).unwrap();
    assert!(matches!(
        cli.command,
        Commands::Copy {
            source: "a.txt",
            ..
        }
    ));
    let cli = parse(&["remote", "rm", "origin"]).unwrap();
    assert_eq!(
        cli.command,
        Commands::Remote {
            offline: false,
            command: Remote::Remove { name: "origin" },
        }
    );
}

#[test]
fn parses_nested_and_newtype_variants() {
    let cli = parse(&[
        "remote",
        "--offline",
        "add",
        "origin",
        "https://example.com",
    ])
    .unwrap();
    assert_eq!(
        cli.command,
        Commands::Remote {
            offline: true,
            command: Remote::Add {
                name: "origin",
                url: "https://example.com",
            },
        }
    );
    let cli = parse(&["remote", "show", "origin"]).unwrap();
    assert_eq!(
        cli.command,
        Commands::Remote {
            offline: false,
            command: Remote::Show(Show {
                name: Some("origin")
            }),
        }
    );
}

#[test]
fn global_options_reach_the_parent() {
    let cli = parse(&["-c", "a.toml", "remote", "add", "o", "u", "--quiet"]).unwrap();
    assert!(cli.quiet);
    assert_eq!(cli.config, Some("a.toml"));
    let cli = parse(&["status", "--config=b.toml", "-q"]).unwrap();
    assert!(cli.quiet);
    assert_eq!(cli.config, Some("b.toml"));
}

#[test]
fn parent_options_are_not_global_by_default() {
    assert_eq!(
        parse(&["remote", "add", "o", "u", "--offline"]),
        Err(Error::UnknownOption {
            option: Opt::Long("offline"),
            suggestion: None,
        })
    );
}

#[test]
fn reads_env_inside_variants() {
    let cli = Cli::from_args_with_env(["copy", "a"].into_iter(), |name| {
        (name == "SUBCOMMANDS_JOBS").then_some("4")
    })
    .unwrap();
    assert!(matches!(cli.command, Commands::Copy { jobs: 4, .. }));
    assert_eq!(
        Cli::from_args_with_env(["copy", "a"].into_iter(), |_| Some("9")),
        Err(Error::ValueOutOfRange {
            name: "--jobs <JOBS> [env: SUBCOMMANDS_JOBS]",
            value: "9",
            range: "1..=8",
        })
    );
}

#[test]
fn reports_missing_and_unknown_subcommands() {
    assert_eq!(
        parse(&[]),
        Err(Error::MissingSubcommand(&["copy", "remote", "status"]))
    );
    assert_eq!(
        parse(&["remote"]),
        Err(Error::MissingSubcommand(&["add", "remove", "show"]))
    );
    assert_eq!(
        parse(&["stats"]),
        Err(Error::UnknownSubcommand {
            name: "stats",
            suggestion: Some("status"),
        })
    );
    assert_eq!(
        parse(&["remote", "rmv"]),
        Err(Error::UnknownSubcommand {
            name: "rmv",
            suggestion: Some("rm"),
        })
    );
    assert_eq!(
        parse(&["--", "status"]),
        Err(Error::UnexpectedArgument("status"))
    );
    assert_eq!(parse(&["copy"]), Err(Error::MissingArgument("<SOURCE>")));
    assert_eq!(
        parse(&["status", "extra"]),
        Err(Error::UnexpectedArgument("extra"))
    );
}

#[test]
fn hidden_subcommands_are_parsed() {
    assert_eq!(parse(&["debug"]).unwrap().command, Commands::Debug);
}

#[test]
fn optional_subcommand() {
    let tool = Optional::from_args_with_env([].into_iter(), |_| None).unwrap();
    assert_eq!(tool.command, None);
    let tool = Optional::from_args_with_env(["status"].into_iter(), |_| None).unwrap();
    assert_eq!(tool.command, Some(Commands::Status));
    assert!(Optional::HELP.starts_with("Usage: tool [OPTIONS] [COMMAND]\n"));
}

#[test]
fn root_help_lists_commands() {
    assert_eq!(
        Cli::HELP,
        "\
A version control tool

Usage: vcs [OPTIONS] <COMMAND>

Commands:
  copy    Copy files
  remote  Manage remotes
  status  Print the status

Options:
  -q, --quiet            Print less output
  -c, --config <CONFIG>  Configuration file
  -h, --help             Print help
  -V, --version          Print version"
    );
    assert_eq!(parse(&["-h"]), Err(Error::Help(Cli::HELP)));
    assert_eq!(parse(&["-V"]), Err(Error::Version("vcs 1.0.0")));
}

#[test]
fn subcommand_help() {
    let Err(Error::Help(help)) = parse(&["copy", "--help"]) else {
        panic!("expected help");
    };
    assert_eq!(
        help,
        "\
Copy files

Usage: vcs copy [OPTIONS] <SOURCE>

Arguments:
  <SOURCE>  Source file

Options:
  -m, --mode <MODE>      Copy mode [default: fast] [possible values: fast, slow]
  -j, --jobs <JOBS>      Number of jobs [env: SUBCOMMANDS_JOBS=] [default: 1]
  -v, --verbose...       More output
  -q, --quiet            Print less output
  -c, --config <CONFIG>  Configuration file
  -h, --help             Print help"
    );
}

#[test]
fn nested_subcommand_help() {
    let Err(Error::Help(help)) = parse(&["remote", "-h"]) else {
        panic!("expected help");
    };
    assert_eq!(
        help,
        "\
Manage remotes

Usage: vcs remote [OPTIONS] <COMMAND>

Commands:
  add     Add a remote
  remove  Remove a remote [aliases: rm]
  show    Show the remotes

Options:
      --offline          Do not touch the network
  -q, --quiet            Print less output
  -c, --config <CONFIG>  Configuration file
  -h, --help             Print help"
    );
    let Err(Error::Help(help)) = parse(&["remote", "show", "-h"]) else {
        panic!("expected help");
    };
    assert_eq!(
        help,
        "\
Show the remotes

Usage: vcs remote show [OPTIONS] [NAME]

Arguments:
  [NAME]  Remote to show

Options:
  -q, --quiet            Print less output
  -c, --config <CONFIG>  Configuration file
  -h, --help             Print help"
    );
}

#[derive(Debug, Subcommand)]
enum Build<'a> {
    /// Compile sources.
    Compile {
        /// Header directories.
        #[arg(short = 'I', long)]
        include: Values<&'a str, 4>,
        /// Source files.
        #[arg(num_args = 1..)]
        sources: Values<&'a str, 4>,
    },
}

#[derive(Debug, GetArgs)]
#[command(name = "make")]
struct Make<'a> {
    #[command(subcommand)]
    command: Build<'a>,
}

#[test]
fn collections_inside_variants() {
    let make =
        Make::from_args(["compile", "a.c", "-I", "inc", "b.c", "-Ivendor"].into_iter()).unwrap();
    let Build::Compile { include, sources } = make.command;
    assert_eq!(include, ["inc", "vendor"]);
    assert_eq!(sources, ["a.c", "b.c"]);
    assert_eq!(
        Make::from_args(["compile"].into_iter()).unwrap_err(),
        Error::MissingArgument("<SOURCES>...")
    );
}

#[test]
fn metadata_lists_subcommands() {
    let command = Cli::COMMAND;
    assert!(command.subcommand_required);
    let names: Vec<&str> = command
        .commands
        .iter()
        .map(|command| command.name)
        .collect();
    assert_eq!(names, ["copy", "remote", "status", "debug"]);
    let quiet = command.args.iter().find(|arg| arg.id == "quiet").unwrap();
    assert!(quiet.global);
    let copy = &command.commands[0];
    assert_eq!(copy.about, "Copy files");
    assert_eq!(copy.aliases, ["cp"]);
    assert!(copy.visible_aliases.is_empty());
    assert_eq!(copy.version, None);
    let ids: Vec<&str> = copy.args.iter().map(|arg| arg.id).collect();
    assert_eq!(ids, ["source", "mode", "jobs", "verbose", "help"]);
    assert!(command.commands[3].hidden);
    let remote = &command.commands[1];
    assert!(remote.subcommand_required);
    assert_eq!(remote.commands[1].visible_aliases, ["rm"]);
    assert_eq!(remote.commands[2].about, "Show the remotes");
    let optional = Optional::COMMAND;
    assert!(!optional.subcommand_required);
}
