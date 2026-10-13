# Generated Help

`#[derive(GetArgs)]` also generates a `HELP` constant holding the help text, built entirely at compile time from the struct, its fields and their doc comments:

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

## Handling `-h` and `--help`

Unless disabled, `-h` and `--help` make `parse` and `from_args` return `Err(Error::Help(text))`, where `text` is the struct's `HELP`.
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
`HELP` is still generated (without the `-h, --help` line) so you can implement your own help option:

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(name = "connect", disable_help_flag)]
struct Cli<'a> {
    /// Host to connect to.
    #[arg(short, long)]
    host: &'a str,
}

# fn main() -> Result<(), getargs_derive::Error> {
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

- a field using `-h` or `--help` while the generated help flag is enabled.

[guide-version]: version.md
