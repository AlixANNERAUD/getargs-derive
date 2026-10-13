# Value Enums

Like Clap, `#[derive(ValueEnum)]` turns an enum with unit variants into the set of values an argument accepts.
This is a **Clap-inspired subset** of Clap's `ValueEnum`: only the `name` and `alias` variant keys are supported, and matching is always case-sensitive.

```rust
use getargs_derive::{GetArgs, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Mode {
    Fast,
    /// Accepted as `slow-and-steady` or `careful`.
    #[value(alias = "careful")]
    SlowAndSteady,
    #[value(name = "turbo", alias = "t")]
    Maximum,
}

#[derive(GetArgs)]
#[command(name = "run")]
struct Cli {
    /// Execution mode.
    #[arg(short, long, value_enum, default_value_t = Mode::SlowAndSteady)]
    mode: Mode,
}

# fn main() -> Result<(), getargs_derive::Error<'static>> {
let cli = Cli::from_args(["--mode", "careful"].into_iter())?;
assert_eq!(cli.mode, Mode::SlowAndSteady);

let cli = Cli::from_args(["-mt"].into_iter())?;
assert_eq!(cli.mode, Mode::Maximum);

assert_eq!(
    Cli::HELP,
    "\
Usage: run [OPTIONS]

Options:
  -m, --mode <MODE>  Execution mode [default: slow-and-steady] [possible values: fast, slow-and-steady, turbo]
  -h, --help         Print help"
);
# Ok(())
# }
```

## Deriving `ValueEnum`

The derive implements the [`ValueEnum`](crate::ValueEnum) trait and [`FromStr`](core::str::FromStr) for the enum:

- Each variant is accepted under its name converted to kebab-case (`SlowAndSteady` → `slow-and-steady`, `HTTPServer` → `http-server`).
- `#[value(name = "...")]` replaces that name.
- `#[value(alias = "...")]` accepts an additional spelling. It can be repeated, and aliases are not listed in the help.
- `ValueEnum::POSSIBLE_VALUES` is a `&'static [&'static str]` holding the name of every variant in declaration order, and `ValueEnum::VARIANTS` the variants themselves.
- `ValueEnum::value_name` returns the name of a variant, and `ValueEnum::from_value` the variant matching a name or alias.
- `FromStr` fails with [`ParseValueEnumError`](crate::ParseValueEnumError) when the string matches no name or alias.

```rust
use getargs_derive::ValueEnum;

#[derive(Debug, PartialEq, ValueEnum)]
enum Level {
    Low,
    High,
}

assert_eq!(Level::POSSIBLE_VALUES, &["low", "high"]);
assert_eq!(Level::High.value_name(), "high");
assert_eq!("low".parse(), Ok(Level::Low));
assert!("LOW".parse::<Level>().is_err());
```

Nothing is allocated: every name is a `&'static str`, so the derive works in `no_std` crates.

## Value Enum Fields

As in Clap, a field opts in with `#[arg(value_enum)]`, which works on positional arguments, named options and `Option<T>` fields:

- The value is parsed with `ValueEnum::from_value`, and an unknown value is reported as [`Error::InvalidValue`](crate::Error::InvalidValue).
- The generated help lists the possible values as `[possible values: a, b, c]`, after the default if there is one.
- A `default_value_t` naming a variant, such as `Mode::SlowAndSteady`, is shown in the help as the variant's name (`[default: slow-and-steady]`). A `default_value` is shown as written, and parsed with `ValueEnum::from_value`.

`HELP` stays a `&'static str` built at compile time: the possible values come from the trait's `POSSIBLE_VALUES` constant and are joined by a `const fn`, without allocation.
Because of that, a `value_enum` field type must not use the struct's generic parameters.

Without `value_enum`, a `ValueEnum` field is still parsed through its `FromStr` implementation, but its possible values are not listed in the help.

The trait can also be implemented by hand, for instance on a type that is not an enum. A `default_value_t` that is not a variant of a derived enum is then shown as its Rust expression.

## Compile-Time Diagnostics

The macros report a compile error, pointing at the offending item or attribute, for:

- `ValueEnum` on a struct, a union, a generic enum or an enum without variants;
- variants with fields;
- unknown or repeated `#[value(...)]` keys, empty names and aliases, and `#[value(...)]` on the enum itself;
- two variants sharing a name or alias;
- `value_enum` with a value, or on a flag;
- a `value_enum` field whose type does not implement `ValueEnum`.

[guide-value-enum]: value_enum.md
