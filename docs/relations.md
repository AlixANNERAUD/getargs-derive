# Argument Relations

Like Clap, arguments can declare relations with each other. Relations reference other arguments by their **field name**, and are checked at
compile time: a relation naming a field that does not exist is rejected. This is a Clap-inspired subset: the supported keys are listed below,
and relations cannot reference groups.

| Attribute                                   | Meaning                                                                                       |
| ------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `#[arg(conflicts_with = "other")]`          | The argument cannot be used together with `other`.                                            |
| `#[arg(requires = "other")]`                | When the argument is given, `other` must be given too.                                        |
| `#[arg(required_unless_present = "other")]` | The argument must be given unless `other` is. Only allowed on optional fields (`Option<T>`, a default, or a flag). |
| `#[arg(group = "id")]`                      | Makes the argument a member of the group `id`. See [Groups](#groups).                         |

`conflicts_with`, `requires` and `required_unless_present` can be repeated: a field conflicting with several arguments lists each one
(`conflicts_with = "a", conflicts_with = "b"`), a field requiring several arguments requires all of them, and a field with several
`required_unless_present` is required unless **any** of them is given (Clap's `required_unless_present_any`).

Relations only look at the command line: an argument is present when the user wrote it, so a default value never satisfies `requires`
and never triggers `conflicts_with`.

```rust
use getargs_derive::{Error, GetArgs};

#[derive(Debug, GetArgs)]
struct Cli<'a> {
    /// Read the configuration from this file.
    #[arg(short, long, required_unless_present = "stdin")]
    config: Option<&'a str>,
    /// Read the configuration from standard input.
    #[arg(long, conflicts_with = "config")]
    stdin: bool,
    /// Compress the output, which requires an output file.
    #[arg(long, requires = "output")]
    compress: bool,
    #[arg(short, long)]
    output: Option<&'a str>,
}

let cli = Cli::from_args(["--stdin"].into_iter()).unwrap();
assert!(cli.stdin);

let error = Cli::from_args(["--stdin", "-c", "app.toml"].into_iter()).unwrap_err();
assert_eq!(error.to_string(), "the argument '--stdin' cannot be used with '--config <CONFIG>'");

let error = Cli::from_args(["--stdin", "--compress"].into_iter()).unwrap_err();
assert_eq!(
    error,
    Error::MissingRequiredArgument { name: "--output <OUTPUT>", required_by: "--compress" }
);

let error = Cli::from_args([].into_iter()).unwrap_err();
assert_eq!(
    error.to_string(),
    "the following required arguments were not provided: <--config <CONFIG>|--stdin>"
);
```

## Groups

A group is a set of arguments sharing a name. Fields join a group with `#[arg(group = "id")]`; a field belongs to at most one group.
By default, as with Clap's `ArgGroup`, the members of a group are **mutually exclusive** and the group is **optional**.
The group can be configured on the struct with `#[command(group(...))]`:

| Key        | Meaning                                                    |
| ---------- | ---------------------------------------------------------- |
| `id = "id"`| The group name, required.                                  |
| `required` | At least one member of the group must be given.            |
| `multiple` | Several members of the group may be given together.        |

A group used by fields without a `#[command(group(...))]` declaration is an optional group of mutually exclusive arguments.

```rust
use getargs_derive::{Error, GetArgs};

#[derive(Debug, GetArgs)]
#[command(group(id = "format", required))]
struct Cli {
    #[arg(long, group = "format")]
    json: bool,
    #[arg(long, group = "format")]
    yaml: bool,
}

let cli = Cli::from_args(["--yaml"].into_iter()).unwrap();
assert!(cli.yaml);

let error = Cli::from_args(["--json", "--yaml"].into_iter()).unwrap_err();
assert_eq!(error, Error::ArgumentConflict { name: "--json", other: "--yaml" });

let error = Cli::from_args([].into_iter()).unwrap_err();
assert_eq!(error, Error::MissingOneOf(&["--json", "--yaml"]));
```

This differs from Clap's derive API, where groups are declared with an `ArgGroup` builder expression
(`#[command(group(ArgGroup::new("format").required(true)))]`) or from a struct with `#[group]`; here the declaration is a plain list of keys,
and groups cannot list their members with `args`.

## Errors

Relations are checked after all arguments are parsed, without allocation, in this order: conflicts (including exclusive groups), then
`required_unless_present` and required groups, then `requires`, then missing required arguments. The first violation is returned:

| Variant                                          | Display                                                                                  |
| ------------------------------------------------ | ---------------------------------------------------------------------------------------- |
| `ArgumentConflict { name, other }`               | `the argument '--stdin' cannot be used with '--config <CONFIG>'`                         |
| `MissingRequiredArgument { name, required_by }`  | `the following required argument was not provided: --output <OUTPUT> (required by '--compress')` |
| `MissingOneOf(names)`                            | `the following required arguments were not provided: <--config <CONFIG>\|--stdin>`        |

See [Error Handling][guide-errors].

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute, for:

- a relation referencing a field that does not exist, the field itself, or the same field twice;
- `conflicts_with`, `requires`, `required_unless_present` or `group` with an empty name or without a string literal;
- `requires` between two arguments that conflict, through `conflicts_with` or an exclusive group;
- `required_unless_present` on a required field;
- more than one `group` on the same field;
- a `#[command(group(...))]` without `id`, with an unknown key, declared twice, or with no member.

[guide-errors]: errors.md
