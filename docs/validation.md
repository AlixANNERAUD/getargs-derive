# Value Validation

Like Clap, the values of options and positional arguments can be parsed by a custom function, restricted to an integer range, or restricted to a
list of possible values. This is a Clap-inspired subset: `value_parser` accepts a function path or a range literal instead of Clap's
`value_parser!` builders, and possible values are listed with a dedicated `possible_values` key.

| Attribute                                  | Field type              | Meaning                                                                             |
| ------------------------------------------ | ----------------------- | ----------------------------------------------------------------------------------- |
| `#[arg(value_parser = path::to::parse)]`   | any `T`                 | Converts the value with `parse(&str) -> Result<T, E>` instead of `FromStr`.         |
| `#[arg(value_parser = 1..=64)]`            | integer                 | Parses the value with `FromStr`, then rejects it unless it is in the range.         |
| `#[arg(possible_values = ["a", "b"])]`     | `&str` or `FromStr`     | Rejects the value unless it is exactly one of the strings, before it is parsed.     |

All three apply to options and positional arguments, including `Option<T>` fields where they validate the inner `T`; they cannot be used on flags.

```rust
use getargs_derive::{Error, GetArgs};

fn parse_port(value: &str) -> Result<u16, core::num::ParseIntError> {
    value.strip_prefix(':').unwrap_or(value).parse()
}

#[derive(Debug, GetArgs)]
#[command(name = "serve")]
struct Cli<'a> {
    /// Port to listen on.
    #[arg(short, long, value_parser = parse_port, default_value = ":8080")]
    port: u16,
    /// Number of worker threads.
    #[arg(short, long, value_parser = 1..=64, default_value_t = 4)]
    jobs: u8,
    /// Scheduling mode.
    #[arg(short, long, possible_values = ["fast", "slow"], default_value = "fast")]
    mode: &'a str,
}

let cli = Cli::from_args(["--port", ":3000", "-j", "8", "-m", "slow"].into_iter()).unwrap();
assert_eq!((cli.port, cli.jobs, cli.mode), (3000, 8, "slow"));

let error = Cli::from_args(["--port", "http"].into_iter()).unwrap_err();
assert_eq!(error, Error::InvalidValue { name: "--port <PORT>", value: "http" });

let error = Cli::from_args(["-j", "0"].into_iter()).unwrap_err();
assert_eq!(error.to_string(), "invalid value '0' for '--jobs <JOBS>': 0 is not in 1..=64");

let error = Cli::from_args(["-m", "turbo"].into_iter()).unwrap_err();
assert_eq!(
    error.to_string(),
    "invalid value 'turbo' for '--mode <MODE>'\n  [possible values: fast, slow]"
);

assert_eq!(
    Cli::HELP,
    "Usage: serve [OPTIONS]

Options:
  -p, --port <PORT>  Port to listen on [default: :8080]
  -j, --jobs <JOBS>  Number of worker threads [default: 4]
  -m, --mode <MODE>  Scheduling mode [default: fast] [possible values: fast, slow]
  -h, --help         Print help"
);
```

## Parser Functions

`value_parser = path` takes the path of a function, or of anything callable as `path(value)`, with the signature
`fn(&str) -> Result<T, E>`, where `T` is the field type (the inner type for `Option<T>`). The error type `E` is free: its value is discarded,
because the crate does not allocate and `Error` cannot carry it, and the failure is reported as `Error::InvalidValue`.
Unlike `FromStr`, the function may borrow from the argument, so a `&'a str` field can be validated or trimmed without copying:

```rust
use getargs_derive::GetArgs;

fn non_empty(value: &str) -> Result<&str, ()> {
    match value.trim() {
        "" => Err(()),
        trimmed => Ok(trimmed),
    }
}

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(value_parser = non_empty)]
    name: &'a str,
}

let cli = Cli::from_args([" app "].into_iter()).unwrap();
assert_eq!(cli.name, "app");
assert!(Cli::from_args(["  "].into_iter()).is_err());
```

## Ranges

`value_parser` also accepts a range on an integer field (`u8`, `i64`, `usize`, ...): every range form is supported (`1..=64`, `1..65`, `1..`,
`..65`, `..=64`), and the bounds may be any constant expression of the field type, including negative numbers (`-5..=5`).
The value is first parsed with `FromStr`, which reports `Error::InvalidValue` when it is not a number, then checked against the range, which reports
`Error::ValueOutOfRange` with the range as written in the attribute. As in Clap, ranges are not shown in the generated help.
Ranges on other types are rejected; use a parser function to validate them.

## Possible Values

`possible_values = ["a", "b"]` lists the accepted strings. The value is compared with them exactly, before it is parsed, so it also works with
`FromStr` fields: `possible_values = ["1", "2", "4"]` on a `u8` field rejects `04`. A rejected value is reported as `Error::InvalidPossibleValue`,
which lists the possible values in its message like Clap. The generated help lists them after the default, as `[possible values: a, b]`, the same
format used for value enums; values containing whitespace are quoted.

## Combining and Defaults

The keys can be combined: the value is checked against `possible_values`, then converted with the parser function (or `FromStr`, or used as-is
for `&str`), then checked against the range. A field has at most one `value_parser`.

A `default_value` string goes through the same steps when the argument is absent, so an invalid default is reported at runtime with the same error,
except that a `default_value` that is not one of the `possible_values` is rejected at compile time. A `default_value_t` expression is a typed value
and is used as-is, without validation. A value read from an [environment variable][guide-env] also goes through the same steps, and its errors
name the variable, e.g. `"--port <PORT> [env: PORT]"`.

`#[arg(value_enum)]` fields already define their accepted values, so they cannot use `value_parser` or `possible_values`; see [Value Enums][guide-value-enum].

See [Error Handling][guide-errors] for the error variants.

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute, for:

- `value_parser` without a value, with a value that is neither a path nor a range, given more than once, or with a range without bounds (`..`);
- a `value_parser` range on a field whose type is not an integer;
- `possible_values` without an array of string literals, with an empty array, an empty string, or the same value twice;
- a `default_value` that is not one of the `possible_values`;
- `value_parser` or `possible_values` on a flag;
- `value_parser` or `possible_values` combined with `value_enum`.

A parser function with the wrong signature, or returning another type than the field's, is reported by the compiler as a type error.

[guide-env]: env.md
[guide-errors]: errors.md
[guide-validation]: validation.md
[guide-value-enum]: value_enum.md
