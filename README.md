# getargs-derive

[![crates.io](https://img.shields.io/crates/v/getargs-derive.svg)](https://crates.io/crates/getargs-derive)
[![docs.rs](https://img.shields.io/docsrs/getargs-derive)](https://docs.rs/getargs-derive)
[![codecov](https://codecov.io/github/AlixANNERAUD/getargs-derive/graph/badge.svg?token=0C27GIS0BK)](https://codecov.io/github/AlixANNERAUD/getargs-derive)

A `#[derive(GetArgs)]` macro for the [`getargs`](https://crates.io/crates/getargs) crate. Automatically generates argument parsing code for your structs, mapping CLI arguments to named fields with minimal boilerplate.

## Installation

```toml
[dependencies]
getargs-derive = "0.1"
getargs = "0.5"
```

## Quick Start

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    /// A required positional argument.
    input: &'a str,

    /// A flag enabled by `-v` or `--verbose` (defaults to `false`).
    #[arg(short, long)]
    verbose: bool,

    /// An option taking a value, given as `-o <value>` or `--output <value>`.
    #[arg(short, long, default_value = "output.txt")]
    output: &'a str,
}

# fn main() -> Result<(), getargs_derive::Error<'static>> {
let args = ["input.txt", "--verbose"];
let cli = Cli::from_args(args.into_iter())?;
assert_eq!(cli.input, "input.txt");
assert!(cli.verbose);
assert_eq!(cli.output, "output.txt");
# Ok(())
# }
```

The `from_args` method takes any iterator of `&str` and handles `getargs::Options` internally.
If you need more control (e.g. reusing an existing `Options`), use `parse` directly.

## Guide

- [Attribute Reference][guide-attributes]: `#[arg(...)]` and `#[command(...)]` attributes, argument resolution and supported field types.
- [Flag Actions][guide-actions]: `SetTrue`, `SetFalse` and `Count` flags.
- [Argument Aliases][guide-aliases]: hidden and visible long and short aliases.
- [Value Enums][guide-value-enum]: `#[derive(ValueEnum)]` and `#[arg(value_enum)]` for arguments accepting a fixed set of values.
- [Default Values][guide-defaults]: `default_value` and `default_value_t`.
- [Environment Variables][guide-env]: `env` fallbacks, `parse_with_env` and the `std` feature.
- [Positional and Optional Arguments][guide-positionals]: positional arguments and `Option<T>` fields.
- [Repeated Values][guide-collections]: repeated options, `value_delimiter`, positionals collecting the remaining arguments with `num_args`, and the `Values`, `heapless::Vec` and `Vec` storage options.
- [Argument Relations][guide-relations]: `conflicts_with`, `requires`, `required_unless_present` and argument groups.
- [Value Validation][guide-validation]: `value_parser` functions and ranges, and `possible_values`.
- [Parsing Modes][guide-parsing]: values starting with `-`, negative numbers and `last` positionals, and the limits imposed by `getargs`.
- [Flattened Structs][guide-flatten]: `#[command(flatten)]` to reuse the arguments of another struct.
- [Subcommands][guide-subcommands]: `#[derive(Subcommand)]`, `#[command(subcommand)]` fields, nested subcommands and `global` arguments.
- [Generated Help][guide-help]: the `HELP` and `LONG_HELP` constants, `-h`/`--help`, `disable_help_flag`, and help customization (hidden arguments, headings, ordering, extra text and wrapping).
- [Version Flag][guide-version]: `#[command(version)]`, `-V`/`--version` and the `VERSION`/`VERSION_TEXT` constants.
- [Error Handling][guide-errors]: the `Error` enum, its Clap-style messages and "did you mean" suggestions.
- [Shell Completions and Man Pages][guide-completions]: the `Command` metadata trait, and bash, zsh and fish completion scripts and man pages with the `std` feature.
- [Non-UTF-8 Arguments][guide-non-utf8]: `from_bytes` and `parse_bytes` for raw `&[u8]` arguments, `&[u8]`, `&OsStr` and `PathBuf` fields, and `from_args_os` with the `std` feature.
- [Clap Compatibility][guide-clap]: what is shared with Clap, and what is not supported.
- [`no_std` Support][guide-no-std].

Every guide page lists the compile-time diagnostics related to its topic.

## License

Licensed under the <a href="LICENSE">MIT License</a>.

---

Developed by [Alix ANNERAUD](https://alix.anneraud.fr).

[guide-attributes]: docs/attributes.md
[guide-actions]: docs/actions.md
[guide-aliases]: docs/aliases.md
[guide-value-enum]: docs/value_enum.md
[guide-defaults]: docs/defaults.md
[guide-env]: docs/env.md
[guide-positionals]: docs/positionals.md
[guide-collections]: docs/collections.md
[guide-relations]: docs/relations.md
[guide-validation]: docs/validation.md
[guide-parsing]: docs/parsing.md
[guide-flatten]: docs/flatten.md
[guide-subcommands]: docs/subcommands.md
[guide-help]: docs/help.md
[guide-version]: docs/version.md
[guide-errors]: docs/errors.md
[guide-completions]: docs/completions.md
[guide-clap]: docs/clap.md
[guide-no-std]: docs/no_std.md
[guide-non-utf8]: docs/non_utf8.md
