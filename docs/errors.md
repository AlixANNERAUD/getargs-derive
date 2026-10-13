# Error Handling

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
        Error::Help(help) => {
            println!("{help}");
        }
        Error::Version(version) => {
            println!("{version}");
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
| `Help(&'static str)`                      | Not a failure: `-h` or `--help` was given. Carries the generated help text, which is also what `Display` prints. See [Generated Help][guide-help]. |
| `Version(&'static str)`                   | Not a failure: `-V` or `--version` was given with `#[command(version)]`. Carries the version line (`VERSION_TEXT`), which is also what `Display` prints. See [Version Flag][guide-version]. |

The name carried by an error is the long name for named options that have one, and the field name otherwise.

[guide-help]: help.md
[guide-version]: version.md
