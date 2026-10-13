//! Shell completion scripts and man pages generated from the [`Command`] metadata.
//!
//! Requires the `std` feature. See [Shell Completions and Man Pages][crate::guide::completions].

mod bash;
mod fish;
mod man;
mod zsh;

use crate::{ArgInfo, Command, CommandInfo, ParseValueEnumError};
use core::fmt;
use core::str::FromStr;
use std::io::{self, Write};
use std::string::String;
use std::vec::Vec;

/// A shell for which a completion script can be generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Shell {
    /// Bash.
    Bash,
    /// Zsh.
    Zsh,
    /// Fish.
    Fish,
}

impl Shell {
    /// Every shell, in the order of [`NAMES`](Self::NAMES).
    pub const ALL: &'static [Shell] = &[Shell::Bash, Shell::Zsh, Shell::Fish];

    /// The name of every shell, as accepted by [`FromStr`] and written by [`Display`](fmt::Display).
    pub const NAMES: &'static [&'static str] = &["bash", "zsh", "fish"];

    /// The name of the shell: `bash`, `zsh` or `fish`.
    pub const fn name(self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
        }
    }

    /// The usual file name of the completion script of the command `name`: `name.bash`, `_name`
    /// or `name.fish`.
    pub fn file_name(self, name: &str) -> String {
        match self {
            Shell::Bash => std::format!("{name}.bash"),
            Shell::Zsh => std::format!("_{name}"),
            Shell::Fish => std::format!("{name}.fish"),
        }
    }

    /// Renders the completion script of `command`.
    pub fn render(self, command: &CommandInfo) -> String {
        match self {
            Shell::Bash => bash::render(command),
            Shell::Zsh => zsh::render(command),
            Shell::Fish => fish::render(command),
        }
    }
}

impl FromStr for Shell {
    type Err = ParseValueEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|shell| shell.name() == value)
            .ok_or(ParseValueEnumError)
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Writes the completion script of `C` for `shell` to `output`.
pub fn generate<C: Command>(shell: Shell, output: &mut impl Write) -> io::Result<()> {
    output.write_all(shell.render(&C::COMMAND).as_bytes())
}

/// Writes the man page of `C`, in roff, to `output`.
pub fn man_page<C: Command>(output: &mut impl Write) -> io::Result<()> {
    output.write_all(render_man_page(&C::COMMAND).as_bytes())
}

/// Renders the man page of `command`, in roff.
pub fn render_man_page(command: &CommandInfo) -> String {
    man::render(command)
}

struct Node<'c> {
    path: Vec<&'c CommandInfo>,
    command: &'c CommandInfo,
    args: Vec<&'c ArgInfo>,
}

impl<'c> Node<'c> {
    fn root(command: &'c CommandInfo) -> Self {
        Node {
            path: std::vec![command],
            command,
            args: command.args.iter().collect(),
        }
    }

    fn id(&self) -> String {
        let names: Vec<String> = self
            .path
            .iter()
            .map(|command| identifier(command.name))
            .collect();
        names.join("__")
    }
}

fn tree(command: &CommandInfo) -> Vec<Node<'_>> {
    fn visit<'c>(node: Node<'c>, nodes: &mut Vec<Node<'c>>) {
        let globals: Vec<&ArgInfo> = node.args.iter().copied().filter(|arg| arg.global).collect();
        let children: Vec<Node<'c>> = node
            .command
            .commands
            .iter()
            .map(|child| {
                let mut path = node.path.clone();
                path.push(child);
                let mut args: Vec<&ArgInfo> = child.args.iter().collect();
                let inherited: Vec<&ArgInfo> = globals
                    .iter()
                    .copied()
                    .filter(|global| !args.iter().any(|arg| arg.id == global.id))
                    .collect();
                args.extend(inherited);
                Node {
                    path,
                    command: child,
                    args,
                }
            })
            .collect();
        nodes.push(node);
        for child in children {
            visit(child, nodes);
        }
    }
    let mut nodes = Vec::new();
    visit(Node::root(command), &mut nodes);
    nodes
}

fn visible<'n>(node: &'n Node<'_>) -> impl Iterator<Item = &'n ArgInfo> {
    node.args.iter().copied().filter(|arg| !arg.hidden)
}

fn options<'n>(node: &'n Node<'_>) -> impl Iterator<Item = &'n ArgInfo> {
    visible(node).filter(|arg| !arg.is_positional())
}

fn positionals<'n>(node: &'n Node<'_>) -> impl Iterator<Item = &'n ArgInfo> {
    visible(node).filter(|arg| arg.is_positional())
}

fn subcommands(command: &CommandInfo) -> impl Iterator<Item = &CommandInfo> {
    command.commands.iter().filter(|command| !command.hidden)
}

fn command_names(command: &CommandInfo) -> Vec<&'static str> {
    core::iter::once(command.name)
        .chain(command.visible_aliases.iter().copied())
        .collect()
}

fn all_command_names(command: &CommandInfo) -> Vec<&'static str> {
    core::iter::once(command.name)
        .chain(command.visible_aliases.iter().copied())
        .chain(command.aliases.iter().copied())
        .collect()
}

fn shorts(arg: &ArgInfo) -> Vec<char> {
    arg.short
        .into_iter()
        .chain(arg.visible_short_aliases.iter().copied())
        .collect()
}

fn longs(arg: &ArgInfo) -> Vec<&'static str> {
    arg.long
        .into_iter()
        .chain(arg.visible_aliases.iter().copied())
        .collect()
}

fn flags(arg: &ArgInfo) -> Vec<String> {
    let mut flags: Vec<String> = shorts(arg)
        .into_iter()
        .map(|short| std::format!("-{short}"))
        .collect();
    flags.extend(longs(arg).into_iter().map(|long| std::format!("--{long}")));
    flags
}

fn summary(text: &str) -> &str {
    text.lines().next().unwrap_or_default().trim()
}

fn identifier(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}
