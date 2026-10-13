/// Compile-time description of a command line interface, implemented by
/// [`#[derive(GetArgs)]`](derive@crate::GetArgs).
///
/// The metadata is `&'static` and available without the `std` feature. It is what the shell
/// completion and man page generators of the `std` feature read, and can be inspected to build
/// other tools.
/// See [Shell Completions and Man Pages][crate::guide::completions].
pub trait Command {
    /// The description of the command.
    const COMMAND: CommandInfo;
}

/// The description of a command: its name, texts, arguments and subcommands.
///
/// Values are built with [`CommandInfo::new`] and the `const` builder methods, so that fields can
/// be added without breaking existing code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct CommandInfo {
    /// The name of the command, as typed on the command line.
    pub name: &'static str,
    /// The short description, from `about` or the first paragraph of the doc comment. Empty when
    /// not set.
    pub about: &'static str,
    /// The long description, from `long_about` or the full doc comment. Empty when not set.
    pub long_about: &'static str,
    /// The version set with `#[command(version)]`.
    pub version: Option<&'static str>,
    /// The arguments, flattened structs included, in field order, followed by the generated
    /// `--help` and `--version` flags.
    pub args: &'static [ArgInfo],
    /// The subcommands. Always empty for now.
    pub commands: &'static [CommandInfo],
}

impl CommandInfo {
    /// A command named `name`, without description, version, arguments or subcommands.
    pub const fn new(name: &'static str) -> Self {
        CommandInfo {
            name,
            about: "",
            long_about: "",
            version: None,
            args: &[],
            commands: &[],
        }
    }

    /// Sets [`about`](Self::about).
    pub const fn about(mut self, about: &'static str) -> Self {
        self.about = about;
        self
    }

    /// Sets [`long_about`](Self::long_about).
    pub const fn long_about(mut self, long_about: &'static str) -> Self {
        self.long_about = long_about;
        self
    }

    /// Sets [`version`](Self::version).
    pub const fn version(mut self, version: &'static str) -> Self {
        self.version = Some(version);
        self
    }

    /// Sets [`args`](Self::args).
    pub const fn args(mut self, args: &'static [ArgInfo]) -> Self {
        self.args = args;
        self
    }

    /// Sets [`commands`](Self::commands).
    pub const fn commands(mut self, commands: &'static [CommandInfo]) -> Self {
        self.commands = commands;
        self
    }
}

/// How an argument is given on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ArgKind {
    /// A positional argument.
    Positional,
    /// A named option taking a value.
    Option,
    /// A named flag taking no value.
    Flag,
}

/// The description of one argument.
///
/// Values are built with [`ArgInfo::new`] and the `const` builder methods, so that fields can be
/// added without breaking existing code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct ArgInfo {
    /// The name of the field, or `help` and `version` for the generated flags.
    pub id: &'static str,
    /// How the argument is given.
    pub kind: ArgKind,
    /// The short name, without the leading `-`.
    pub short: Option<char>,
    /// The long name, without the leading `--`.
    pub long: Option<&'static str>,
    /// The short aliases listed in the help.
    pub visible_short_aliases: &'static [char],
    /// The short aliases hidden from the help.
    pub short_aliases: &'static [char],
    /// The long aliases listed in the help.
    pub visible_aliases: &'static [&'static str],
    /// The long aliases hidden from the help.
    pub aliases: &'static [&'static str],
    /// The name of the value, e.g. `FILE`. Empty for flags.
    pub value_name: &'static str,
    /// The short help. Empty when not set.
    pub help: &'static str,
    /// The long help. Empty when not set.
    pub long_help: &'static str,
    /// The accepted values, from `possible_values` or a [`ValueEnum`](crate::ValueEnum). Empty
    /// when any value is accepted.
    pub possible_values: &'static [&'static str],
    /// The environment variable the argument falls back to.
    pub env: Option<&'static str>,
    /// Whether the argument is hidden from the help.
    pub hidden: bool,
    /// Whether the argument must be given.
    pub required: bool,
    /// Whether the argument can be given several times, e.g. a `Count` flag.
    pub multiple: bool,
    /// Whether the positional argument is only filled after `--`.
    pub last: bool,
}

impl ArgInfo {
    /// An argument named `id`, without names, help or values.
    pub const fn new(id: &'static str, kind: ArgKind) -> Self {
        ArgInfo {
            id,
            kind,
            short: None,
            long: None,
            visible_short_aliases: &[],
            short_aliases: &[],
            visible_aliases: &[],
            aliases: &[],
            value_name: "",
            help: "",
            long_help: "",
            possible_values: &[],
            env: None,
            hidden: false,
            required: false,
            multiple: false,
            last: false,
        }
    }

    /// Sets [`short`](Self::short).
    pub const fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    /// Sets [`long`](Self::long).
    pub const fn long(mut self, long: &'static str) -> Self {
        self.long = Some(long);
        self
    }

    /// Sets [`visible_short_aliases`](Self::visible_short_aliases).
    pub const fn visible_short_aliases(mut self, aliases: &'static [char]) -> Self {
        self.visible_short_aliases = aliases;
        self
    }

    /// Sets [`short_aliases`](Self::short_aliases).
    pub const fn short_aliases(mut self, aliases: &'static [char]) -> Self {
        self.short_aliases = aliases;
        self
    }

    /// Sets [`visible_aliases`](Self::visible_aliases).
    pub const fn visible_aliases(mut self, aliases: &'static [&'static str]) -> Self {
        self.visible_aliases = aliases;
        self
    }

    /// Sets [`aliases`](Self::aliases).
    pub const fn aliases(mut self, aliases: &'static [&'static str]) -> Self {
        self.aliases = aliases;
        self
    }

    /// Sets [`value_name`](Self::value_name).
    pub const fn value_name(mut self, value_name: &'static str) -> Self {
        self.value_name = value_name;
        self
    }

    /// Sets [`help`](Self::help).
    pub const fn help(mut self, help: &'static str) -> Self {
        self.help = help;
        self
    }

    /// Sets [`long_help`](Self::long_help).
    pub const fn long_help(mut self, long_help: &'static str) -> Self {
        self.long_help = long_help;
        self
    }

    /// Sets [`possible_values`](Self::possible_values).
    pub const fn possible_values(mut self, values: &'static [&'static str]) -> Self {
        self.possible_values = values;
        self
    }

    /// Sets [`env`](Self::env).
    pub const fn env(mut self, env: &'static str) -> Self {
        self.env = Some(env);
        self
    }

    /// Sets [`hidden`](Self::hidden).
    pub const fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Sets [`required`](Self::required).
    pub const fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Sets [`multiple`](Self::multiple).
    pub const fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// Sets [`last`](Self::last).
    pub const fn last(mut self, last: bool) -> Self {
        self.last = last;
        self
    }

    /// Whether the argument takes a value: options and positional arguments.
    pub const fn takes_value(&self) -> bool {
        matches!(self.kind, ArgKind::Positional | ArgKind::Option)
    }

    /// Whether the argument is positional.
    pub const fn is_positional(&self) -> bool {
        matches!(self.kind, ArgKind::Positional)
    }

    /// The long help, or the short help when there is no long help.
    pub const fn long_or_short_help(&self) -> &'static str {
        if self.long_help.is_empty() {
            self.help
        } else {
            self.long_help
        }
    }
}
