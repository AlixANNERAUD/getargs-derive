# Clap Compatibility

The attributes follow the conventions of [Clap's derive API](https://docs.rs/clap/latest/clap/_derive/index.html):
`#[arg(short, long)]`, `short = 'x'`, `long = "name"`, `default_value`, `default_value_t`, `env`, `help`, `long_help`, `value_name`, `value_enum`, `hide`, `help_heading`, `display_order`, the [alias keys][guide-aliases] (`alias`, `aliases`, `short_alias`, `short_aliases` and their `visible_` forms) and the `SetTrue`, `SetFalse` and `Count` actions mean the same thing here as they do in Clap,
and so do the `name`, `about`, `long_about`, `before_help`, `after_help`, `max_term_width`, `version` and `disable_help_flag` keys of `#[command(...)]`. Doc comments become help text the way they do in Clap, the generated help uses Clap's layout,
`-h` and `--help` show the short and long help as in Clap, and [error messages][guide-errors] use Clap's wording.
`Option<T>` fields are optional arguments that are `None` when absent, as in Clap.
The `conflicts_with`, `requires`, `required_unless_present` and `group` relations behave as in Clap, with groups declared through a simpler `#[command(group(id = "...", required, multiple))]` syntax; see [Argument Relations][guide-relations].
`#[derive(ValueEnum)]` with `#[value(name = "...")]` and `#[value(alias = "...")]` maps unit variants to accepted values as Clap's does.
Values can be validated with `value_parser` (a parser function or an integer range) and `possible_values`, with Clap's error messages and `[possible values: ...]` help; see [Value Validation][guide-validation].
As in Clap, an option refuses a separate value starting with `-` unless it has `allow_hyphen_values` or, for negative numbers, `allow_negative_numbers` (also accepted on `#[command(...)]`), and a `last = true` positional is only filled after `--`.
`getargs` limits these modes: `allow_hyphen_values` is not supported on positionals, and positionals only accept negative numbers through `from_args`; see [Parsing Modes][guide-parsing].
Collection fields receive repeated values as in Clap: a repeated option appends one value per occurrence (`ArgAction::Append`), `value_delimiter` splits values,
`default_values` gives several defaults, and a positional collection takes the remaining positional arguments, bounded by `num_args`; see [Repeated Values][guide-collections].
They are stored in `getargs_derive::Values<T, N>`, `heapless::Vec<T, N>` (feature `heapless`) or `Vec<T>` (feature `alloc`).
`#[command(flatten)]` merges the arguments of another `GetArgs` struct into its parent as in Clap, with positional arguments filled in field order and duplicate names rejected at compile time; see [Flattened Structs][guide-flatten].
`#[derive(Subcommand)]` and `#[command(subcommand)]` (optionally on an `Option<T>` field) work as in Clap: unit, named-field and newtype variants become kebab-cased subcommands, renamed with `name` and given other names with `alias`, `aliases`, `visible_alias` and `visible_aliases`, hidden with `hide`, nested to any depth, listed in a `Commands:` section, with their own help for `<cmd> --help` and Clap's `unrecognized subcommand` error and suggestions.
`#[arg(global = true)]` options and flags are accepted after subcommands as in Clap; their value is read from the struct declaring them. See [Subcommands][guide-subcommands].
Like `clap_complete` and `clap_mangen`, the `std` feature generates bash, zsh and fish completion scripts and a man page, leaving out hidden arguments and hidden aliases and completing possible values; see [Shell Completions and Man Pages][guide-completions].

This crate only implements a **Clap-inspired subset**. It is not a drop-in replacement for Clap and does not aim for full Clap API compatibility:
other `#[command(...)]` keys (`disable_version_flag`, `allow_hyphen_values`, `trailing_var_arg`, `before_long_help`, `after_long_help`, `term_width`, `next_line_help`, help templates, ...), wrapping at the terminal width, completion scripts for other shells (PowerShell, Elvish, ...), dynamic completion and value hints, the generated `help` subcommand, version flags on subcommands, subcommand settings (`subcommand_required`, `args_conflicts_with_subcommands`, `infer_subcommands`, `allow_external_subcommands`, `short_flag`, ...), global positional arguments, flattening structs with subcommands, flattening `Option<T>` fields, `Option<Vec<T>>` and fixed-size array fields, `num_args` on options and `default_values_t`, `Option<bool>` and `Option<Option<T>>` fields, other `#[value(...)]` keys (`skip`, `help`, `rename_all`, ...) and case-insensitive value enums, `value_parser!` builders and value parser expressions other than a function path or an integer range, actions other than `SetTrue`, `SetFalse` and `Count` (`Append` is implied by the field type),
relation variants such as `conflicts_with_all`, `requires_if` or `required_if_eq`, relations referencing groups, `#[group]` on structs, `env` on flags, `hide_env` and `hide_env_values`, and most other Clap attributes are not supported.
The help is computed at compile time, so it is only wrapped when `max_term_width` is set, at exactly that width; see [Generated Help][guide-help] for the other differences.
Unsupported attributes are rejected at compile time rather than silently ignored. See [Attribute Reference][guide-attributes] for the supported subset, [Flag Actions][guide-actions] for the supported actions, [Environment Variables][guide-env] for `env`, and [Value Enums][guide-value-enum] for value enums.

[guide-actions]: actions.md
[guide-aliases]: aliases.md
[guide-attributes]: attributes.md
[guide-completions]: completions.md
[guide-collections]: collections.md
[guide-env]: env.md
[guide-errors]: errors.md
[guide-flatten]: flatten.md
[guide-help]: help.md
[guide-parsing]: parsing.md
[guide-relations]: relations.md
[guide-subcommands]: subcommands.md
[guide-validation]: validation.md
[guide-value-enum]: value_enum.md
