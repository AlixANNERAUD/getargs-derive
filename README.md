# getargs-derive

[![crates.io](https://img.shields.io/crates/v/getargs-derive.svg)](https://crates.io/crates/getargs-derive)
[![docs.rs](https://img.shields.io/docsrs/getargs-derive)](https://docs.rs/getargs-derive)
[![CI](https://github.com/AlixANNERAUD/getargs-derive/actions/workflows/check.yml/badge.svg)](https://github.com/AlixANNERAUD/getargs-derive/actions/workflows/check.yml)
[![codecov](https://codecov.io/github/AlixANNERAUD/getargs-derive/graph/badge.svg?token=0C27GIS0BK)](https://codecov.io/github/AlixANNERAUD/getargs-derive)
[![License: MIT](https://img.shields.io/crates/l/getargs-derive.svg)](https://github.com/AlixANNERAUD/getargs-derive/blob/main/LICENSE)
[![MSRV](https://img.shields.io/crates/msrv/getargs-derive.svg)](https://github.com/AlixANNERAUD/getargs-derive/blob/main/Cargo.toml)

**A Clap-style `#[derive]` command-line argument parser for `no_std` and embedded Rust.**
Describe your CLI as a struct, and getargs-derive generates a zero-allocation parser on top of
[`getargs`](https://crates.io/crates/getargs), with compile-time help, subcommands, helpful errors and shell completions.
If you know Clap's derive API, you already know this one.

## Why getargs-derive?

- **`no_std` and no allocation by default.** The parser only needs `core`: parsed values and errors borrow the arguments, and repeated values fit in fixed-capacity collections. It runs the same on a microcontroller, in a kernel or in a regular binary.
- **Help computed at compile time.** `--help`, `--version` and the help of every subcommand are `&'static str` constants built by the macro, so no help-formatting code ends up in your binary.
- **Clap-familiar attributes.** `#[arg(short, long, default_value_t = ...)]`, doc comments as help text, `#[derive(ValueEnum)]`, `#[derive(Subcommand)]`, `#[command(flatten)]`, `env`, `conflicts_with`, `requires`, argument groups, aliases, `Count` flags and more, with Clap's help layout and error wording.
- **Helpful errors.** Clap-style messages with "did you mean" suggestions for misspelled options and subcommands, and compile-time diagnostics for invalid attributes instead of runtime panics.
- **Shell completions and man pages.** With the `std` feature, generate bash, zsh and fish completion scripts and a roff man page from the same struct.
- **Small dependency tree.** At run time, only [`getargs`](https://crates.io/crates/getargs), which has no dependencies itself, plus [`heapless`](https://crates.io/crates/heapless) if you enable it.
- **Non-UTF-8 arguments.** Parse raw `&[u8]` arguments anywhere, and `OsString`/`PathBuf` fields with `std`.

## Installation

```sh
cargo add getargs-derive getargs
```

or in `Cargo.toml`:

```toml
[dependencies]
getargs-derive = "0.1"
getargs = "0.5"
```

No feature is enabled by default:

| Feature    | Enables                                                                                                              |
| ---------- | -------------------------------------------------------------------------------------------------------------------- |
| *(none)*   | Everything not listed below, in `no_std` without an allocator. Repeated values are stored in `Values<T, N>`.        |
| `heapless` | `heapless::Vec<T, N>` fields for repeated values, stored inline without allocation.                                  |
| `alloc`    | `Vec<T>` fields for repeated values.                                                                                 |
| `std`      | Environment variable fallbacks from the process environment, shell completions and man pages, `OsStr`/`Path`/`PathBuf` fields and `from_args_os`. Implies `alloc`. |

For a regular command-line application, enable `std`:

```sh
cargo add getargs-derive --features std
```

## Quick Start

```rust,no_run
use getargs_derive::{GetArgs, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Format {
    Json,
    Yaml,
    Toml,
}

/// Convert configuration files between formats.
#[derive(Debug, GetArgs)]
#[command(name = "confconv", version = "1.0.0")]
struct Cli<'a> {
    /// File to convert.
    input: &'a str,
    /// Output format.
    #[arg(short, long, value_enum, default_value_t = Format::Toml)]
    format: Format,
    /// Write the result to this file instead of standard output.
    #[arg(short, long)]
    output: Option<&'a str>,
    /// Print more details, repeat for even more.
    #[arg(short, long, action = Count)]
    verbose: u8,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = match Cli::from_args(args.iter().map(String::as_str)) {
        Ok(cli) => cli,
        Err(error) if error.is_help() || error.is_version() => {
            println!("{error}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    };
    println!("{cli:?}");
}
```

`confconv --help` prints the help generated at compile time, the `Cli::HELP` constant:

```text
Convert configuration files between formats

Usage: confconv [OPTIONS] <INPUT>

Arguments:
  <INPUT>  File to convert

Options:
  -f, --format <FORMAT>  Output format [default: toml] [possible values: json, yaml, toml]
  -o, --output <OUTPUT>  Write the result to this file instead of standard output
  -v, --verbose...       Print more details, repeat for even more
  -h, --help             Print help
  -V, --version          Print version
```

and a typo gets a suggestion:

```text
$ confconv config.json --frmat yaml
error: unexpected argument '--frmat' found

  tip: a similar argument exists: '--format'
```

The parser never prints or exits by itself: `from_args` takes any iterator of `&str` and returns the struct or an
[`Error`][guide-errors], so the same code works in tests, in `no_std` firmware reading arguments from a serial console, or behind `parse`
when you already have a `getargs::Options`. With the `std` feature, `Cli::from_args_os()` reads `std::env::args_os()` directly
and accepts [non-UTF-8 arguments][guide-non-utf8].

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    input: &'a str,
    #[arg(short, long)]
    verbose: bool,
    #[arg(short, long, default_value = "output.txt")]
    output: &'a str,
}

let cli = Cli::from_args(["input.txt", "--verbose"].into_iter()).unwrap();
assert_eq!(cli.input, "input.txt");
assert!(cli.verbose);
assert_eq!(cli.output, "output.txt");
```

## Compared to Clap

getargs-derive implements a **Clap-inspired subset** of Clap's derive API for programs where Clap does not fit, such as `no_std` targets.
It is not a drop-in replacement: [Clap Compatibility][guide-clap] lists what is shared and what is not supported.

|                                                   | getargs-derive                                                  | Clap                                   |
| ------------------------------------------------- | --------------------------------------------------------------- | -------------------------------------- |
| `no_std`, no allocator                            | yes                                                             | no, requires `std`                     |
| Help text                                         | `&'static str` built at compile time                            | built at run time                      |
| Derive API, doc comments as help                  | yes                                                             | yes                                    |
| Builder API                                       | no                                                              | yes                                    |
| Subcommands, flatten, `ValueEnum`                 | yes                                                             | yes                                    |
| Defaults, `env` fallbacks, aliases, `Count` flags | yes                                                             | yes                                    |
| Relations                                         | `conflicts_with`, `requires`, `required_unless_present`, groups | many more variants                     |
| Value validation                                  | parser functions, integer ranges, `possible_values`             | `value_parser!` and custom parsers     |
| "Did you mean" suggestions                        | yes                                                             | yes                                    |
| Non-UTF-8 arguments                               | yes                                                             | yes                                    |
| Shell completions                                 | bash, zsh, fish (`std`)                                         | more shells, dynamic completion        |
| Man pages                                         | yes (`std`)                                                     | with `clap_mangen`                     |
| Colored output, terminal-width wrapping           | no                                                              | yes                                    |
| Prints help and exits                             | no, your code decides                                           | yes, by default                        |

Unsupported Clap attributes are rejected at compile time rather than silently ignored.

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
- [`no_std` Support][guide-no-std]: allocation-free parsing and the crate features.

Every guide page lists the compile-time diagnostics related to its topic.

## Minimum Supported Rust Version

getargs-derive requires Rust 1.88 or newer, and is tested against it in CI.

## License

Licensed under the [MIT License](https://github.com/AlixANNERAUD/getargs-derive/blob/main/LICENSE).

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
