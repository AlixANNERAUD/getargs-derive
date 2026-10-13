#![cfg(feature = "std")]
#![allow(dead_code)]

use getargs_derive::completions::{Shell, generate, man_page, render_man_page};
use getargs_derive::{Command, GetArgs, ValueEnum};
use std::path::PathBuf;
use std::process;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Color {
    Auto,
    Always,
    Never,
}

#[derive(GetArgs)]
struct Verbosity {
    /// More output, repeat for more.
    #[arg(short, long, action = Count)]
    verbose: u8,
}

/// Copy files.
///
/// Copies the input to the output, keeping
/// the file's permissions.
///
/// .Lines starting with a dot are escaped.
#[derive(GetArgs)]
#[command(name = "copy", version = "1.2.3")]
struct Cli<'a> {
    /// Input file.
    input: &'a str,
    /// Copy mode.
    #[arg(possible_values = ["fast", "safe"])]
    mode: Option<&'a str>,
    /// Output file, `-` for "stdout".
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
    /// When to use [colors]: auto or not.
    ///
    /// The colors are disabled when the output is not a terminal.
    #[arg(long, value_enum, default_value_t = Color::Auto, env = "COPY_COLOR")]
    color: Color,
    /// Size in bytes.
    #[arg(short, long)]
    size: u64,
    /// Overwrite existing files.
    #[arg(short, long)]
    force: bool,
    /// Internal debugging.
    #[arg(long, hide)]
    debug: bool,
    #[command(flatten)]
    verbosity: Verbosity,
    /// Arguments passed through.
    #[arg(last)]
    rest: Option<&'a str>,
}

#[derive(GetArgs)]
#[command(name = "bare", disable_help_flag)]
struct Bare<'a> {
    value: &'a str,
}

fn snapshot(name: &str, actual: &str) {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "snapshots", name]
        .iter()
        .collect();
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "missing snapshot {}, run with UPDATE_SNAPSHOTS=1",
            path.display()
        )
    });
    assert_eq!(actual, expected, "snapshot {name} differs");
}

fn completion<C: Command>(shell: Shell) -> String {
    let mut output = Vec::new();
    generate::<C>(shell, &mut output).unwrap();
    String::from_utf8(output).unwrap()
}

fn syntax_check(program: &str, flag: &str, script: &str) {
    let directory = std::env::temp_dir().join(format!("getargs-derive-{}", process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join(format!("{program}-completion"));
    std::fs::write(&path, script).unwrap();
    let status = process::Command::new(program).arg(flag).arg(&path).status();
    std::fs::remove_file(&path).unwrap();
    if let Ok(status) = status {
        assert!(status.success(), "{program} rejected the generated script");
    }
}

#[test]
fn bash() {
    let script = completion::<Cli>(Shell::Bash);
    snapshot("copy.bash", &script);
    syntax_check("bash", "-n", &script);
    let bare = completion::<Bare>(Shell::Bash);
    snapshot("bare.bash", &bare);
    syntax_check("bash", "-n", &bare);
}

#[test]
fn zsh() {
    let script = completion::<Cli>(Shell::Zsh);
    snapshot("copy.zsh", &script);
    syntax_check("zsh", "-n", &script);
    let bare = completion::<Bare>(Shell::Zsh);
    snapshot("bare.zsh", &bare);
    syntax_check("zsh", "-n", &bare);
}

#[test]
fn fish() {
    let script = completion::<Cli>(Shell::Fish);
    snapshot("copy.fish", &script);
    syntax_check("fish", "--no-execute", &script);
    snapshot("bare.fish", &completion::<Bare>(Shell::Fish));
}

#[test]
fn man() {
    let mut output = Vec::new();
    man_page::<Cli>(&mut output).unwrap();
    let page = String::from_utf8(output).unwrap();
    assert_eq!(page, render_man_page(&Cli::COMMAND));
    snapshot("copy.1", &page);
    snapshot("bare.1", &render_man_page(&Bare::COMMAND));
}

#[test]
fn shell_names() {
    for (shell, name) in Shell::ALL.iter().zip(Shell::NAMES) {
        assert_eq!(shell.name(), *name);
        assert_eq!(shell.to_string(), *name);
        assert_eq!(name.parse::<Shell>(), Ok(*shell));
    }
    assert!("powershell".parse::<Shell>().is_err());
    assert_eq!(Shell::Bash.file_name("copy"), "copy.bash");
    assert_eq!(Shell::Zsh.file_name("copy"), "_copy");
    assert_eq!(Shell::Fish.file_name("copy"), "copy.fish");
}
