# Attribute Reference

All field attributes live in `#[arg(...)]`. Entries can be combined in one attribute (`#[arg(short, long, default_value_t = 4)]`) or split across several `#[arg(...)]` attributes on the same field.

| Attribute                         | Description                                                                                             |
| --------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `#[arg(short)]`                   | Named option with a short name inferred from the first character of the field name (`jobs` → `-j`).     |
| `#[arg(short = 'x')]`             | Named option with an explicit short name (`-x`).                                                        |
| `#[arg(long)]`                    | Named option with a long name inferred from the kebab-cased field name (`dry_run` → `--dry-run`).       |
| `#[arg(long = "name")]`           | Named option with an explicit long name (`--name`).                                                     |
| `#[arg(default_value = "text")]`  | Default given as a string literal. Used as-is for `&str` fields, otherwise parsed with `FromStr`. See [Default Values][guide-defaults]. |
| `#[arg(default_value_t = expr)]`  | Default given as a typed Rust expression of the field's type. See [Default Values][guide-defaults].     |
| `#[arg(env = "NAME")]`          | Falls back to the environment variable `NAME` when absent from the command line. A bare `env` uses the upper-cased field name. See [Environment Variables][guide-env]. |
| `#[arg(help = "text")]`           | Description shown in the generated help. Overrides the field's doc comment. See [Generated Help][guide-help]. |
| `#[arg(value_name = "NAME")]`     | Placeholder shown in the generated help (`<NAME>`). Defaults to the upper-cased field name. Not allowed on flags. |
| `#[arg(value_enum)]`              | Field of a `ValueEnum` type: parsed by variant name, with its possible values listed in the help. See [Value Enums][guide-value-enum]. |
| `#[arg(value_parser = parse)]`    | Converts the value with a function `fn(&str) -> Result<T, E>` instead of `FromStr`. See [Value Validation][guide-validation]. |
| `#[arg(value_parser = 1..=64)]`   | Rejects integer values outside the range. See [Value Validation][guide-validation].                     |
| `#[arg(possible_values = ["a", "b"])]` | Rejects values that are not one of the strings, and lists them in the help. See [Value Validation][guide-validation]. |
| `#[arg(action = Count)]`          | Flag counting its occurrences on an integer field. See [Flag Actions][guide-actions].                   |
| `#[arg(action = SetFalse)]`       | Flag on a `bool` field that defaults to `true` and is set to `false` by its presence.                   |
| `#[arg(action = SetTrue)]`        | Explicit form of the default `bool` flag behaviour.                                                     |

`short` and `long` can be combined: `#[arg(short, long)]` accepts both `-v` and `--verbose`.

## Argument Resolution

- A field is a **named option** if and only if it has `short` and/or `long`. It is only reachable through the names it declares: a field with `#[arg(short)]` has no long name, and vice versa.
- A named `bool` field is a **flag**: its presence sets it to `true`, and it defaults to `false`. It never consumes a value.
- A named field with an `action` is a **flag** behaving as described in [Flag Actions][guide-actions].
- A named field of any other type is an **option** taking a value (`-j 4`, `-j4`, `--jobs 4` or `--jobs=4`).
- Every other field is a **positional argument**, filled in declaration order. `bool` fields cannot be positional.
- A field of type `Option<T>` is **optional**: it is `None` when absent, and `Some(value)` otherwise, with the value parsed as for a `T` field. See [Positional and Optional Arguments][guide-positionals].

## Supported Field Types

| Type                      | Positional | Named option | Notes                                                                       |
| ------------------------- | ---------- | ------------ | --------------------------------------------------------------------------- |
| `&str`                    | ✓          | ✓            | Borrows the argument string directly — no allocation.                       |
| `bool`                    | ✗          | ✓            | Flag, set to `true` by its presence (`false` with `action = SetFalse`).     |
| Integer with `action = Count` | ✗      | ✓            | Flag counting its occurrences.                                              |
| `NonZeroU*` / `NonZeroI*` | ✓          | ✓            | Parsed via `.parse()`. Required unless a default is given.                  |
| Any `FromStr` type        | ✓          | ✓            | Parsed via `.parse()`. Required unless a default is given.                  |
| Any type with `value_parser = function` | ✓ | ✓          | Converted by the function. See [Value Validation][guide-validation].        |
| `Option<T>`               | ✓          | ✓            | Optional, `None` when absent. `T` is any type above except `bool`.          |

## Container Attributes

The struct accepts a `#[command(...)]` attribute supporting this Clap-like subset only:

| Attribute                           | Description                                                                                                   |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `#[command(name = "tool")]`         | Program name shown in the usage line. Defaults to the package name (`CARGO_PKG_NAME`) of the crate deriving `GetArgs`. |
| `#[command(about = "text")]`        | Description shown at the top of the help. Overrides the struct's doc comment.                                 |
| `#[command(version)]`               | Enables `-V`/`--version` with the package version (`CARGO_PKG_VERSION`) of the crate deriving `GetArgs`. See [Version Flag][guide-version]. |
| `#[command(version = "1.2.3")]`     | Enables `-V`/`--version` with an explicit version.                                                            |
| `#[command(disable_help_flag)]`     | Disables `-h`/`--help` handling: they become ordinary unknown options, and fields may use them. See [Generated Help][guide-help]. |

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

# fn main() -> Result<(), getargs_derive::Error<'static>> {
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

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute or field, for:

- unknown `#[arg(...)]` keys, and keys given more than once on the same field;
- `short` values that are not a character literal, or are `-` or whitespace;
- `long` values that are not a string literal, are empty, start with `-`, or contain whitespace or `=`;
- two fields using the same short or long name;
- unknown or repeated `#[command(...)]` keys, and `value_name` on a flag;
- enums, unions, tuple structs and unit structs.

[guide-actions]: actions.md
[guide-defaults]: defaults.md
[guide-env]: env.md
[guide-help]: help.md
[guide-positionals]: positionals.md
[guide-validation]: validation.md
[guide-value-enum]: value_enum.md
[guide-version]: version.md
