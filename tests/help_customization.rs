#![allow(dead_code)]

use getargs_derive::{Error, GetArgs, ValueEnum};

#[derive(Debug, GetArgs)]
#[command(name = "net")]
struct Sections<'a> {
    /// Input file
    input: &'a str,
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    /// Remote host
    #[arg(long, help_heading = "Network")]
    host: Option<&'a str>,
    /// Secret switch
    #[arg(long, hide)]
    secret: bool,
    /// Hidden positional
    #[arg(hide)]
    extra: Option<&'a str>,
    /// Remote port
    #[arg(long, help_heading = "Network", default_value_t = 80)]
    port: u16,
    /// Log file
    #[arg(long, help_heading = "Logging")]
    log: Option<&'a str>,
    /// Quiet output
    #[arg(short, long, help_heading = "Options")]
    quiet: bool,
    /// Target directory
    #[arg(help_heading = "Network")]
    target: Option<&'a str>,
}

#[derive(Debug, GetArgs)]
#[command(name = "order")]
struct Ordered {
    #[arg(long)]
    alpha: bool,
    #[arg(long, display_order = 0)]
    beta: bool,
    #[arg(long, display_order = 10)]
    gamma: bool,
    #[arg(long)]
    delta: bool,
}

#[derive(Debug, GetArgs)]
#[command(name = "hidden", disable_help_flag)]
struct OnlyHidden {
    #[arg(long, hide)]
    debug: bool,
}

#[derive(Debug, GetArgs)]
#[command(
    name = "texts",
    about = "Short about",
    before_help = "Before the help",
    after_help = "After the help"
)]
struct Texts {
    #[arg(short, long)]
    verbose: bool,
}

/// Copy files around.
///
/// Files are copied
/// one by one.
///
/// Nothing is overwritten.
#[derive(Debug, GetArgs)]
#[command(name = "copy", version = "1.0.0", after_help = "See the manual")]
struct LongDoc<'a> {
    /// Input file.
    ///
    /// Use `-` to read from standard input.
    input: &'a str,
    /// Verbose output.
    #[arg(short, long)]
    verbose: bool,
    /// Number of jobs
    #[arg(
        short,
        long,
        default_value_t = 4,
        long_help = "Number of parallel jobs.\n\nUse 1 to copy sequentially."
    )]
    jobs: u32,
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, GetArgs)]
#[command(
    name = "about",
    about = "Short about",
    long_about = "Long about.\n\nWith a second paragraph."
)]
struct LongAbout {
    #[arg(short, long)]
    verbose: bool,
}

/// Single paragraph.
#[derive(Debug, GetArgs)]
#[command(name = "plain")]
struct NoLongHelp {
    /// Hidden field documentation.
    ///
    /// Not shown anywhere.
    #[arg(long, hide)]
    hidden: bool,
    /// Verbose output.
    #[arg(short, long)]
    verbose: bool,
}

/// Tool.
///
/// Longer description.
#[derive(Debug, GetArgs)]
#[command(name = "custom", disable_help_flag)]
struct LongHelpWithoutFlag {
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, GetArgs)]
#[command(
    name = "wrap",
    max_term_width = 40,
    about = "A tool whose about line is long enough to be wrapped at forty columns",
    after_help = "Examples:\n  wrap --level 3 --name something"
)]
struct Wrapped<'a> {
    /// Name of the thing to process, described at length
    #[arg(short, long)]
    name: &'a str,
    /// Short
    #[arg(short, long)]
    level: u8,
}

#[derive(Debug, GetArgs)]
#[command(name = "narrow", max_term_width = 40)]
struct NextLine<'a> {
    /// Configuration file used for every run of the program
    #[arg(long = "configuration-file", value_name = "PATH")]
    configuration_file: &'a str,
    #[arg(short, long)]
    verbose: bool,
}

/// Wrapped tool.
///
/// A long help paragraph that is wrapped at the configured width as well.
#[derive(Debug, GetArgs)]
#[command(name = "long-wrap", max_term_width = 30)]
struct LongWrapped {
    /// Verbose output.
    ///
    /// Prints every file as it is copied to the output.
    #[arg(short, long)]
    verbose: bool,
}

#[test]
fn help_headings_hide_and_merge_sections() {
    assert_eq!(
        Sections::HELP,
        "\
Usage: net [OPTIONS] <INPUT> [TARGET]

Arguments:
  <INPUT>  Input file

Options:
  -v, --verbose  Verbose output
  -q, --quiet    Quiet output
  -h, --help     Print help

Network:
      --host <HOST>  Remote host
      --port <PORT>  Remote port [default: 80]
  [TARGET]           Target directory

Logging:
      --log <LOG>  Log file"
    );
    assert!(core::ptr::eq(Sections::HELP, Sections::LONG_HELP));
}

#[test]
fn hidden_arguments_are_still_parsed() {
    let parsed =
        Sections::from_args(["in", "--secret", "hidden", "target", "--host", "h"].into_iter())
            .unwrap();
    assert!(parsed.secret);
    assert_eq!(parsed.extra, Some("hidden"));
    assert_eq!(parsed.target, Some("target"));
    assert_eq!(parsed.host, Some("h"));
    assert_eq!(parsed.port, 80);
}

#[test]
fn display_order_sorts_options() {
    assert_eq!(
        Ordered::HELP,
        "\
Usage: order [OPTIONS]

Options:
      --alpha
      --beta
      --delta
      --gamma
  -h, --help   Print help"
    );
}

#[test]
fn only_hidden_options_omit_options_tag() {
    assert_eq!(OnlyHidden::HELP, "Usage: hidden");
}

#[test]
fn before_and_after_help() {
    assert_eq!(
        Texts::HELP,
        "\
Before the help

Short about

Usage: texts [OPTIONS]

Options:
  -v, --verbose
  -h, --help     Print help

After the help"
    );
}

#[test]
fn short_help_uses_first_paragraphs() {
    assert_eq!(
        LongDoc::HELP,
        "\
Copy files around

Usage: copy [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input file

Options:
  -v, --verbose      Verbose output
  -j, --jobs <JOBS>  Number of jobs [default: 4]
      --dry-run
  -h, --help         Print help (see more with '--help')
  -V, --version      Print version

See the manual"
    );
}

#[test]
fn long_help_uses_full_doc_comments_and_long_help() {
    assert_eq!(
        LongDoc::LONG_HELP,
        "\
Copy files around.

Files are copied one by one.

Nothing is overwritten.

Usage: copy [OPTIONS] <INPUT>

Arguments:
  <INPUT>
          Input file.

          Use `-` to read from standard input.

Options:
  -v, --verbose
          Verbose output

  -j, --jobs <JOBS>
          Number of parallel jobs.

          Use 1 to copy sequentially.

          [default: 4]

      --dry-run

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

See the manual"
    );
}

#[test]
fn short_flag_returns_help_and_long_flag_returns_long_help() {
    assert_eq!(
        LongDoc::from_args(["-h"].into_iter()).unwrap_err(),
        Error::Help(LongDoc::HELP)
    );
    assert_eq!(
        LongDoc::from_args(["--help"].into_iter()).unwrap_err(),
        Error::Help(LongDoc::LONG_HELP)
    );
    assert_ne!(LongDoc::HELP, LongDoc::LONG_HELP);
}

#[test]
fn long_about_attribute() {
    assert_eq!(
        LongAbout::HELP,
        "\
Short about

Usage: about [OPTIONS]

Options:
  -v, --verbose
  -h, --help     Print help (see more with '--help')"
    );
    assert_eq!(
        LongAbout::LONG_HELP,
        "\
Long about.

With a second paragraph.

Usage: about [OPTIONS]

Options:
  -v, --verbose

  -h, --help
          Print help (see a summary with '-h')"
    );
}

#[test]
fn long_help_is_an_alias_without_long_text() {
    assert_eq!(
        NoLongHelp::HELP,
        "\
Single paragraph

Usage: plain [OPTIONS]

Options:
  -v, --verbose  Verbose output
  -h, --help     Print help"
    );
    assert!(core::ptr::eq(NoLongHelp::HELP, NoLongHelp::LONG_HELP));
    assert_eq!(
        NoLongHelp::from_args(["--help"].into_iter()).unwrap_err(),
        Error::Help(NoLongHelp::HELP)
    );
}

#[test]
fn long_help_without_help_flag() {
    assert_eq!(
        LongHelpWithoutFlag::HELP,
        "Tool\n\nUsage: custom [OPTIONS]\n\nOptions:\n  -v, --verbose"
    );
    assert_eq!(
        LongHelpWithoutFlag::LONG_HELP,
        "Tool.\n\nLonger description.\n\nUsage: custom [OPTIONS]\n\nOptions:\n  -v, --verbose"
    );
}

#[test]
fn wrapping_at_max_term_width() {
    assert_eq!(
        Wrapped::HELP,
        "\
A tool whose about line is long enough
to be wrapped at forty columns

Usage: wrap [OPTIONS]

Options:
  -n, --name <NAME>
          Name of the thing to process,
          described at length
  -l, --level <LEVEL>  Short
  -h, --help           Print help

Examples:
  wrap --level 3 --name something"
    );
    assert!(Wrapped::HELP.lines().all(|line| line.len() <= 40));
}

#[test]
fn wide_labels_move_descriptions_to_the_next_line() {
    assert_eq!(
        NextLine::HELP,
        "\
Usage: narrow [OPTIONS]

Options:
      --configuration-file <PATH>
          Configuration file used for
          every run of the program
  -v, --verbose
  -h, --help
          Print help"
    );
}

#[test]
fn wrapping_long_help() {
    assert_eq!(
        LongWrapped::HELP,
        "\
Wrapped tool

Usage: long-wrap [OPTIONS]

Options:
  -v, --verbose
          Verbose output
  -h, --help
          Print help (see more
          with '--help')"
    );
    assert_eq!(
        LongWrapped::LONG_HELP,
        "\
Wrapped tool.

A long help paragraph that is
wrapped at the configured
width as well.

Usage: long-wrap [OPTIONS]

Options:
  -v, --verbose
          Verbose output.

          Prints every file as
          it is copied to the
          output.

  -h, --help
          Print help (see a
          summary with '-h')"
    );
    assert!(LongWrapped::LONG_HELP.lines().all(|line| line.len() <= 30));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Color {
    Auto,
    Never,
}

#[derive(Debug, GetArgs)]
#[command(name = "values", max_term_width = 40)]
struct Values<'a> {
    /// Output color.
    ///
    /// Controls when the output is colored.
    #[arg(long, value_enum, default_value_t = Color::Auto)]
    color: Color,
    /// Compression level.
    #[arg(long, possible_values = ["low", "high"])]
    level: Option<&'a str>,
    #[arg(long, value_enum, hide)]
    secret: Option<Color>,
}

#[test]
fn possible_values_in_short_and_long_help() {
    assert_eq!(
        Values::HELP,
        "\
Usage: values [OPTIONS]

Options:
      --color <COLOR>  Output color [default: auto] [possible values: auto, never]
      --level <LEVEL>
          Compression level [possible
          values: low, high]
  -h, --help
          Print help (see more with
          '--help')"
    );
    assert_eq!(
        Values::LONG_HELP,
        "\
Usage: values [OPTIONS]

Options:
      --color <COLOR>
          Output color.

          Controls when the output is colored.

          [default: auto] [possible values: auto, never]

      --level <LEVEL>
          Compression level

          [possible values: low, high]

  -h, --help
          Print help (see a summary with
          '-h')"
    );
    assert!(!Values::HELP.contains("secret"));
    assert!(!Values::LONG_HELP.contains("secret"));
    assert_eq!(
        Values::from_args(["--secret", "never"].into_iter())
            .unwrap()
            .secret,
        Some(Color::Never)
    );
}
