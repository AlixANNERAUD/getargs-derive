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

# fn main() -> Result<(), getargs_derive::Error> {
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

## Clap Compatibility

The field attributes follow the conventions of [Clap's derive API](https://docs.rs/clap/latest/clap/_derive/index.html):
`#[arg(short, long)]`, `short = 'x'`, `long = "name"`, `default_value` and `default_value_t` mean the same thing here as they do in Clap.

This crate only implements a **Clap-inspired subset**. It is not a drop-in replacement for Clap and does not aim for full Clap API compatibility:
container attributes, subcommands, `Option<T>` / `Vec<T>` fields, value enums, actions, and most other Clap attributes are not supported.
Unsupported attributes are rejected at compile time rather than silently ignored.

## Attribute Reference

All field attributes live in `#[arg(...)]`. Entries can be combined in one attribute (`#[arg(short, long, default_value_t = 4)]`) or split across several `#[arg(...)]` attributes on the same field.

| Attribute                         | Description                                                                                             |
| --------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `#[arg(short)]`                   | Named option with a short name inferred from the first character of the field name (`jobs` → `-j`).     |
| `#[arg(short = 'x')]`             | Named option with an explicit short name (`-x`).                                                        |
| `#[arg(long)]`                    | Named option with a long name inferred from the kebab-cased field name (`dry_run` → `--dry-run`).       |
| `#[arg(long = "name")]`           | Named option with an explicit long name (`--name`).                                                     |
| `#[arg(default_value = "text")]`  | Default given as a string literal. Used as-is for `&str` fields, otherwise parsed with `FromStr`.       |
| `#[arg(default_value_t = expr)]`  | Default given as a typed Rust expression of the field's type.                                           |

`short` and `long` can be combined: `#[arg(short, long)]` accepts both `-v` and `--verbose`.

### Argument Resolution

- A field is a **named option** if and only if it has `short` and/or `long`. It is only reachable through the names it declares: a field with `#[arg(short)]` has no long name, and vice versa.
- A named `bool` field is a **flag**: its presence sets it to `true`, and it defaults to `false`. It never consumes a value.
- A named field of any other type is an **option** taking a value (`-j 4`, `-j4`, `--jobs 4` or `--jobs=4`).
- Every other field is a **positional argument**, filled in declaration order. `bool` fields cannot be positional.

### Default Values

Use `default_value = "..."` when the default is most naturally written as command-line text, and `default_value_t = expr` when it is a typed value:

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Server<'a> {
    #[arg(long, default_value = "127.0.0.1")]
    host: &'a str,
    #[arg(short, long, default_value = "8080")]
    port: u16,
    #[arg(short, long, default_value_t = 4)]
    workers: u32,
}

# fn main() -> Result<(), getargs_derive::Error> {
let server = Server::from_args(["-p", "3000"].into_iter())?;
assert_eq!(server.host, "127.0.0.1");
assert_eq!(server.port, 3000);
assert_eq!(server.workers, 4);
# Ok(())
# }
```

A `default_value` string is only parsed when the argument is absent; if it does not parse, `parse` returns `Error::ParseError`.
A field may have at most one default.

Arguments without a default are required. Positional arguments with a default are optional, and must come after all required positional arguments:

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Copy<'a> {
    source: &'a str,
    #[arg(default_value = "out.txt")]
    destination: &'a str,
}

# fn main() -> Result<(), getargs_derive::Error> {
let copy = Copy::from_args(["in.txt"].into_iter())?;
assert_eq!(copy.destination, "out.txt");
# Ok(())
# }
```

### Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute or field, for:

- unknown `#[arg(...)]` keys, and keys given more than once on the same field;
- more than one default on the same field;
- `short` values that are not a character literal, or are `-` or whitespace;
- `long` values that are not a string literal, are empty, start with `-`, or contain whitespace or `=`;
- two fields using the same short or long name;
- positional `bool` fields;
- a required positional argument following an optional one;
- legacy `flag` on a non-`bool` field or without `short`/`long`, and legacy `positional` combined with `short`/`long`;
- enums, unions, tuple structs and unit structs.

## Legacy Syntax

Earlier versions of this crate used a different attribute syntax. These forms are still accepted so existing code keeps compiling, but they are deprecated in favour of the attributes above:

| Legacy form                   | Replacement                                    | Notes                                                               |
| ----------------------------- | ---------------------------------------------- | ------------------------------------------------------------------- |
| `#[arg(default = expr)]`      | `#[arg(default_value_t = expr)]`               | Identical behaviour.                                                |
| `#[default(expr)]`            | `#[arg(default_value_t = expr)]`               | Identical behaviour.                                                |
| `#[default = expr]`           | `#[arg(default_value_t = expr)]`               | Identical behaviour.                                                |
| `#[arg(flag)]`                | nothing, `bool` named options are flags        | Only valid on `bool` fields that also have `short` and/or `long`.   |
| `#[arg(positional)]`          | nothing, fields without `short`/`long` are positional | Cannot be combined with `short` or `long`.                   |

### Migrating From the Legacy Syntax

Previously, every named option automatically received **both** a short name (first character of the field) and a long name (kebab-cased field name),
and `flag`, `default` or `#[default]` alone were enough to make a field a named option. This is no longer the case:
a field is named only when it declares `short` and/or `long`, and only the declared names are accepted. Migrate as follows:

| Before                                         | After                                               |
| ---------------------------------------------- | --------------------------------------------------- |
| `#[arg(flag)] verbose: bool`                   | `#[arg(short, long)] verbose: bool`                 |
| `#[arg(default = 4)] jobs: u32`                | `#[arg(short, long, default_value_t = 4)] jobs: u32` |
| `#[default(4)] jobs: u32`                      | `#[arg(short, long, default_value_t = 4)] jobs: u32` |
| `#[arg(short = 'o', default = "out")] output`  | `#[arg(short = 'o', long, default_value = "out")] output` |
| `#[arg(long = "mode", default = "fast")] mode` | `#[arg(short, long = "mode", default_value = "fast")] mode` |
| `#[arg(positional)] input: &'a str`            | `input: &'a str`                                    |

Keep only `short` or only `long` if you do not want both names. Note the behaviour changes when migrating code that relied on the old inference:

- `#[arg(flag)]` without `short`/`long` is now a compile error asking you to add them.
- A legacy default (`#[arg(default = ...)]`, `#[default(...)]`) without `short`/`long` no longer creates a named option: the field becomes an **optional positional argument**.
  Add `short` and/or `long` to keep it a named option.

## Supported Field Types

| Type                      | Positional | Named option | Notes                                                                       |
| ------------------------- | ---------- | ------------ | --------------------------------------------------------------------------- |
| `&str`                    | ✓          | ✓            | Borrows the argument string directly — no allocation.                       |
| `bool`                    | ✗          | ✓            | Flag, set to `true` by its presence.                                        |
| `NonZeroU*` / `NonZeroI*` | ✓          | ✓            | Parsed via `.parse()`. Required unless a default is given.                  |
| Any `FromStr` type        | ✓          | ✓            | Parsed via `.parse()`. Required unless a default is given.                  |

## Error Handling

The `Error` enum covers the following failure cases:

```rust
use getargs_derive::Error;

fn handle_error(err: Error) {
    match err {
        Error::MissingPositionalArgument(name) => {
            eprintln!("Missing required argument: --{name}");
        }
        Error::UnknownOption => {
            eprintln!("Unknown option");
        }
        Error::MissingOptionValue(name) => {
            eprintln!("Option '{name}' requires a value");
        }
        Error::ParseError(name) => {
            eprintln!("Failed to parse value for '{name}'");
        }
        Error::InvalidNumberOfArguments => {
            eprintln!("Too many positional arguments provided");
        }
    }
}
```

| Variant                                   | Cause                                                                                        |
| ----------------------------------------- | -------------------------------------------------------------------------------------------- |
| `MissingPositionalArgument(&'static str)` | A required named or positional argument was not provided on the command line.                |
| `UnknownOption`                           | An unrecognised option was given (e.g. `--bogus` or `-z`).                                   |
| `MissingOptionValue(&'static str)`        | A named option was given without a value (e.g. `--output` at end of args).                   |
| `ParseError(&'static str)`                | A value or `default_value` could not be parsed into the target field type (e.g. `"abc"` for a `u32`). |
| `InvalidNumberOfArguments`                | More positional arguments were provided than the struct defines.                             |

The name carried by an error is the long name for named options that have one, and the field name otherwise.

## Full Example

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Config<'a> {
    /// First positional argument (required).
    input: &'a str,
    /// Second positional argument (optional).
    #[arg(default_value = "out.txt")]
    output: &'a str,
    /// Verbose flag, `-v` or `--verbose`.
    #[arg(short, long)]
    verbose: bool,
    /// Number of worker threads, `-j` only.
    #[arg(short, default_value_t = 4)]
    jobs: u32,
    /// Mode string, `--mode` only.
    #[arg(long, default_value = "fast")]
    mode: &'a str,
    /// Archive flag with explicit names, `-x` or `--extract`.
    #[arg(short = 'x', long = "extract")]
    archive: bool,
}

# fn main() -> Result<(), getargs_derive::Error> {
let args = ["in.txt", "--verbose", "-j8", "--mode=deep", "-x"];
let config = Config::from_args(args.into_iter())?;
assert_eq!(config.input, "in.txt");
assert_eq!(config.output, "out.txt");
assert!(config.verbose);
assert_eq!(config.jobs, 8);
assert_eq!(config.mode, "deep");
assert!(config.archive);
# Ok(())
# }
```

## `no_std` Support

This crate is `#![no_std]`-compatible. It only depends on `core` and the `getargs` crate.

## License

Licensed under the <a href="LICENSE">MIT License</a>.

---

Developed by [Alix ANNERAUD](https://alix.anneraud.fr).
