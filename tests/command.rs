#![allow(dead_code)]

use getargs_derive::{ArgInfo, ArgKind, Command, CommandInfo, GetArgs, ValueEnum, Values};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Color {
    Auto,
    Never,
}

#[derive(GetArgs)]
struct Verbosity {
    /// More output.
    #[arg(short, long, action = Count)]
    verbose: u8,
}

/// Copy files.
///
/// Copies the input to the output.
#[derive(GetArgs)]
#[command(name = "copy", version = "1.2.3")]
struct Cli<'a> {
    /// Input file.
    input: &'a str,
    /// Output file.
    #[arg(
        short,
        long,
        alias = "out",
        visible_alias = "destination",
        visible_short_alias = 'd',
        short_alias = 'O',
        value_name = "FILE"
    )]
    output: Option<&'a str>,
    /// When to color.
    #[arg(long, value_enum, default_value_t = Color::Auto, env = "COPY_COLOR")]
    color: Color,
    /// Copy mode.
    #[arg(long, possible_values = ["fast", "safe"])]
    mode: &'a str,
    #[arg(long, hide)]
    debug: bool,
    #[command(flatten)]
    verbosity: Verbosity,
    /// Extra arguments.
    #[arg(last)]
    rest: Option<&'a str>,
}

#[derive(GetArgs)]
#[command(name = "bare", disable_help_flag)]
struct Bare<T: core::str::FromStr> {
    value: T,
}

const fn info<C: Command>() -> CommandInfo {
    C::COMMAND
}

#[test]
fn command_metadata() {
    let command = Cli::COMMAND;
    assert_eq!(command.name, "copy");
    assert_eq!(command.about, "Copy files");
    assert_eq!(
        command.long_about,
        "Copy files.\n\nCopies the input to the output."
    );
    assert_eq!(command.version, Some("1.2.3"));
    assert!(command.commands.is_empty());
    let ids: Vec<&str> = command.args.iter().map(|arg| arg.id).collect();
    assert_eq!(
        ids,
        [
            "input", "output", "color", "mode", "debug", "verbose", "rest", "help", "version"
        ]
    );
}

#[test]
fn arg_metadata() {
    let args = Cli::COMMAND.args;
    assert_eq!(
        args[0],
        ArgInfo::new("input", ArgKind::Positional)
            .value_name("INPUT")
            .help("Input file")
            .required(true)
    );
    assert_eq!(
        args[1],
        ArgInfo::new("output", ArgKind::Option)
            .short('o')
            .long("output")
            .visible_short_aliases(&['d'])
            .short_aliases(&['O'])
            .visible_aliases(&["destination"])
            .aliases(&["out"])
            .value_name("FILE")
            .help("Output file")
    );
    assert_eq!(args[2].possible_values, Color::POSSIBLE_VALUES);
    assert_eq!(args[2].env, Some("COPY_COLOR"));
    assert!(!args[2].required);
    assert_eq!(args[3].possible_values, ["fast", "safe"]);
    assert!(args[3].required);
    assert!(args[4].hidden);
    assert_eq!(args[4].kind, ArgKind::Flag);
    assert!(!args[4].takes_value());
    assert!(args[5].multiple);
    assert!(args[6].last);
    assert!(args[6].is_positional());
    assert_eq!(args[7].short, Some('h'));
    assert_eq!(args[7].help, "Print help (see more with '--help')");
    assert_eq!(args[8].long, Some("version"));
}

#[test]
fn generic_command_metadata() {
    let command = info::<Bare<u8>>();
    assert_eq!(command.name, "bare");
    assert_eq!(command.version, None);
    assert_eq!(command.args.len(), 1);
    assert_eq!(command.args[0].id, "value");
}

#[test]
fn flattened_struct_metadata() {
    let command = Verbosity::COMMAND;
    assert_eq!(command.name, env!("CARGO_PKG_NAME"));
    let ids: Vec<&str> = command.args.iter().map(|arg| arg.id).collect();
    assert_eq!(ids, ["verbose", "help"]);
}

#[test]
fn info_builders() {
    const SUB: CommandInfo = CommandInfo::new("sub");
    const ROOT: CommandInfo = CommandInfo::new("root")
        .about("About")
        .long_about("Long about")
        .version("1.0.0")
        .commands(&[SUB]);
    assert_eq!(ROOT.commands, [SUB]);
    assert_eq!(ROOT.long_about, "Long about");
    let arg = ArgInfo::new("help", ArgKind::Flag).help("Short");
    assert_eq!(arg.long_or_short_help(), "Short");
    assert_eq!(arg.long_help("Long").long_or_short_help(), "Long");
}

#[derive(GetArgs)]
#[command(name = "cc")]
struct Compile<'a> {
    /// Header directories.
    #[arg(short = 'I', long, default_values = ["include"])]
    include: Values<&'a str, 4>,
    /// Source files.
    #[arg(num_args = 1..)]
    sources: Values<&'a str, 8>,
    /// Linker arguments.
    #[arg(last = true)]
    linker: Values<&'a str, 4>,
    #[command(flatten)]
    verbosity: Verbosity,
}

#[test]
fn collection_metadata() {
    let args = Compile::COMMAND.args;
    assert_eq!(
        args[0],
        ArgInfo::new("include", ArgKind::Option)
            .short('I')
            .long("include")
            .value_name("INCLUDE")
            .help("Header directories")
            .multiple(true)
    );
    assert_eq!(
        args[1],
        ArgInfo::new("sources", ArgKind::Positional)
            .value_name("SOURCES")
            .help("Source files")
            .required(true)
            .multiple(true)
    );
    assert!(args[2].multiple);
    assert!(args[2].last);
    assert!(!args[2].required);
    assert_eq!(args[3].id, "verbose");
    assert!(args[3].multiple);
    assert_eq!(args[4].id, "help");
    assert!(!args[4].multiple);
}

#[derive(GetArgs)]
struct Files<'a> {
    files: Values<&'a str, 8>,
}

#[test]
fn optional_positional_collection_metadata() {
    let files = Files::COMMAND.args[0];
    assert!(files.multiple);
    assert!(!files.required);
    assert!(files.is_positional());
}
