# Error Handling

`parse` and `from_args` return `Error<'a>`, where `'a` is the lifetime of the parsed arguments: errors describing what the user typed
borrow the offending argument from the command line instead of copying it, so reporting them needs no allocation.
Its `Display` implementation produces Clap-style messages, for example `unexpected argument '--bogus' found`.
This is a Clap-inspired subset of Clap's error reporting: there are no error kinds, colours, usage reminders or exit codes.

```rust
use getargs_derive::{Error, GetArgs, Opt};

#[derive(Debug, GetArgs)]
struct Cli<'a> {
    input: &'a str,
    #[arg(short, long)]
    verbose: bool,
    #[arg(short, long, default_value_t = 1)]
    jobs: u32,
}

let error = Cli::from_args(["in.txt", "--verbos"].into_iter()).unwrap_err();
assert_eq!(
    error,
    Error::UnknownOption { option: Opt::Long("verbos"), suggestion: Some("verbose") }
);
assert_eq!(
    error.to_string(),
    "unexpected argument '--verbos' found\n\n  tip: a similar argument exists: '--verbose'"
);

let error = Cli::from_args(["in.txt", "-j", "many"].into_iter()).unwrap_err();
assert_eq!(error, Error::InvalidValue { name: "--jobs <JOBS>", value: "many" });
assert_eq!(error.to_string(), "invalid value 'many' for '--jobs <JOBS>'");

let error = Cli::from_args([].into_iter()).unwrap_err();
assert_eq!(error.to_string(), "the following required argument was not provided: <INPUT>");
```

| Variant                                               | Cause                                                                                                       | `Display`                                                         |
| ----------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| `MissingArgument(&'static str)`                       | A required positional argument or named option was not provided.                                           | `the following required argument was not provided: <INPUT>`       |
| `UnknownOption { option: Opt<'a>, suggestion }`       | An option the struct does not define was given (`--bogus`, `-z`). `suggestion` is the closest long name, if any. | `unexpected argument '--bogus' found`, followed by a `tip:` line when there is a suggestion |
| `UnexpectedArgument(&'a str)`                         | A positional argument was given but the struct expects no more of them.                                     | `unexpected argument 'extra' found`                               |
| `UnexpectedValue(Opt<'a>)`                            | A value was attached to a flag (`--verbose=yes`).                                                           | `unexpected value for '--verbose': it does not take a value`      |
| `MissingOptionValue(&'static str)`                    | An option was given without its value (`--output` at the end of the arguments).                            | `a value is required for '--output <OUTPUT>' but none was supplied` |
| `InvalidValue { name: &'static str, value: &'a str }` | A value or `default_value` could not be parsed into the field type (`abc` for a `u32`), or a `value_parser` function rejected it. | `invalid value 'abc' for '--port <PORT>'`                         |
| `InvalidPossibleValue { name, value, possible_values }` | A value or `default_value` is not one of the `possible_values`. See [Value Validation][guide-validation]. | `invalid value 'turbo' for '--mode <MODE>'`, followed by a `[possible values: fast, slow]` line |
| `ValueOutOfRange { name, value, range }`              | A value or `default_value` is outside the `value_parser` range. `range` is written as in the attribute.   | `invalid value '0' for '--jobs <JOBS>': 0 is not in 1..=64`        |
| `TooManyValues { name, value, max }`                  | A [repeated argument][guide-collections] received more values than its collection holds or its `num_args` accepts. `value` is the first value that did not fit. | `unexpected value 'd' for '--include <INCLUDE>' found; no more were expected` |
| `TooFewValues { name, min, actual }`                  | A positional collection received fewer values than its `num_args` minimum, but at least one.               | `2 values required by '<PATHS>...'; only 1 was provided`          |
| `ArgumentConflict { name, other }`                    | Two arguments that cannot be used together were given (`conflicts_with` or an exclusive group). See [Argument Relations][guide-relations]. | `the argument '--quiet' cannot be used with '--verbose'` |
| `MissingRequiredArgument { name, required_by }`       | An argument was given without an argument it `requires`.                                                    | `the following required argument was not provided: --output <OUTPUT> (required by '--compress')` |
| `MissingOneOf(&'static [&'static str])`               | None of a set of arguments was given (`required_unless_present` or a `required` group).                     | `the following required arguments were not provided: <--config <CONFIG>\|--stdin>` |
| `MissingSubcommand(&'static [&'static str])`          | A required subcommand was not given. Lists the visible subcommands. See [Subcommands][guide-subcommands].   | `a subcommand is required but one was not provided`, followed by a `[subcommands: add, remove]` line |
| `UnknownSubcommand { name: &'a str, suggestion }`     | A positional argument where a subcommand was expected is not a subcommand name or alias. `suggestion` is the closest visible name or alias, if any. | `unrecognized subcommand 'stats'`, followed by a `tip: a similar subcommand exists: 'status'` line when there is a suggestion |
| `InvalidArguments`                                    | `getargs` reported an error kind this crate does not recognise (a future `getargs` release). Never returned with known `getargs` 0.5 releases. | `the arguments could not be parsed` |
| `InvalidUtf8 { name: Option<&'static str>, value: &'a [u8] }` | With [raw byte arguments][guide-non-utf8], a value that is not valid UTF-8 was given to a field parsed from text. `name` is `None` for an argument that matches no field (an extra positional argument, an option or subcommand name). | `invalid UTF-8 in value 'caf\xE9' for '--name <NAME>'`, or `invalid UTF-8 in argument 'caf\xE9'` without a name |
| `Help(&'static str)`                                  | Not a failure: `-h` or `--help` was given. Carries the generated help text (`HELP` for `-h`, `LONG_HELP` for `--help`), or the help of the subcommand when given after one. See [Generated Help][guide-help]. | the help text                                                   |
| `Version(&'static str)`                               | Not a failure: `-V` or `--version` was given with `#[command(version)]`. Carries the version line (`VERSION_TEXT`). See [Version Flag][guide-version]. | the version line                     |

`Error::is_help()` and `Error::is_version()` tell these two outcomes apart from parse failures.

`Error` is `#[non_exhaustive]`: new features add new kinds of failures, such as `InvalidUtf8` for [raw byte arguments][guide-non-utf8],
and this lets them be added in a minor release instead of breaking every exhaustive `match`. A `match` on an `Error` outside this crate therefore needs a
wildcard arm, typically printing the error with its `Display` implementation. `Opt` stays exhaustive: an option is either short or long.

```rust
use getargs_derive::{Error, GetArgs};

#[derive(GetArgs)]
struct Cli<'a> {
    input: &'a str,
}

let message = match Cli::from_args([].into_iter()) {
    Ok(cli) => cli.input.to_string(),
    Err(Error::Help(help)) => help.to_string(),
    Err(Error::MissingArgument(name)) => format!("missing {name}"),
    Err(error) => error.to_string(),
};
assert_eq!(message, "missing <INPUT>");
```

The `&'static str` names are written as in the generated help: `<VALUE_NAME>` for a positional argument, and the long name (or the short name
when there is none) followed by `<VALUE_NAME>` for a named option (`--jobs <JOBS>`, `-p <PORT>`). `Opt<'a>` is the option as the user wrote it,
`Opt::Short('z')` or `Opt::Long("bogus")`, and displays with its dashes.

Suggestions are computed without allocation against the struct's long option names and [long aliases][guide-aliases], including `help` unless the help flag is disabled and `version`
when the version flag is enabled, using the edit distance with transpositions: a name is suggested when the distance is at most a third of the longer
of the two names. Short options never get suggestions. After a subcommand, unknown long options are matched against the names of that
subcommand, and unknown subcommands against the visible names and aliases of the subcommands.

Because errors borrow from the arguments, they must be handled while the arguments are alive. When the arguments are string literals the error is `Error<'static>`
and can be returned from `main`; with `std::env::args()`, collect the arguments first and report the error before they are dropped,
as shown in [Generated Help][guide-help].

[guide-aliases]: aliases.md
[guide-collections]: collections.md
[guide-help]: help.md
[guide-non-utf8]: non_utf8.md
[guide-relations]: relations.md
[guide-subcommands]: subcommands.md
[guide-validation]: validation.md
[guide-version]: version.md
