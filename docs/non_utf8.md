# Non-UTF-8 Arguments

Command-line arguments are not always valid UTF-8: on Unix, file names are arbitrary bytes.
Besides `parse` and `from_args`, which take `&str` arguments and are unchanged, every struct deriving `GetArgs` gets methods taking raw `&[u8]` arguments,
which `getargs` parses the same way:

| `&str` arguments     | `&[u8]` arguments     |
| -------------------- | --------------------- |
| `parse`              | `parse_bytes`         |
| `from_args`          | `from_bytes`          |
| `parse_with_env`     | `parse_bytes_with_env` |
| `from_args_with_env` | `from_bytes_with_env` |

The `_with_env` methods exist under the same conditions as their `&str` counterparts; see [Environment Variables][guide-env].
Environment variable values and defaults stay strings.

```rust
use getargs_derive::{Error, GetArgs};

#[derive(Debug, GetArgs)]
struct Cli<'a> {
    /// Borrowed as raw bytes, never checked.
    input: &'a [u8],
    /// Parsed from text, so the value must be valid UTF-8.
    #[arg(short, long)]
    name: Option<&'a str>,
    #[arg(short, long, default_value_t = 1)]
    jobs: u32,
}

let args: [&[u8]; 4] = [b"caf\xE9.txt", b"--name", b"alice", b"-j4"];
let cli = Cli::from_bytes(args.into_iter()).unwrap();
assert_eq!(cli.input, b"caf\xE9.txt");
assert_eq!(cli.name, Some("alice"));
assert_eq!(cli.jobs, 4);

let args: [&[u8]; 3] = [b"in", b"--name", b"caf\xE9"];
let error = Cli::from_bytes(args.into_iter()).unwrap_err();
assert_eq!(error, Error::InvalidUtf8 { name: Some("--name <NAME>"), value: b"caf\xE9" });
assert_eq!(error.to_string(), "invalid UTF-8 in value 'caf\\xE9' for '--name <NAME>'");

let cli = Cli::from_args(["in"].into_iter()).unwrap();
assert_eq!(cli.input, b"in");
```

## Field Types

- `&[u8]` fields borrow the raw bytes of the argument, whatever they are. They can be optional, collected with [repeated values][guide-collections]
  (`value_delimiter` splits the bytes at the delimiter's UTF-8 encoding) and given `default_value`.
  With `possible_values`, the value must be valid UTF-8 to be compared.
- With the `std` feature, `&OsStr`, `&Path`, `OsString` and `PathBuf` fields convert the argument without checking it on Unix, through
  `std::os::unix::ffi::OsStrExt`. On other platforms an `OsStr` cannot be built from arbitrary bytes, so these fields require valid UTF-8 there.
  Without the `std` feature, `OsString` and `PathBuf` fields are parsed with `FromStr` as before, and `&OsStr` and `&Path` fields are rejected.
- Every other field, `&str` and `FromStr` types, [value enums][guide-value-enum] and `value_parser` functions included, is parsed from text:
  a value that is not valid UTF-8 is reported as `Error::InvalidUtf8` naming the argument, as in the help (`--name <NAME>`, `<INPUT>`).

`&str` arguments always work for every field type, so the same struct supports both entry points.

## Errors

`Error::InvalidUtf8 { name, value }` carries the raw bytes. `name` is the argument name, or `None` for an argument that matches no field:
an extra positional argument, an option name or a subcommand name that is not valid UTF-8 can only be reported this way,
since `Error::UnexpectedArgument`, `Error::UnknownOption` and `Error::UnknownSubcommand` hold text. For the same reason, a value that does not fit in a
`&[u8]` collection is reported as `Error::InvalidUtf8` instead of `Error::TooManyValues` when it is not valid UTF-8.

Invalid UTF-8 cannot be written as a `&str`, and escaping it into a `String` would allocate.
`Display` writes it with [`Lossy`](crate::Lossy), a small adapter that writes valid UTF-8 as it is and every invalid byte as a `\xNN` escape,
without allocating: `invalid UTF-8 in value 'caf\xE9' for '--name <NAME>'`, or `invalid UTF-8 in argument 'caf\xE9'` without a name.

With `&[u8]` arguments, `getargs` reads short options one byte at a time, so only ASCII short options are recognised;
another byte in a short option cluster is reported as `Error::UnknownOption` with `Opt::Short('\u{FFFD}')`.
Long option names must be valid UTF-8 to match, as the declared names are strings.

## Reading the Process Arguments

With the `std` feature, `from_args_os()` parses `std::env::args_os()` without the program name, reading `env` fallbacks from the process environment like `from_args`:

```rust,no_run
# #[cfg(feature = "std")]
# fn main() {
use getargs_derive::GetArgs;
use std::path::PathBuf;

#[derive(GetArgs)]
struct Cli {
    /// Any file name, even one that is not valid UTF-8 on Unix.
    path: PathBuf,
}

match Cli::from_args_os() {
    Ok(cli) => println!("{}", cli.path.display()),
    Err(error) if error.is_help() => println!("{error}"),
    Err(error) => {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}
# }
# #[cfg(not(feature = "std"))]
# fn main() {}
```

`args_os` yields owned `OsString` values, while parsed structs and errors borrow their arguments.
`from_args_os` therefore reads the arguments once, leaks them (`Box::leak`) and returns an `Error<'static>`, so that the struct can borrow them for `'static`
and the error can be returned from `main`. Later calls reuse the same leaked arguments instead of leaking them again.
This costs one copy of the command line for the whole program, as `env` fallbacks already do.
To avoid the leak, collect the arguments yourself and borrow them:

```rust
# #[cfg(unix)]
# fn main() {
use getargs_derive::GetArgs;
use std::os::unix::ffi::OsStrExt;

#[derive(GetArgs)]
struct Cli<'a> {
    path: &'a [u8],
}

let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
let result = Cli::from_bytes(args.iter().map(|arg| arg.as_bytes()));
# let _ = result;
# }
# #[cfg(not(unix))]
# fn main() {}
```

On Unix, `from_args_os` parses the arguments as bytes, like `from_bytes`. On other platforms it converts them with `OsStr::to_str`
and returns `Error::InvalidUtf8` with `name: None` for the first argument that is not valid Unicode, before parsing anything.

The generated help, the `Command` metadata and the [completion scripts][guide-completions] do not depend on the argument type.

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending field, for:

- `&OsStr` and `&Path` fields without the `std` feature.

Passing `&[u8]` arguments to `parse` or `from_args`, or `&str` arguments to `parse_bytes` or `from_bytes`, is a type error.
Owned byte buffers such as `Box<[u8]>` are not supported as values, as they do not implement `FromStr`: use `&[u8]`
(a `Vec<u8>` field is a [collection][guide-collections] of repeated `u8` values).

This remains a Clap-inspired subset: Clap's `OsString` value parsers, `value_parser!(PathBuf)` and `ArgMatches::get_raw` have no equivalent beyond the field types above.

[guide-collections]: collections.md
[guide-completions]: completions.md
[guide-env]: env.md
[guide-value-enum]: value_enum.md
