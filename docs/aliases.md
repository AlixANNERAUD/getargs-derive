# Argument Aliases

Like Clap, a named option can be given extra names. An alias matches exactly like the option's own `short` or `long` name,
including `--alias=value`, grouped short flags (`-xv`) and repeated `Count` flags.

| Attribute                                  | Description                                                    |
| ------------------------------------------ | -------------------------------------------------------------- |
| `#[arg(alias = "name")]`                   | Hidden long alias (`--name`).                                  |
| `#[arg(aliases = ["a", "b"])]`             | Several hidden long aliases.                                   |
| `#[arg(visible_alias = "name")]`           | Long alias listed in the generated help.                       |
| `#[arg(visible_aliases = ["a", "b"])]`     | Several long aliases listed in the generated help.             |
| `#[arg(short_alias = 'x')]`                | Hidden short alias (`-x`).                                     |
| `#[arg(short_aliases = ['x', 'y'])]`       | Several hidden short aliases.                                  |
| `#[arg(visible_short_alias = 'x')]`        | Short alias listed in the generated help.                      |
| `#[arg(visible_short_aliases = ['x', 'y'])]` | Several short aliases listed in the generated help.          |

Each key can be repeated and combined with the others. Aliases follow the same rules as `short` and `long` names: a long alias is written without
the leading `--`, and a short alias is a single character.

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(name = "copy")]
struct Cli<'a> {
    /// Output file.
    #[arg(short, long, alias = "out", visible_alias = "destination", visible_short_alias = 'd')]
    output: &'a str,
    /// Verbose output.
    #[arg(long, short_aliases = ['v', 'V'], visible_aliases = ["loud", "chatty"])]
    verbose: bool,
}

# fn main() -> Result<(), getargs_derive::Error<'static>> {
let cli = Cli::from_args(["--out", "a.txt", "-v"].into_iter())?;
assert_eq!(cli.output, "a.txt");
assert!(cli.verbose);

let cli = Cli::from_args(["-d", "b.txt", "--chatty"].into_iter())?;
assert_eq!(cli.output, "b.txt");
assert!(cli.verbose);

assert_eq!(
    Cli::HELP,
    "\
Usage: copy [OPTIONS]

Options:
  -o, --output <OUTPUT>  Output file [aliases: --destination] [short aliases: -d]
      --verbose          Verbose output [aliases: --loud, --chatty]
  -h, --help             Print help"
);
# Ok(())
# }
```

- **Help**: as in Clap, visible long aliases are listed as `[aliases: --a, --b]` and visible short aliases as `[short aliases: -x, -y]`,
  after the `[default: ...]` value. Hidden aliases are never shown.
- **Errors**: errors always name the option by its primary name (`--output <OUTPUT>`), whichever alias was used.
- **Suggestions**: long aliases, hidden or visible, are candidates for the "did you mean" suggestion of an unknown long option, as in Clap. See [Error Handling][guide-errors].
- **Reserved names**: `-h`/`--help` (unless `#[command(disable_help_flag)]`) and `-V`/`--version` (when `#[command(version)]` is set) cannot be used as aliases.

This remains a Clap-inspired subset: aliases are only supported on named options, and subcommand aliases do not exist since subcommands are not supported.

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute or field, for:

- an alias on a positional argument (a field without `short` or `long`);
- an alias that duplicates any other short or long name, primary or alias, of the same or another field;
- an alias using `-h`/`--help` while the generated help flag is enabled, or `-V`/`--version` while the version flag is enabled;
- a long alias that is empty, starts with `-`, or contains whitespace or `=`, and a short alias that is `-` or whitespace;
- an alias key without a value, a non-string long alias, a non-character short alias, and a plural key whose value is not an array.

[guide-errors]: errors.md
