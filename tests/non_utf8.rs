#![allow(dead_code)]

use getargs_derive::{Error, GetArgs, Lossy, Opt, Subcommand, ValueEnum, Values};

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum Mode {
    Fast,
    Slow,
}

#[derive(Debug, PartialEq, GetArgs)]
struct Cli<'a> {
    input: &'a [u8],
    #[arg(short, long)]
    name: Option<&'a str>,
    #[arg(short, long, default_value_t = 1)]
    count: u32,
    #[arg(short, long)]
    verbose: bool,
    #[arg(long, value_enum, default_value_t = Mode::Fast)]
    mode: Mode,
    #[arg(long, default_value = "raw")]
    tag: &'a [u8],
    #[arg(short = 'I', long, value_delimiter = ',')]
    include: Values<&'a [u8], 4>,
}

#[test]
fn utf8_bytes_parse_like_strings() {
    let args: [&[u8]; 8] = [
        b"in.txt",
        b"--name=alice",
        b"-c",
        b"3",
        b"-v",
        b"--mode",
        b"slow",
        b"-Ia,b",
    ];
    let cli = Cli::from_bytes(args.into_iter()).unwrap();
    assert_eq!(cli.input, b"in.txt");
    assert_eq!(cli.name, Some("alice"));
    assert_eq!(cli.count, 3);
    assert!(cli.verbose);
    assert_eq!(cli.mode, Mode::Slow);
    assert_eq!(cli.tag, b"raw");
    assert_eq!(cli.include, [&b"a"[..], b"b"]);
}

#[test]
fn byte_fields_borrow_invalid_utf8() {
    let args: [&[u8]; 4] = [b"caf\xE9", b"--tag=\xFF\xFE", b"-I", b"\xC0,ok"];
    let cli = Cli::from_bytes(args.into_iter()).unwrap();
    assert_eq!(cli.input, b"caf\xE9");
    assert_eq!(cli.tag, b"\xFF\xFE");
    assert_eq!(cli.include, [&b"\xC0"[..], b"ok"]);
}

#[test]
fn text_fields_reject_invalid_utf8() {
    let args: [&[u8]; 3] = [b"in", b"--name", b"caf\xE9"];
    let error = Cli::from_bytes(args.into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::InvalidUtf8 {
            name: Some("--name <NAME>"),
            value: b"caf\xE9",
        }
    );
    assert_eq!(
        error.to_string(),
        "invalid UTF-8 in value 'caf\\xE9' for '--name <NAME>'"
    );

    let args: [&[u8]; 3] = [b"in", b"-c", b"\xFF"];
    assert_eq!(
        Cli::from_bytes(args.into_iter()).unwrap_err(),
        Error::InvalidUtf8 {
            name: Some("--count <COUNT>"),
            value: b"\xFF",
        }
    );

    let args: [&[u8]; 2] = [b"in", b"--mode=\xFF"];
    assert_eq!(
        Cli::from_bytes(args.into_iter()).unwrap_err(),
        Error::InvalidUtf8 {
            name: Some("--mode <MODE>"),
            value: b"\xFF",
        }
    );
}

#[test]
fn unmatched_arguments_report_invalid_utf8() {
    let args: [&[u8]; 2] = [b"in", b"extra\xFF"];
    let error = Cli::from_bytes(args.into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::InvalidUtf8 {
            name: None,
            value: b"extra\xFF",
        }
    );
    assert_eq!(error.to_string(), "invalid UTF-8 in argument 'extra\\xFF'");

    let args: [&[u8]; 2] = [b"in", b"--bo\xFFgus"];
    assert_eq!(
        Cli::from_bytes(args.into_iter()).unwrap_err(),
        Error::InvalidUtf8 {
            name: None,
            value: b"bo\xFFgus",
        }
    );

    let args: [&[u8]; 2] = [b"in", b"extra"];
    assert_eq!(
        Cli::from_bytes(args.into_iter()).unwrap_err(),
        Error::UnexpectedArgument("extra")
    );

    let args: [&[u8]; 2] = [b"in", b"--verbose=\xFF"];
    assert_eq!(
        Cli::from_bytes(args.into_iter()).unwrap_err(),
        Error::UnexpectedValue(Opt::Long("verbose"))
    );
}

#[test]
fn non_ascii_short_options_are_bytes() {
    let args: [&[u8]; 2] = [b"in", "-é".as_bytes()];
    assert_eq!(
        Cli::from_bytes(args.into_iter()).unwrap_err(),
        Error::UnknownOption {
            option: Opt::Short(char::REPLACEMENT_CHARACTER),
            suggestion: None,
        }
    );
}

#[test]
fn help_and_parse_bytes() {
    let args: [&[u8]; 1] = [b"--help"];
    assert_eq!(
        Cli::from_bytes(args.into_iter()).unwrap_err(),
        Error::Help(Cli::LONG_HELP)
    );

    let args: [&[u8]; 2] = [b"in", b"-v"];
    let mut options = getargs::Options::new(args.into_iter());
    let cli = Cli::parse_bytes(&mut options).unwrap();
    assert!(cli.verbose);
}

#[test]
fn default_str_api_is_unchanged() {
    let cli = Cli::from_args(["in", "--tag", "x"].into_iter()).unwrap();
    assert_eq!(cli.input, b"in");
    assert_eq!(cli.tag, b"x");
}

#[derive(Debug, PartialEq, GetArgs)]
struct Server<'a> {
    #[arg(long, env = "NON_UTF8_ROOT")]
    root: &'a [u8],
    #[arg(long, env = "NON_UTF8_PORT", allow_negative_numbers)]
    port: i32,
}

#[test]
fn env_fallbacks_with_bytes() {
    let args: [&[u8]; 2] = [b"--port", b"-1"];
    let server = Server::from_bytes_with_env(args.into_iter(), |name| {
        (name == "NON_UTF8_ROOT").then_some("/srv")
    })
    .unwrap();
    assert_eq!(server.root, b"/srv");
    assert_eq!(server.port, -1);
}

#[derive(Debug, PartialEq, GetArgs)]
struct Shared<'a> {
    #[arg(long)]
    label: Option<&'a [u8]>,
}

#[derive(Debug, PartialEq, GetArgs)]
#[command(allow_negative_numbers)]
struct Numbers<'a> {
    first: i32,
    rest: Values<&'a [u8], 4>,
    #[command(flatten)]
    shared: Shared<'a>,
}

#[test]
fn negative_numbers_and_flatten_with_bytes() {
    let args: [&[u8]; 5] = [b"-3", b"-4", b"\xFF", b"--label", b"\xFE"];
    let numbers = Numbers::from_bytes(args.into_iter()).unwrap();
    assert_eq!(numbers.first, -3);
    assert_eq!(numbers.rest, [&b"-4"[..], b"\xFF"]);
    assert_eq!(numbers.shared.label, Some(&b"\xFE"[..]));
}

#[derive(Debug, PartialEq, Subcommand)]
enum Commands<'a> {
    Add {
        #[arg(short, long)]
        force: bool,
        path: &'a [u8],
    },
    Remove {
        name: &'a str,
    },
}

#[derive(Debug, PartialEq, GetArgs)]
struct Tool<'a> {
    #[arg(short, long, global)]
    quiet: bool,
    #[command(subcommand)]
    command: Commands<'a>,
}

#[test]
fn subcommands_with_bytes() {
    let args: [&[u8]; 4] = [b"add", b"-f", b"\xFF.txt", b"-q"];
    let tool = Tool::from_bytes(args.into_iter()).unwrap();
    assert!(tool.quiet);
    assert_eq!(
        tool.command,
        Commands::Add {
            force: true,
            path: b"\xFF.txt",
        }
    );

    let args: [&[u8]; 1] = [b"ad\xFF"];
    assert_eq!(
        Tool::from_bytes(args.into_iter()).unwrap_err(),
        Error::InvalidUtf8 {
            name: None,
            value: b"ad\xFF",
        }
    );

    let args: [&[u8]; 1] = [b"ad"];
    assert_eq!(
        Tool::from_bytes(args.into_iter()).unwrap_err(),
        Error::UnknownSubcommand {
            name: "ad",
            suggestion: Some("add"),
        }
    );

    let args: [&[u8]; 2] = [b"remove", b"\xFF"];
    assert_eq!(
        Tool::from_bytes(args.into_iter()).unwrap_err(),
        Error::InvalidUtf8 {
            name: Some("<NAME>"),
            value: b"\xFF",
        }
    );
}

#[derive(Debug, PartialEq, GetArgs)]
struct Limited<'a> {
    #[arg(long, possible_values = ["a", "b"])]
    choice: Option<&'a [u8]>,
    #[arg(long)]
    item: Values<&'a [u8], 1>,
}

#[test]
fn byte_fields_with_possible_values_and_limits() {
    let args: [&[u8]; 2] = [b"--choice", b"b"];
    assert_eq!(
        Limited::from_bytes(args.into_iter()).unwrap().choice,
        Some(&b"b"[..])
    );

    let args: [&[u8]; 2] = [b"--choice", b"\xFF"];
    assert_eq!(
        Limited::from_bytes(args.into_iter()).unwrap_err(),
        Error::InvalidUtf8 {
            name: Some("--choice <CHOICE>"),
            value: b"\xFF",
        }
    );

    let args: [&[u8]; 4] = [b"--item", b"a", b"--item", b"\xFF"];
    assert_eq!(
        Limited::from_bytes(args.into_iter()).unwrap_err(),
        Error::InvalidUtf8 {
            name: Some("--item <ITEM>"),
            value: b"\xFF",
        }
    );
}

#[test]
fn lossy_display() {
    assert_eq!(Lossy(b"").to_string(), "");
    assert_eq!(Lossy(b"a\xF0\x9F\x98b").to_string(), "a\\xF0\\x9F\\x98b");
    assert_eq!(Lossy("ü".as_bytes()).to_string(), "ü");
}

#[test]
fn non_exhaustive_match_needs_wildcard() {
    let error = Error::MissingArgument("<INPUT>");
    let message = match error {
        Error::MissingArgument(name) => name,
        _ => "other",
    };
    assert_eq!(message, "<INPUT>");
}

#[cfg(feature = "std")]
mod os {
    use getargs_derive::{Error, GetArgs};
    use std::ffi::{OsStr, OsString};
    use std::path::{Path, PathBuf};

    #[derive(Debug, PartialEq, GetArgs)]
    struct Files<'a> {
        input: PathBuf,
        #[arg(long)]
        raw: Option<&'a OsStr>,
        #[arg(long)]
        owned: Option<OsString>,
        #[arg(long)]
        path: Option<&'a Path>,
        #[arg(long, default_value = "out")]
        output: PathBuf,
        #[arg(long)]
        extra: Vec<PathBuf>,
    }

    #[test]
    fn os_fields_from_strings() {
        let files = Files::from_args(
            [
                "in", "--raw", "r", "--owned", "o", "--path", "p", "--extra", "a", "--extra", "b",
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(files.input, PathBuf::from("in"));
        assert_eq!(files.raw, Some(OsStr::new("r")));
        assert_eq!(files.owned, Some(OsString::from("o")));
        assert_eq!(files.path, Some(Path::new("p")));
        assert_eq!(files.output, PathBuf::from("out"));
        assert_eq!(files.extra, [PathBuf::from("a"), PathBuf::from("b")]);
    }

    #[derive(Debug, PartialEq, GetArgs)]
    #[command(disable_help_flag)]
    struct Empty {}

    #[test]
    fn from_args_os_skips_the_program_name() {
        let first: Result<Empty, Error<'static>> = Empty::from_args_os();
        assert_eq!(first.is_ok(), std::env::args_os().len() == 1);
        assert_eq!(Empty::from_args_os(), first);
    }

    #[cfg(unix)]
    #[test]
    fn os_fields_keep_invalid_utf8() {
        use std::os::unix::ffi::OsStrExt;

        let args: [&[u8]; 7] = [
            b"in\xFF",
            b"--raw",
            b"r\xFF",
            b"--owned=o\xFF",
            b"--path",
            b"p\xFF",
            b"--extra=a\xFF",
        ];
        let files = Files::from_bytes(args.into_iter()).unwrap();
        assert_eq!(files.input.as_os_str().as_bytes(), b"in\xFF");
        assert_eq!(files.raw, Some(OsStr::from_bytes(b"r\xFF")));
        assert_eq!(files.owned.as_deref(), Some(OsStr::from_bytes(b"o\xFF")));
        assert_eq!(files.path, Some(Path::new(OsStr::from_bytes(b"p\xFF"))));
        assert_eq!(files.extra, [PathBuf::from(OsStr::from_bytes(b"a\xFF"))]);
    }
}
