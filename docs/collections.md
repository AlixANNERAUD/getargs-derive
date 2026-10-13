# Repeated Values

As in Clap, a field whose type is a collection receives every value given to its argument: a named option given several times (`-I a -I b`)
appends one value per occurrence, like Clap's `ArgAction::Append`, and a positional argument collects all the remaining positional arguments.
Each value is converted and [validated][guide-validation] like the value of a single-valued field, then appended.
This is a Clap-inspired subset, implemented without allocation unless you choose an allocating collection.

```rust
use getargs_derive::{Error, GetArgs, Values};

#[derive(Debug, GetArgs)]
#[command(name = "cc")]
struct Compile<'a> {
    /// Directories searched for headers.
    #[arg(short = 'I', long)]
    include: Values<&'a str, 8>,
    /// Comma-separated features.
    #[arg(short, long, value_delimiter = ',')]
    features: Values<&'a str, 4>,
    /// Source files.
    #[arg(num_args = 1..)]
    sources: Values<&'a str, 16>,
}

# fn main() -> Result<(), Error<'static>> {
let compile = Compile::from_args(
    ["main.c", "-I", "include", "-Ivendor", "--features=simd,lto", "util.c"].into_iter(),
)?;
assert_eq!(compile.include, ["include", "vendor"]);
assert_eq!(compile.features, ["simd", "lto"]);
assert_eq!(compile.sources, ["main.c", "util.c"]);

let error = Compile::from_args(["-f", "a,b,c,d,e", "main.c"].into_iter()).unwrap_err();
assert_eq!(error, Error::TooManyValues { name: "--features <FEATURES>", value: "e", max: 4 });

let error = Compile::from_args(["-I", "include"].into_iter()).unwrap_err();
assert_eq!(error, Error::MissingArgument("<SOURCES>..."));

assert!(Compile::HELP.starts_with("Usage: cc [OPTIONS] <SOURCES>...\n"));
# Ok(())
# }
```

## Storage

The collection type is chosen by the field type, recognised by the last segment of its path (`Values`, `Vec`), and determines where the values are stored:

| Field type                        | Feature    | Allocates | Capacity                  |
| --------------------------------- | ---------- | --------- | ------------------------- |
| `getargs_derive::Values<T, N>`    | none       | no        | `N` values, inline        |
| `heapless::Vec<T, N>`             | `heapless` | no        | `N` values, inline        |
| `Vec<T>` (`alloc::vec::Vec`)      | `alloc`    | yes       | unbounded                 |

- `Values<T, N>` is provided by this crate and always available, so the default build stays `no_std` without allocation. It is a minimal inline vector:
  it dereferences to `[T]`, and offers `new`, `push`, `capacity`, `as_slice`, `as_mut_slice`, equality with slices and arrays, `Debug`, `Clone`, `Hash` and iteration by reference.
- `heapless::Vec<T, N>` stores the values inline as well, and is the better choice when the rest of the program already uses `heapless`.
  The `heapless` feature adds a dependency on `heapless` 0.9.
- `Vec<T>` has no fixed limit, at the cost of requiring an allocator. The `alloc` feature only uses the `alloc` crate, so it works in `no_std` programs with a global allocator; the `std` feature enables it.

A bounded collection reserves room for `N` values in the struct itself, whatever the number of values given: choose `N` from the number of values the program can handle,
and expect [`Error::TooManyValues`][guide-errors] beyond it. An unbounded `Vec<T>` never reports it, unless `num_args` sets a maximum.

Fixed-size arrays `[T; N]` are not supported, because they would need exactly `N` values; use `Values<T, N>` with `num_args = N` instead.
`Option<Values<T, N>>` and other optional collections are not supported either: a collection is simply empty when its argument is absent.
Elements cannot be collections, arrays or `Option<T>`.

## Repeated Options

A named option of a collection type takes one value per occurrence, in every form accepted for single values (`-I a`, `-Ia`, `--include a`, `--include=a`),
and the occurrences may be interleaved with other arguments. The field is empty when the option is absent.
There is no `action = Append`: the collection type implies it, and Clap's `num_args` on options (several values per occurrence, as in `--include a b`) is not supported.

## Value Delimiter

`value_delimiter = ','` splits every value at the given character, so `--features simd,lto` appends `simd` and `lto`. The pieces are borrowed from the
argument with `str::split`, so splitting does not allocate. An empty piece is kept: `a,,b` gives three values, the second one empty.
The delimiter applies to option values, positional values, [environment variables][guide-env] and `default_value`.

## Positional Collections

A positional argument of a collection type collects every remaining positional argument, so no positional argument can follow it, except a [`last`][guide-parsing] positional.
`num_args` bounds the number of values it collects, as a Clap-inspired subset of Clap's `num_args` for positionals:

| Attribute               | Meaning                                                  |
| ----------------------- | -------------------------------------------------------- |
| none                    | Any number of values, none included (`[NAME]...`).       |
| `num_args = 1..`        | At least one value (`<NAME>...`).                        |
| `num_args = 2..=4`      | Between 2 and 4 values. `2..5` is the same range.        |
| `num_args = 3`          | Exactly 3 values.                                        |

When fewer values than the minimum are given, parsing fails with `Error::MissingArgument` if there are none, and with [`Error::TooFewValues`][guide-errors] otherwise;
more values than the maximum fail with `Error::TooManyValues`. The bounds must be integer literals.

With `last = true`, the positional collects every argument after `--`, whatever it looks like, which is how a wrapper passes arguments through to another program:

```rust
use getargs_derive::{GetArgs, Values};

#[derive(GetArgs)]
#[command(name = "run")]
struct Run<'a> {
    #[arg(short, long)]
    verbose: bool,
    program: &'a str,
    #[arg(last = true)]
    arguments: Values<&'a str, 8>,
}

# fn main() -> Result<(), getargs_derive::Error<'static>> {
let run = Run::from_args(["cargo", "-v", "--", "build", "--release"].into_iter())?;
assert_eq!(run.program, "cargo");
assert!(run.verbose);
assert_eq!(run.arguments, ["build", "--release"]);
assert!(Run::HELP.starts_with("Usage: run [OPTIONS] <PROGRAM> [-- <ARGUMENTS>...]\n"));
# Ok(())
# }
```

With `allow_negative_numbers`, a positional collection also collects negative numbers such as `-5`, with the [limits][guide-parsing] that apply to positionals.
Clap's `trailing_var_arg`, which makes a positional also collect arguments that look like options once it has started, is not supported, because `getargs`
has already split them into options; `last = true` and `--` cover the same need.

## Defaults and Environment Variables

A collection that received no value on the command line is filled from its [environment variable][guide-env], then from its default:

- `default_value = "a,b"` is one value, split with the `value_delimiter` when there is one;
- `default_values = ["a", "b"]` gives several values, as in Clap, and is shown as `[default: a, b]` in the help.

An environment variable gives one value, split with the `value_delimiter` when there is one: with `env = "PATHS"` and `value_delimiter = ':'`, `PATHS=/bin:/usr/bin`
gives two values. Values given on the command line replace the environment and the defaults rather than adding to them.
`default_value_t` is not supported on collections.

```rust
use getargs_derive::{GetArgs, Values};

#[derive(GetArgs)]
struct Search<'a> {
    #[arg(long, env = "SEARCH_PATHS", value_delimiter = ':', default_value = "/usr/bin")]
    path: Values<&'a str, 8>,
    #[arg(long, default_values = ["rs", "toml"])]
    extension: Values<&'a str, 4>,
}

# fn main() -> Result<(), getargs_derive::Error<'static>> {
let search = Search::from_args_with_env([].into_iter(), |_| Some("/bin:/sbin"))?;
assert_eq!(search.path, ["/bin", "/sbin"]);
assert_eq!(search.extension, ["rs", "toml"]);

let search = Search::from_args_with_env(["--extension", "md"].into_iter(), |_| None)?;
assert_eq!(search.path, ["/usr/bin"]);
assert_eq!(search.extension, ["md"]);
# Ok(())
# }
```

## Composition

- [Validation][guide-validation]: `value_parser` and `possible_values` check every value, and the error names the value that failed.
- [Value enums][guide-value-enum]: `#[arg(value_enum)]` on a `Values<Mode, N>` field parses every value as a `Mode`, and lists the possible values in the help.
- [Relations][guide-relations]: a collection is present when it received at least one value from the command line.
- [Flattened structs][guide-flatten]: collection fields of a flattened struct behave as in the struct itself. A positional collection collects every remaining positional
  argument, so the positional arguments of a struct flattened after it are never filled.

## Generated Help

A positional collection is shown as `<NAME>...` when it requires values and `[NAME]...` otherwise, in the usage line and in the `Arguments` section,
and as `[-- <NAME>...]` with `last = true`. A repeated option keeps its usual label, `-I, --include <INCLUDE>`, as in Clap, since it takes one value per occurrence.
Errors name positional collections the same way, e.g. `<SOURCES>...`.

The [command metadata][guide-completions] marks collection arguments as `multiple`, so the generated completion scripts offer a repeated option again
and keep completing a positional collection, and the man page shows `<NAME>...`.

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute or field, for:

- fixed-size array fields, `Option<T>` wrapping a collection, and collections of collections, arrays or `Option<T>`;
- `value_delimiter`, `num_args` or `default_values` on a field that is not a collection;
- `value_delimiter` without a character literal, `default_values` without a non-empty array of string literals, and any of them given more than once;
- `num_args` that is not an integer literal or a range of integer literals, is empty, or accepts no value;
- `num_args` on a named option;
- `default_value_t` on a collection, and `default_values` combined with `default_value`;
- a `default_value` piece or a `default_values` entry that is not one of the `possible_values`;
- a positional argument following a positional collection, unless it is `last`.

A field whose type is named `Vec` or `Values` but is not a supported collection, such as `Vec<T>` without the `alloc` feature, is reported by the compiler as
`` `Vec<&str>` cannot collect repeated argument values ``, with a note listing the supported types and their features.

[guide-completions]: completions.md
[guide-env]: env.md
[guide-errors]: errors.md
[guide-flatten]: flatten.md
[guide-parsing]: parsing.md
[guide-relations]: relations.md
[guide-validation]: validation.md
[guide-value-enum]: value_enum.md
