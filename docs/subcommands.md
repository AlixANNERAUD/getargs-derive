# Subcommands

Like Clap, an enum deriving `Subcommand` describes the subcommands of a program, and a field marked `#[command(subcommand)]` in a struct
deriving `GetArgs` receives the one given on the command line:

```rust
use getargs_derive::{Error, GetArgs, Subcommand};

#[derive(Debug, PartialEq, GetArgs)]
struct Push<'a> {
    /// Remote to push to.
    #[arg(default_value = "origin")]
    remote: &'a str,
    /// Push even when the remote has diverged.
    #[arg(short, long)]
    force: bool,
}

#[derive(Debug, PartialEq, Subcommand)]
enum Commands<'a> {
    /// Record changes.
    #[command(alias = "ci")]
    Commit {
        /// Commit message.
        #[arg(short, long)]
        message: &'a str,
    },
    /// Update the remote.
    Push(Push<'a>),
    /// Show the working tree status.
    Status,
}

/// A version control tool.
#[derive(Debug, PartialEq, GetArgs)]
#[command(name = "vcs")]
struct Cli<'a> {
    /// Print less output.
    #[arg(short, long, global = true)]
    quiet: bool,
    #[command(subcommand)]
    command: Commands<'a>,
}

# fn main() -> Result<(), Error<'static>> {
let cli = Cli::from_args(["ci", "-m", "Fix the parser", "--quiet"].into_iter())?;
assert!(cli.quiet);
assert_eq!(cli.command, Commands::Commit { message: "Fix the parser" });

let cli = Cli::from_args(["push", "-f"].into_iter())?;
assert_eq!(cli.command, Commands::Push(Push { remote: "origin", force: true }));

assert_eq!(
    Cli::HELP,
    "\
A version control tool

Usage: vcs [OPTIONS] <COMMAND>

Commands:
  commit  Record changes
  push    Update the remote
  status  Show the working tree status

Options:
  -q, --quiet  Print less output
  -h, --help   Print help"
);

let error = Cli::from_args(["push", "--help"].into_iter()).unwrap_err();
assert_eq!(
    error,
    Error::Help(
        "\
Update the remote

Usage: vcs push [OPTIONS] [REMOTE]

Arguments:
  [REMOTE]  Remote to push to [default: origin]

Options:
  -f, --force  Push even when the remote has diverged
  -q, --quiet  Print less output
  -h, --help   Print help"
    )
);

let error = Cli::from_args(["stats"].into_iter()).unwrap_err();
assert_eq!(error, Error::UnknownSubcommand { name: "stats", suggestion: Some("status") });
assert_eq!(
    error.to_string(),
    "unrecognized subcommand 'stats'\n\n  tip: a similar subcommand exists: 'status'"
);
# Ok(())
# }
```

This is a **Clap-inspired subset**, and it stays `no_std` and allocation-free: the parsed subcommand is stored in the enum, and every help
text, those of the subcommands included, is built at compile time.

## Variants

Each variant is a subcommand. Three kinds of variants are supported:

- **Unit variants** (`Status`) take no arguments of their own.
- **Variants with named fields** (`Commit { message: &'a str }`) declare their arguments like the fields of a `GetArgs` struct: every
  `#[arg(...)]` key works there, and so do `#[command(flatten)]` and a nested `#[command(subcommand)]` field.
- **Newtype variants** (`Push(Push<'a>)`) wrap a struct deriving `GetArgs`, which can then also be parsed on its own.

The subcommand is named after the variant, in kebab case (`DryRun` becomes `dry-run`). These keys are accepted in `#[command(...)]` on a
variant:

| Key                                          | Meaning                                                                                   |
| -------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `name = "..."`                               | The name of the subcommand, instead of the kebab-cased variant name.                     |
| `alias = "..."`, `aliases = ["...", ...]`    | Other names accepted for the subcommand, hidden from the help.                            |
| `visible_alias`, `visible_aliases`           | Other names accepted for the subcommand, listed as `[aliases: ...]` in the help.          |
| `hide`                                       | Hides the subcommand from the `Commands` section, the completions and the man page. It is still accepted. |
| `about`, `long_about`                        | The description of the subcommand, instead of the doc comment.                            |
| `before_help`, `after_help`, `max_term_width`, `disable_help_flag`, `allow_negative_numbers`, `group` | As on a struct, for the subcommand help and arguments. Not accepted on newtype variants, whose struct carries them. |

The doc comment of a variant describes the subcommand. For a newtype variant without doc comment or `about`, the `about` and
`long_about` of the wrapped struct are used. The `name` and `version` of the wrapped struct are ignored: the subcommand is named after the
variant, and only the top-level command has a version flag.

## The Subcommand Field

A struct has at most one `#[command(subcommand)]` field. When its type is the enum itself, a subcommand is required, and its absence is
reported as `Error::MissingSubcommand`. When it is `Option<Enum>`, the subcommand is optional and the field is `None` without one.

The field can be declared anywhere in the struct. Its value is not an argument of the struct, so it takes no `#[arg(...)]` attribute and
cannot be referenced by relations.

## Parsing

- A positional argument that is the name or an alias of a subcommand selects it, even when a positional argument of the parent is still
  empty, as in Clap. Other positional arguments fill the positional arguments of the parent; once they are all filled, a positional
  argument that is not a subcommand is reported as `Error::UnknownSubcommand`, with the closest visible name or alias as suggestion.
- After `--`, every argument is positional and no subcommand is recognised.
- Everything after the subcommand name belongs to the subcommand: its options, its positional arguments and its own subcommands. The
  options of the parent are no longer accepted there, unless they are `global`.
- `-h` and `--help` after a subcommand return the help of that subcommand, through `Error::Help`. `-V` and `--version` are only handled
  before the subcommand.
- Environment variable fallbacks, defaults, validation and relations of a subcommand are checked once the whole command line is read,
  after those of the parent.

## Global Arguments

As in Clap, an option or a flag of the parent marked `#[arg(global = true)]` is also accepted after its subcommands, nested ones
included. **Its value is stored in the field of the parent that declares it**, wherever it was given on the command line: in the example
above, `vcs ci -m "..." --quiet` sets `cli.quiet`. The subcommands do not get a copy of it.

An argument given after the subcommand is first offered to the subcommand, then to the global arguments of the closest parent, then to
those of the parents above it. A global argument whose short or long name is also used by a subcommand, nested ones included, is rejected
at compile time, so this order never matters. Global arguments of a flattened struct are global too.

Global arguments are listed in the `Options` section of the help of every subcommand, after the subcommand's own arguments, are marked
`global` in the [command metadata][guide-completions] and are completed after the subcommands.

## Help

The help of a struct with subcommands shows `<COMMAND>` in its usage line (`[COMMAND]` when the subcommand is optional), after its
positional arguments, and lists the visible subcommands with their short description in a `Commands:` section, before `Arguments:`.
Visible aliases are appended as `[aliases: ...]`.

Each subcommand has its own `&'static str` short and long help, returned in `Error::Help` for `-h` and `--help` after the subcommand.
Its usage line starts with the full command path, e.g. `Usage: vcs remote add [OPTIONS] <NAME>`, and it lists the global arguments of its
parents. The `HELP` and `LONG_HELP` constants of the top-level struct are the help of the command itself.

## Errors

| Variant                                          | Cause                                                                 | `Display`                                                         |
| ------------------------------------------------ | --------------------------------------------------------------------- | ----------------------------------------------------------------- |
| `MissingSubcommand(&'static [&'static str])`     | A required subcommand was not given. Lists the visible subcommands.   | `a subcommand is required but one was not provided`, followed by a `[subcommands: ...]` line |
| `UnknownSubcommand { name, suggestion }`         | A positional argument is neither a subcommand nor accepted by a positional argument of the parent. | `unrecognized subcommand 'stats'`, followed by a `tip:` line when there is a suggestion |

Errors of a subcommand's own arguments are the usual ones: `vcs push --bogus` is an `Error::UnknownOption`, and suggestions for unknown
long options of a subcommand are taken from its own long names. See [Error Handling][guide-errors].

## Metadata, Completions and Man Pages

The [`CommandInfo`](crate::CommandInfo) of a struct with subcommands lists them in `commands`, recursively, with their name, aliases,
descriptions, arguments (their own `--help` flag included, without `--version`) and whether they are hidden or require a subcommand
themselves. The bash, zsh and fish completion scripts complete subcommand names and the arguments of each subcommand, global arguments
included, and the man page has a `SUBCOMMANDS` section with the synopsis of each visible subcommand. See
[Shell Completions and Man Pages][guide-completions].

## Differences From Clap

This is a Clap-inspired subset. There is no generated `help` subcommand, and `-V`/`--version` are not propagated to subcommands.
Clap's `subcommand_required`, `arg_required_else_help`, `args_conflicts_with_subcommands`, `subcommand_precedence_over_arg`,
`subcommand_negates_reqs`, `infer_subcommands`, `allow_external_subcommands`, `short_flag`/`long_flag` subcommands,
`#[command(subcommand)]` on an enum variant field other than a named field, `#[command(flatten)]` on enum variants, and generic
subcommand enums are not supported. Subcommands cannot be flattened: a struct with a `#[command(subcommand)]` field cannot be used
in `#[command(flatten)]`. Global arguments must be options or flags, and their values are only available in the struct declaring them.
A subcommand enum used by several parents gets the help of each parent's command path; recursive subcommand types are not supported.

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending item or attribute, for:

- `#[derive(Subcommand)]` on a struct or a union, on an enum without variants, or on an enum with type or const parameters;
- `#[command(...)]` or `#[arg(...)]` on the enum itself;
- a tuple variant with more than one field, and attributes on the field of a newtype variant;
- an unknown `#[command(...)]` key on a variant, such as `version`, and the struct-level keys (`before_help`, `group`, ...) on a newtype
  variant;
- a subcommand name or alias that is empty, starts with `-` or contains whitespace, or that is already used by another variant;
- `hide` with a value, and alias keys whose value is not a string or an array of strings;
- more than one `#[command(subcommand)]` field in a struct, `#[command(subcommand)]` with a value, combined with `flatten` or with
  `#[arg(...)]`, on an `Option<Option<T>>` field, or in a struct with type or const parameters;
- a `#[command(subcommand)]` field whose type does not derive `Subcommand`, and a newtype variant whose type does not derive `GetArgs`;
- `#[command(flatten)]` on a struct that has a `#[command(subcommand)]` field;
- a `global` argument whose short or long name is also used by a subcommand, nested ones included;
- `global` on a positional argument, or with a value that is not a boolean literal;
- the errors of the arguments of a variant with named fields, as for a `GetArgs` struct;
- `parse` or `from_args` without the `std` feature when a subcommand has `env` fields.

The checks that look into other types (flattening, global names) are compile-time constant assertions, reported as `evaluation panicked`
errors.

[guide-completions]: completions.md
[guide-errors]: errors.md
[guide-subcommands]: subcommands.md
