# Flattened Structs

Like Clap, a field marked `#[command(flatten)]` takes its arguments from another struct deriving `GetArgs`. The flattened struct's options
and positional arguments are parsed as if they were declared in the parent, which makes it easy to share groups of arguments between
programs:

```rust
use getargs_derive::GetArgs;

#[derive(Debug, GetArgs)]
struct Verbosity {
    /// More output.
    #[arg(short, long, action = Count)]
    verbose: u8,
    /// Less output.
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,
}

/// Copy files around.
#[derive(Debug, GetArgs)]
#[command(name = "copy")]
struct Cli<'a> {
    /// Input file.
    input: &'a str,
    #[command(flatten)]
    verbosity: Verbosity,
}

let cli = Cli::from_args(["-vv", "notes.txt"].into_iter()).unwrap();
assert_eq!(cli.input, "notes.txt");
assert_eq!(cli.verbosity.verbose, 2);

assert_eq!(
    Cli::HELP,
    "\
Copy files around

Usage: copy [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input file

Options:
  -v, --verbose...  More output
  -q, --quiet       Less output
  -h, --help        Print help"
);
```

Flattened structs can themselves flatten other structs, and may borrow from the command line (`Network<'a>`) like any other struct.

## Behaviour

- **Options** of a flattened struct are accepted anywhere on the command line, like the parent's own options.
- **Positional arguments** are filled in field order, as in Clap: the positional arguments of a flattened struct take the place of the
  `#[command(flatten)]` field among the parent's positional fields. A struct with the fields `source` (flattened, one positional),
  `mode` (positional) and `target` (flattened, one positional) fills `source`, then `mode`, then `target`.
- **Defaults, required arguments and relations** of a flattened struct are handled by that struct. Relations cannot cross struct
  boundaries: `conflicts_with`, `requires`, `required_unless_present` and `group` only reference fields of the struct declaring them,
  and a flattened field is not an argument that relations can reference.
- **Attributes** of the fields of a flattened struct keep their meaning: [aliases][guide-aliases], [value validation][guide-validation],
  [value enums][guide-value-enum], [environment variables][guide-env] and [parsing modes][guide-parsing] work as if the fields were
  declared in the parent, and the parent's `#[command(allow_negative_numbers)]` also applies to them.
- **Help**: the parent's `HELP` and `LONG_HELP` list the arguments of flattened structs in field order, within the parent's sections and
  aligned with the parent's own arguments, exactly as if they were declared in the parent: `hide`, `help_heading`, `display_order`,
  `long_help` and multi-paragraph doc comments of flattened fields apply, `display_order` counts positions in the merged field list, and
  the parent's `max_term_width` wraps them. `HELP` and `LONG_HELP` remain `&'static str` constants computed at compile time.
- **Environment variables**: a struct that flattens other structs also gets `parse_with_env` and `from_args_with_env`, since its
  flattened structs may read environment variables. Its `parse` and `from_args` read the process environment for them, which needs the
  `std` feature of this crate: without it, using `parse` or `from_args` is a compile error when a flattened struct has `env` fields.
- **`last` positionals**: a `last` positional must be the final positional argument, flattened structs included. A parent's `last`
  positional receives the arguments given after `--`, skipping the positionals of flattened structs, as it skips the parent's own.
- **`-h`/`--help` and `-V`/`--version`** are handled by the parent only, according to its own `#[command(...)]` attributes. The
  `#[command(...)]` attributes of a flattened struct (`name`, `about`, `version`, `disable_help_flag`, `group`) only apply when that
  struct is parsed on its own; `group` declarations still apply to its own fields.
- **Errors** of a flattened struct use the names of its own arguments, e.g. `--port <PORT>`. Unknown long options are matched against all
  long names, flattened ones included, for "did you mean" suggestions.

## Differences From Clap

This is a Clap-inspired subset. Flattening `Option<T>` fields, field-level `#[command(...)]` keys other than `flatten` and `subcommand`
(such as `next_help_heading`), `#[group]` on flattened structs, flattening a struct with a `#[command(subcommand)]` field, and flattening
in structs with type or const parameters are not supported.
Flattened arguments do not get their own help heading. A short option of the parent is matched before the argument is offered to its
flattened structs, so it takes precedence over a negative number expected by a flattened positional, while within one struct a negative
number takes precedence over a short option with the same name.

## Compile-Time Diagnostics

The macro reports a compile error for:

- `#[command(flatten)]` on an `Option<T>` field;
- `#[command(flatten)]` with a value, an unknown field-level `#[command(...)]` key, `flatten` repeated, or `flatten` combined with
  `subcommand`;
- a flattened struct that has a `#[command(subcommand)]` field;
- `#[arg(...)]` on a flattened field;
- a flattened field whose type does not derive `GetArgs`;
- `#[command(flatten)]` in a struct with type or const parameters;
- a short or long name of a flattened struct that is already used by a field of the parent, by another flattened field, or by the
  parent's generated `-h`/`--help` or `-V`/`--version` flags;
- a required positional argument following an optional one, across flattened structs;
- a `last` positional argument followed by another positional argument, across flattened structs;
- `parse` or `from_args` without the `std` feature when a flattened struct has `env` fields.

Name and positional checks across struct boundaries are compile-time constant assertions: they are reported as `evaluation panicked`
errors pointing at the `#[command(flatten)]` attribute or at the offending field, and name the conflicting option when it belongs to
the parent. Conflicts between two flattened structs name both fields but not the option.

[guide-aliases]: aliases.md
[guide-env]: env.md
[guide-flatten]: flatten.md
[guide-parsing]: parsing.md
[guide-validation]: validation.md
[guide-value-enum]: value_enum.md
