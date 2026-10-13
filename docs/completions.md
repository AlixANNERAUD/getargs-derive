# Shell Completions and Man Pages

Like [`clap_complete`](https://docs.rs/clap_complete) and [`clap_mangen`](https://docs.rs/clap_mangen), this crate can generate
bash, zsh and fish completion scripts and a man page from a `GetArgs` struct. This is a **Clap-inspired subset**:
the scripts complete option names, possible values and [subcommands][guide-subcommands], but have no dynamic completion or value hints.

## Command Metadata

`#[derive(GetArgs)]` implements the [`Command`](crate::Command) trait, whose `COMMAND` constant is a [`CommandInfo`](crate::CommandInfo)
describing the command: its name, `about` and `long_about` texts, version and arguments, flattened structs included.
Each argument is an [`ArgInfo`](crate::ArgInfo) with its kind, short and long names, visible and hidden aliases, value name, help,
possible values, environment variable and whether it is hidden, required, repeatable or `global`. The generated `--help` and `--version` flags are
listed last. `commands` lists the subcommands of a struct with a `#[command(subcommand)]` field, recursively, each a `CommandInfo` with
its own arguments (its `--help` flag included), aliases, `hidden` flag and `subcommand_required`; the `global` arguments of a parent are
not repeated in the arguments of its subcommands.

The metadata is made of `&'static` data built at compile time. It does not need the `std` feature and does not allocate:

```rust
use getargs_derive::{ArgKind, Command, GetArgs};

/// Copy files.
#[derive(GetArgs)]
#[command(name = "copy", version = "1.2.3")]
struct Cli<'a> {
    /// Input file.
    input: &'a str,
    /// Output file.
    #[arg(short, long, visible_alias = "destination")]
    output: Option<&'a str>,
}

let command = Cli::COMMAND;
assert_eq!(command.name, "copy");
assert_eq!(command.about, "Copy files");
assert_eq!(command.version, Some("1.2.3"));
let ids: Vec<&str> = command.args.iter().map(|arg| arg.id).collect();
assert_eq!(ids, ["input", "output", "help", "version"]);
assert_eq!(command.args[1].kind, ArgKind::Option);
assert_eq!(command.args[1].visible_aliases, ["destination"]);
```

`CommandInfo` and `ArgInfo` are `#[non_exhaustive]`: they are built with `new` and `const` builder methods, so that later releases can
describe more of the command without breaking code that reads them.

## Generating Scripts and Man Pages

With the `std` feature, the `getargs_derive::completions` module renders the metadata. `generate` writes the completion script for a
`Shell` and `man_page` writes a roff man page, to any `std::io::Write`:

```rust
# #[cfg(feature = "std")]
# fn main() -> std::io::Result<()> {
use getargs_derive::GetArgs;
use getargs_derive::completions::{Shell, generate, man_page};

#[derive(GetArgs)]
#[command(name = "copy")]
struct Cli<'a> {
    /// Output file.
    #[arg(short, long)]
    output: Option<&'a str>,
}

let mut script = Vec::new();
generate::<Cli>(Shell::Bash, &mut script)?;
assert!(String::from_utf8(script).unwrap().contains("complete -F _copy"));

let mut page = Vec::new();
man_page::<Cli>(&mut page)?;
assert!(String::from_utf8(page).unwrap().starts_with(".TH copy 1"));
# Ok(())
# }
# #[cfg(not(feature = "std"))]
# fn main() {}
```

`Shell` implements `FromStr` and `Display` with the names `bash`, `zsh` and `fish`, and `Shell::file_name` gives the usual file name of a
script (`copy.bash`, `_copy`, `copy.fish`). `Shell::render` and `render_man_page` return a `String` from a `CommandInfo` instead of writing it.
The scripts are usually written from a `build.rs` script or a small helper binary, and installed with the package.

## What Is Completed

As in Clap:

- hidden arguments (`hide`) are left out of the scripts and the man page;
- hidden aliases (`alias`, `short_alias`) are left out, while visible aliases are completed and listed in the man page;
- the values of `possible_values` and of `value_enum` arguments are offered as completions, for options and positional arguments;
- other values fall back to the shell's default completion, usually file names;
- `Count` flags and [repeated options][guide-collections] can be completed again after being given once;
- a positional collection keeps completing its values for every remaining positional argument, and is shown as `<NAME>...` or `[NAME]...` in the man page;
- subcommand names and visible aliases are completed where a subcommand is expected, hidden subcommands and hidden aliases are left out,
  and after a subcommand the scripts complete that subcommand's arguments followed by the `global` arguments of its parents.

The man page has the `NAME`, `SYNOPSIS`, `DESCRIPTION`, `ARGUMENTS`, `OPTIONS`, `SUBCOMMANDS` and `VERSION` sections. `SUBCOMMANDS` lists
every visible subcommand, nested ones included, with its full path, description, visible aliases and synopsis; there is no separate page
per subcommand. Arguments use their long help,
and list their environment variable and possible values. Default values, help headings and `before_help`/`after_help` are not rendered.

## Compile-Time Diagnostics

The metadata and the generators add no compile-time diagnostics: every `GetArgs` struct can be described and rendered.

[guide-collections]: collections.md
[guide-subcommands]: subcommands.md
