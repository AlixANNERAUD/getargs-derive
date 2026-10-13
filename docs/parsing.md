# Parsing Modes

By default, and as in Clap, an option taking a value refuses a separate value starting with `-`: in `--pattern -v`, `-v` looks like an option,
so the parser reports [`Error::MissingOptionValue`][guide-errors] (`a value is required for '--pattern <PATTERN>' but none was supplied`)
instead of silently swallowing it. A lone `-` (often used for standard input) is always accepted as a value, and so is a value attached to its option
(`--pattern=-v`, `-p-v`), because it cannot be mistaken for an option. Likewise, `-5` given where a positional argument is expected is read as the short
option `-5`. These attributes change that, as a Clap-inspired subset:

| Attribute                                | Applies to                          | Meaning                                                                                  |
| ---------------------------------------- | ----------------------------------- | ---------------------------------------------------------------------------------------- |
| `#[arg(allow_hyphen_values = true)]`     | options taking a value              | The option accepts the next argument as its value, whatever it starts with (`-v`, `--`, ...). |
| `#[arg(allow_negative_numbers = true)]`  | options taking a value, positionals | Negative numbers such as `-5`, `-1.5` or `-2e3` are accepted as values instead of options. |
| `#[command(allow_negative_numbers)]`     | the struct                          | Same as `allow_negative_numbers` on every option taking a value and every positional.    |
| `#[arg(last = true)]`                    | the final positional                | The positional is only filled by an argument given after `--`.                           |

Each key can be written alone (`#[arg(last)]`) or with a boolean literal (`#[arg(last = true)]`, `#[arg(last = false)]`).

```rust
use getargs_derive::{Error, GetArgs};

#[derive(GetArgs)]
#[command(allow_negative_numbers)]
struct Cli<'a> {
    /// Text to search for, which may start with `-`.
    #[arg(short, long, allow_hyphen_values = true)]
    pattern: Option<&'a str>,
    /// Shift applied to every match.
    #[arg(short, long, default_value_t = 0)]
    offset: i32,
    /// Starting point.
    start: Option<f64>,
}

# fn main() -> Result<(), Error<'static>> {
let cli = Cli::from_args(["--pattern", "-v", "-o", "-3", "-1.5"].into_iter())?;
assert_eq!(cli.pattern, Some("-v"));
assert_eq!(cli.offset, -3);
assert_eq!(cli.start, Some(-1.5));
# Ok(())
# }
```

## Hyphen Values

`allow_hyphen_values` makes the option take the next argument as its value unconditionally, so `--pattern --verbose` sets `pattern` to
`--verbose` rather than enabling `--verbose`, and `--pattern --` sets it to `--`. It implies `allow_negative_numbers` for that option.
It is only accepted on options taking a value: flags never take a value, and positional arguments cannot accept hyphen values (see
[Limitations](#limitations)).

## Negative Numbers

With `allow_negative_numbers`, an argument is a negative number when it is `-` followed by digits, with at most one `.` after the first
digit and an optional exponent (`e` or `E`, an optional sign, then digits): `-5`, `-1.5`, `-1.`, `-2e3` and `-2E-3` are numbers, `-.5`, `-x`,
`-inf` and `-1e` are not. Arguments that are not numbers keep their usual meaning: `--offset -x` is still refused, and `-v` is still a flag.
On a positional, a negative number takes precedence over a short option with the same name: with `#[arg(short = '1')]` on a flag, `-1`
fills the positional while it accepts negative numbers.

Only the positionals with `allow_negative_numbers` accept negative numbers: once they are filled, `-5` is an option again.
The number is checked syntactically, then parsed into the field type: `-1.5` for an `i32` positional is reported as
[`Error::InvalidValue`][guide-errors].

## Last Positional

A positional argument with `last = true` is only filled by an argument given after `--`, like the arguments passed through by `cargo run -- ...`.
Before `--`, an argument reaching it is reported as `Error::UnexpectedArgument`. After `--`, the next argument goes straight to it,
skipping any positional still empty, as in Clap. Only one positional can use `last`, and it must be declared after all other positionals.
It can be required: then `--` must be given. The usage line shows it as `[-- <NAME>]`, or `-- <NAME>` when required.

```rust
use getargs_derive::{Error, GetArgs};

#[derive(Debug, GetArgs)]
#[command(name = "run")]
struct Run<'a> {
    #[arg(short, long)]
    verbose: bool,
    program: Option<&'a str>,
    #[arg(last = true)]
    argument: Option<&'a str>,
}

# fn main() -> Result<(), Error<'static>> {
let run = Run::from_args(["cargo", "--", "--verbose"].into_iter())?;
assert_eq!(run.program, Some("cargo"));
assert_eq!(run.argument, Some("--verbose"));
assert!(!run.verbose);

let error = Run::from_args(["cargo", "build"].into_iter()).unwrap_err();
assert_eq!(error, Error::UnexpectedArgument("build"));

assert!(Run::HELP.starts_with("Usage: run [OPTIONS] [PROGRAM] [-- <ARGUMENT>]\n"));
# Ok(())
# }
```

## Global Arguments

Clap's `global = true` propagates an argument to subcommands. Subcommands are not supported yet, so `global` is not supported either and is
rejected as an unknown `#[arg(...)]` key.

## Limitations

`getargs` 0.5 tokenises every argument before the generated code sees it, and offers no way to peek at the raw argument or to push one back.
This shapes what can be supported:

- `-5` reaches the generated code as the short option `5`, and `-1.5` as the short option `1` followed by `.5`. To recover the original
  argument, `from_args` (and `from_args_with_env` for structs with [`env`][guide-env] fields) wraps the iterator to remember the last raw argument. A `getargs::Options` passed to
  `parse` or `parse_with_env` cannot be wrapped, so **with `parse`, positionals do not accept negative numbers**: `-5` is reported as an unknown option `-5`.
  Option values do not have this limitation, because `getargs` hands the next argument to an option as is.
- `allow_hyphen_values` is not supported on positionals: a positional could not tell `-v` meant as a value from the flag `-v` without the
  raw argument, and Clap's precedence rules between such positionals and known flags would not be reproducible. Use `last = true` and `--`
  to pass arbitrary values.
- A rejected hyphen value has already been consumed by `getargs`, so it cannot be reparsed as the option it looks like: parsing stops with
  `Error::MissingOptionValue`, as Clap does when the next argument is a known option.
- After `--`, `getargs` returns every remaining argument as a positional, so options, negative numbers and hyphen values need no special
  handling there.
- Clap's `allow_hyphen_values` on the whole command, `trailing_var_arg`, `raw` and `allow_missing_positional` are not supported.
  A `last` positional can collect several values with a collection type; see [Repeated Values][guide-collections].

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute, for:

- `allow_hyphen_values`, `allow_negative_numbers` or `last` given a value that is not a boolean literal, or given more than once;
- `allow_hyphen_values` on a flag or a positional argument;
- `allow_negative_numbers` on a flag;
- `last` on a named option or flag;
- `last` on more than one positional, or on a positional that is not the final one;
- `global`, which is not supported.

[guide-collections]: collections.md
[guide-env]: env.md
[guide-errors]: errors.md
[guide-parsing]: parsing.md
