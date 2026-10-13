# Clap Compatibility

The attributes follow the conventions of [Clap's derive API](https://docs.rs/clap/latest/clap/_derive/index.html):
`#[arg(short, long)]`, `short = 'x'`, `long = "name"`, `default_value`, `default_value_t`, `help`, `value_name` and the `SetTrue`, `SetFalse` and `Count` actions mean the same thing here as they do in Clap,
and so do the `name`, `about`, `version` and `disable_help_flag` keys of `#[command(...)]`. Doc comments become help text the way they do in Clap, and the generated help uses Clap's layout.
`Option<T>` fields are optional arguments that are `None` when absent, as in Clap.

This crate only implements a **Clap-inspired subset**. It is not a drop-in replacement for Clap and does not aim for full Clap API compatibility:
other `#[command(...)]` keys (`long_about`, `disable_version_flag`, ...), long help, subcommands, `Vec<T>` fields, `Option<bool>` and `Option<Option<T>>` fields, value enums, actions other than `SetTrue`, `SetFalse` and `Count`, and most other Clap attributes are not supported.
Unsupported attributes are rejected at compile time rather than silently ignored. See [Attribute Reference][guide-attributes] for the supported subset, and [Flag Actions][guide-actions] for the supported actions.

[guide-actions]: actions.md
[guide-attributes]: attributes.md
