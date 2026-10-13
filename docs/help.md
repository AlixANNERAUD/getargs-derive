# Generated Help

`#[derive(GetArgs)]` also generates `HELP` and `LONG_HELP` constants holding the short and [long help](#short-and-long-help) texts, built entirely at compile time from the struct, its fields and their doc comments.
Both are `&'static str`: no allocation and no formatting happen at run time.

```rust
use getargs_derive::GetArgs;

/// Copy files around.
#[derive(GetArgs)]
#[command(name = "copy")]
struct Cli<'a> {
    /// Input file.
    input: &'a str,
    /// Output file.
    #[arg(default_value = "out.txt")]
    output: &'a str,
    /// Verbose output.
    #[arg(short, long)]
    verbose: bool,
    /// Number of jobs.
    #[arg(short, long, default_value_t = 4)]
    jobs: u32,
    #[arg(long, value_name = "KIND", help = "Copy mode")]
    mode: &'a str,
}

assert_eq!(
    Cli::HELP,
    "\
Copy files around

Usage: copy [OPTIONS] <INPUT> [OUTPUT]

Arguments:
  <INPUT>   Input file
  [OUTPUT]  Output file [default: out.txt]

Options:
  -v, --verbose      Verbose output
  -j, --jobs <JOBS>  Number of jobs [default: 4]
      --mode <KIND>  Copy mode
  -h, --help         Print help"
);
```

The text follows Clap's layout:

- The **about** line is the first paragraph of the struct's doc comment, or `#[command(about = "...")]`.
- The **usage** line shows `[OPTIONS]` when there is at least one named option (the help flag counts), then each positional argument in order: `<NAME>` when required, `[NAME]` when it has a default or is an `Option<T>`.
- Each **argument description** is the first paragraph of the field's doc comment, or `#[arg(help = "...")]`. As in Clap, the lines of the paragraph are joined with spaces and a single trailing period is removed; an explicit `help` is used as written.
- Defaults are shown as `[default: value]`, except on flags. A `default_value` or string `default_value_t` is shown as its string (quoted if it contains whitespace), any other `default_value_t` as its Rust expression.
- Empty sections are omitted. The text has no trailing newline.
- The layout can be customized as described in [Customizing the Help](#customizing-the-help).

## Short and Long Help

As in Clap, `-h` shows the short help (`HELP`) and `--help` shows the long help (`LONG_HELP`). The long help uses:

- the struct's full doc comment instead of its first paragraph, or `#[command(long_about = "...")]`;
- each field's full doc comment instead of its first paragraph, or `#[arg(long_help = "...")]`.

A full doc comment is only used when it has more than one paragraph. Its paragraphs are separated by a blank line, the lines of each paragraph are joined with spaces, and
trailing periods are kept. When there is no long text at all, `LONG_HELP` is the same string as `HELP` and `--help` behaves like `-h`.

Otherwise, the short help's `-h, --help` line reads `Print help (see more with '--help')`, and the long help uses Clap's long layout: each description starts on the line after its argument,
indented by 10 spaces, entries are separated by blank lines, and defaults are shown as a separate paragraph:

```rust
use getargs_derive::GetArgs;

/// Copy files around.
///
/// Files are copied one by one.
#[derive(GetArgs)]
#[command(name = "copy")]
struct Cli {
    /// Number of jobs.
    #[arg(short, long, default_value_t = 4, long_help = "Number of parallel jobs.\n\nUse 1 to copy sequentially.")]
    jobs: u32,
}

assert_eq!(
    Cli::HELP,
    "\
Copy files around

Usage: copy [OPTIONS]

Options:
  -j, --jobs <JOBS>  Number of jobs [default: 4]
  -h, --help         Print help (see more with '--help')"
);
assert_eq!(
    Cli::LONG_HELP,
    "\
Copy files around.

Files are copied one by one.

Usage: copy [OPTIONS]

Options:
  -j, --jobs <JOBS>
          Number of parallel jobs.

          Use 1 to copy sequentially.

          [default: 4]

  -h, --help
          Print help (see a summary with '-h')"
);
```

## Customizing the Help

These attributes change the generated help, in both its short and long forms. They never change how arguments are parsed.

| Attribute                          | Description                                                                                  |
| ---------------------------------- | -------------------------------------------------------------------------------------------- |
| `#[arg(hide)]`                     | Omits the argument from the help, including the usage line. It is still parsed.              |
| `#[arg(help_heading = "Name")]`    | Lists the argument in a `Name:` section instead of `Arguments:` or `Options:`.               |
| `#[arg(display_order = N)]`        | Position of a named option within its section. Not allowed on positional arguments.          |
| `#[arg(long_help = "text")]`       | Description shown in the long help. Overrides the field's full doc comment.                  |
| `#[command(long_about = "text")]`  | Description shown at the top of the long help. Overrides the struct's full doc comment.      |
| `#[command(before_help = "text")]` | Text shown before everything else, followed by a blank line.                                 |
| `#[command(after_help = "text")]`  | Text shown after everything else, preceded by a blank line.                                  |
| `#[command(max_term_width = N)]`   | Wraps the help at `N` columns. Without it, the help is not wrapped.                          |

Sections are shown in this order: `Arguments`, `Options`, then the custom headings in the order they first appear among the fields.
A `help_heading` named `Arguments` or `Options` uses that standard section. Sections with only hidden arguments are omitted, and so is `[OPTIONS]` when every named option is hidden and the help flag is disabled.

Within a section, entries are sorted by `display_order`. A field without one is ordered as if its `display_order` were its position among the struct's fields, counting from 0,
and ties keep the declaration order. The `-h, --help` and `-V, --version` lines always come last in `Options`.

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(name = "fetch", before_help = "fetch 1.0", after_help = "See fetch(1) for details.")]
struct Cli<'a> {
    /// Address to fetch.
    url: &'a str,
    /// Output file.
    #[arg(short, long)]
    output: Option<&'a str>,
    /// Verbose output.
    #[arg(short, long, display_order = 0)]
    verbose: bool,
    /// Proxy to use.
    #[arg(long, help_heading = "Network")]
    proxy: Option<&'a str>,
    /// Internal tracing.
    #[arg(long, hide)]
    trace: bool,
}

assert_eq!(
    Cli::HELP,
    "\
fetch 1.0

Usage: fetch [OPTIONS] <URL>

Arguments:
  <URL>  Address to fetch

Options:
  -v, --verbose          Verbose output
  -o, --output <OUTPUT>  Output file
  -h, --help             Print help

Network:
      --proxy <PROXY>  Proxy to use

See fetch(1) for details."
);
```

### Wrapping

With `#[command(max_term_width = N)]`, the help is wrapped at `N` columns at compile time, since the terminal width is not known when the text is built.
Text is wrapped between words, and a word longer than the available width is not split. The about, `before_help` and `after_help` texts are wrapped at `N` columns,
and descriptions wrap with their continuation lines aligned under the first one. As in Clap, when the labels of a section take more than 40% of the width and a description does not fit next to its label,
that description starts on the next line, indented by 10 spaces. The usage line is not wrapped.
The description of a [`value_enum`][guide-value-enum] field is not wrapped, because its default and `[possible values: ...]` come from the
`ValueEnum` implementation and are only known when the help constant is evaluated: it stays on the label's line in the short help, and starts on the next line in the long help.

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(name = "wrap", max_term_width = 40)]
struct Cli<'a> {
    /// Name of the thing to process, described at length.
    #[arg(short, long)]
    name: &'a str,
}

assert_eq!(
    Cli::HELP,
    "\
Usage: wrap [OPTIONS]

Options:
  -n, --name <NAME>
          Name of the thing to process,
          described at length
  -h, --help         Print help"
);
```

## Handling `-h` and `--help`

Unless disabled, `-h` and `--help` make `parse` and `from_args` return `Err(Error::Help(text))`, where `text` is the struct's `HELP` for `-h` and its `LONG_HELP` for `--help`.
This takes priority over missing required arguments, so `prog --help` works even when the required arguments are absent.
It is a distinct outcome rather than a parse failure: check it with `Error::is_help()`. The generated code never prints anything or exits the process,
so it stays `no_std`; the application decides how to display the help. With `std`, print it to standard output and exit successfully, as Clap does:

```rust,no_run
use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(name = "copy")]
struct Cli<'a> {
    /// Input file.
    input: &'a str,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = match Cli::from_args(args.iter().map(String::as_str)) {
        Ok(cli) => cli,
        Err(error) if error.is_help() => {
            println!("{error}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("error: {error}\n\n{}", Cli::HELP);
            std::process::exit(2);
        }
    };
    println!("copying {}", cli.input);
}
```

`-h` and `--help` are reserved: a field declaring short name `h` (including one inferred from a field name starting with `h`) or long name `help` is a compile error.

## Disabling the Help Flag

With `#[command(disable_help_flag)]`, `-h` and `--help` become ordinary unknown options, and fields may use them.
`HELP` and `LONG_HELP` are still generated (without the `-h, --help` line) so you can implement your own help option:

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(name = "connect", disable_help_flag)]
struct Cli<'a> {
    /// Host to connect to.
    #[arg(short, long)]
    host: &'a str,
}

# fn main() -> Result<(), getargs_derive::Error<'static>> {
let cli = Cli::from_args(["-h", "example.com"].into_iter())?;
assert_eq!(cli.host, "example.com");
assert_eq!(
    Cli::HELP,
    "Usage: connect [OPTIONS]\n\nOptions:\n  -h, --host <HOST>  Host to connect to"
);
# Ok(())
# }
```

The version flag is described in [Version Flag][guide-version].

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute or field, for:

- a field using `-h` or `--help` while the generated help flag is enabled;
- `hide` given a value;
- `help_heading` that is not a string literal, or is empty;
- `display_order` that is not a non-negative integer literal, or is used on a positional argument;
- `long_help`, `long_about`, `before_help` or `after_help` that is not a string literal;
- `max_term_width` that is not a positive integer literal;
- any of these keys given more than once.

## Differences From Clap

This remains a Clap-inspired subset of Clap's help customization. `before_long_help`, `after_long_help`, `next_line_help`, `term_width`, `hide_short_help`, `hide_long_help`, help templates
and styling are not supported. The help is only wrapped when `max_term_width` is set, at exactly that width, whereas Clap wraps at the terminal width (up to 100 columns by default).
Clap sorts entries with equal `display_order` by name, while this crate keeps their declaration order.

[guide-value-enum]: value_enum.md
[guide-version]: version.md
