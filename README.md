# getargs-derive

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

    /// An optional named flag (defaults to `false`).
    #[arg(flag)]
    verbose: bool,

    /// An optional named argument with a value.
    #[arg(short = 'o', default = "output.txt")]
    output: &'a str,
}

# fn main() -> Result<(), getargs_derive::Error> {
let args = ["input.txt", "--verbose"];
let mut options = getargs::Options::new(args.into_iter());
let cli = Cli::parse(&mut options)?;
assert_eq!(cli.input, "input.txt");
assert!(cli.verbose);
assert_eq!(cli.output, "output.txt");
# Ok(())
# }
```

## Attribute Reference

| Attribute                | Applies to          | Description                                                                                   |
| ------------------------ | ------------------- | --------------------------------------------------------------------------------------------- |
| `#[arg(positional)]`     | Any field           | Marks the field as a positional argument (default if no option attributes are given).         |
| `#[arg(flag)]`           | `bool` fields       | Marks the field as a boolean flag — no value is consumed, just `--name` toggles it to `true`. |
| `#[arg(short = 'x')]`    | Named options       | Custom short name (e.g. `-o`). Defaults to the first character of the field name.             |
| `#[arg(long = "name")]`  | Named options       | Custom long name (e.g. `--output`). Defaults to the kebab-cased field name.                   |
| `#[arg(default = expr)]` | Named or positional | Default value expression when the argument is not provided.                                   |
| `#[default(expr)]`       | Named or positional | Legacy syntax for default values (equivalent to `#[arg(default = expr)]`).                    |

### Argument Resolution

A field is treated as a **named option** (e.g. `--name`) if any of `#[arg(flag)]`, `#[arg(short)]`, `#[arg(long)]`, `#[arg(default)]`, or `#[default]` is present. Otherwise it is treated as a **positional argument**.

## Supported Field Types

| Type                      | Positional | Named option            | Notes                                                                                                 |
| ------------------------- | ---------- | ----------------------- | ----------------------------------------------------------------------------------------------------- |
| `&str`                    | ✓          | ✓                       | Borrows the argument string directly — no allocation.                                                 |
| `bool`                    | ✗          | ✓ (with `#[arg(flag)]`) | Toggled by the presence of the flag.                                                                  |
| `NonZeroU*` / `NonZeroI*` | ✓          | ✓                       | Parsed via `.parse()`. Defaults to `NonZero::MIN` (1 for unsigned) when no explicit default is given. |
| Any `FromStr` type        | ✓          | ✓                       | Parsed via `.parse()`. Requires a `#[default]` or is treated as required.                             |

## Error Handling

The `Error` enum covers three failure cases:

```rust
use getargs_derive::Error;

fn handle_error(err: Error) {
    match err {
        Error::MissingPositionalArgument(name) => {
            eprintln!("Missing required argument: --{name}");
        }
        Error::InvalidOption => {
            eprintln!("Invalid option or failed to parse option value");
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
| `InvalidOption`                           | An unrecognised option was given, or a value could not be parsed into the target field type. |
| `InvalidNumberOfArguments`                | More positional arguments were provided than the struct defines.                             |

## Full Example

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Config<'a> {
    /// First positional argument (required).
    input: &'a str,
    /// Second positional argument (required).
    output: &'a str,
    /// Verbose flag — defaults to `false`.
    #[arg(flag)]
    verbose: bool,
    /// Number of worker threads — defaults to 4.
    #[arg(short = 'j', default = 4)]
    jobs: u32,
    /// Optional mode string — defaults to "fast".
    #[arg(long = "mode", default = "fast")]
    mode: &'a str,
}

# fn main() -> Result<(), getargs_derive::Error> {
let args = ["in.txt", "out.txt", "--verbose", "-j8", "--mode=deep"];
let mut options = getargs::Options::new(args.into_iter());
let config = Config::parse(&mut options)?;
assert_eq!(config.input, "in.txt");
assert_eq!(config.output, "out.txt");
assert!(config.verbose);
assert_eq!(config.jobs, 8);
assert_eq!(config.mode, "deep");
# Ok(())
# }
```

## `no_std` Support

This crate is `#![no_std]`-compatible. It only depends on `core` and the `getargs` crate.

## License

Licensed under the <a href="LICENSE">MIT License</a>.
