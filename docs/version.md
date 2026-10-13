# Version Flag

`#[command(version)]` enables a version flag, as in Clap. The version is the package version (`CARGO_PKG_VERSION`) of the crate deriving `GetArgs`,
and `#[command(version = "1.2.3")]` sets it explicitly. Two constants are then generated: `VERSION`, the version itself, and `VERSION_TEXT`,
the line Clap prints for `--version` (the command name, a space and the version). The help gains a `-V, --version  Print version` line.

## Handling `-V` and `--version`

`-V` and `--version` make `parse` and `from_args` return `Err(Error::Version(text))`, where `text` is the struct's `VERSION_TEXT`.
Like help, this takes priority over missing required arguments, is checked with `Error::is_version()`, and never prints anything or exits the process:

```rust
use getargs_derive::{Error, GetArgs};

#[derive(Debug, GetArgs)]
#[command(name = "copy", version = "1.2.3")]
struct Cli<'a> {
    /// Input file.
    input: &'a str,
}

assert_eq!(Cli::VERSION, "1.2.3");
let error = Cli::from_args(["--version"].into_iter()).unwrap_err();
assert!(error.is_version());
assert_eq!(error, Error::Version("copy 1.2.3"));
assert_eq!(
    Cli::HELP,
    "\
Usage: copy [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input file

Options:
  -h, --help     Print help
  -V, --version  Print version"
);
```

With `std`, print it to standard output and exit successfully, as for help (see [Generated Help][guide-help]). `-V` and `--version` are only reserved when `version` is set:
a field declaring short name `V` or long name `version` is then a compile error. Without `version`, no version flag is generated and fields may use those names.

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute or field, for:

- a field using `-V` or `--version` while `#[command(version)]` is set;
- an empty or non-string `#[command(version = ...)]`.

[guide-help]: help.md
