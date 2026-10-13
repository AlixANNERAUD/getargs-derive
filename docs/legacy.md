# Legacy Syntax

Earlier versions of this crate used a different attribute syntax. These forms are still accepted so existing code keeps compiling, but they are deprecated in favour of the attributes described in [Attribute Reference][guide-attributes]:

| Legacy form                   | Replacement                                    | Notes                                                               |
| ----------------------------- | ---------------------------------------------- | ------------------------------------------------------------------- |
| `#[arg(default = expr)]`      | `#[arg(default_value_t = expr)]`               | Identical behaviour.                                                |
| `#[default(expr)]`            | `#[arg(default_value_t = expr)]`               | Identical behaviour.                                                |
| `#[default = expr]`           | `#[arg(default_value_t = expr)]`               | Identical behaviour.                                                |
| `#[arg(flag)]`                | nothing, `bool` named options are flags        | Only valid on `bool` fields that also have `short` and/or `long`.   |
| `#[arg(positional)]`          | nothing, fields without `short`/`long` are positional | Cannot be combined with `short` or `long`.                   |

## Migrating From the Legacy Syntax

Previously, every named option automatically received **both** a short name (first character of the field) and a long name (kebab-cased field name),
and `flag`, `default` or `#[default]` alone were enough to make a field a named option. This is no longer the case:
a field is named only when it declares `short` and/or `long`, and only the declared names are accepted. Migrate as follows:

| Before                                         | After                                               |
| ---------------------------------------------- | --------------------------------------------------- |
| `#[arg(flag)] verbose: bool`                   | `#[arg(short, long)] verbose: bool`                 |
| `#[arg(default = 4)] jobs: u32`                | `#[arg(short, long, default_value_t = 4)] jobs: u32` |
| `#[default(4)] jobs: u32`                      | `#[arg(short, long, default_value_t = 4)] jobs: u32` |
| `#[arg(short = 'o', default = "out")] output`  | `#[arg(short = 'o', long, default_value = "out")] output` |
| `#[arg(long = "mode", default = "fast")] mode` | `#[arg(short, long = "mode", default_value = "fast")] mode` |
| `#[arg(positional)] input: &'a str`            | `input: &'a str`                                    |

Keep only `short` or only `long` if you do not want both names. Note the behaviour changes when migrating code that relied on the old inference:

- `#[arg(flag)]` without `short`/`long` is now a compile error asking you to add them.
- A legacy default (`#[arg(default = ...)]`, `#[default(...)]`) without `short`/`long` no longer creates a named option: the field becomes an **optional positional argument**.
  Add `short` and/or `long` to keep it a named option.

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute or field, for:

- legacy `flag` on a non-`bool` field or without `short`/`long`, and legacy `positional` combined with `short`/`long`.

[guide-attributes]: attributes.md
