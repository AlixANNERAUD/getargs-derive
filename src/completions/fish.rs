use super::{
    Node, all_command_names, command_names, longs, options, positionals, shorts, subcommands,
    summary, tree,
};
use crate::CommandInfo;
use std::string::String;
use std::vec::Vec;

fn quote(text: &str) -> String {
    let mut quoted = String::from("'");
    for character in text.chars() {
        if matches!(character, '\\' | '\'') {
            quoted.push('\\');
        }
        quoted.push(character);
    }
    quoted.push('\'');
    quoted
}

fn word(text: &str) -> String {
    let plain = !text.is_empty()
        && text.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        });
    if plain {
        String::from(text)
    } else {
        quote(text)
    }
}

fn values(values: &[&str]) -> String {
    let values: Vec<&str> = values.to_vec();
    quote(&values.join(" "))
}

fn seen(commands: &[&CommandInfo]) -> String {
    let names: Vec<String> = commands
        .iter()
        .flat_map(|command| all_command_names(command))
        .map(word)
        .collect();
    std::format!("__fish_seen_subcommand_from {}", names.join(" "))
}

fn condition(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        parts => std::format!(" -n {}", quote(&parts.join("; and "))),
    }
}

pub(super) fn render(command: &CommandInfo) -> String {
    let name = word(command.name);
    let mut output = String::new();
    for node in tree(command) {
        let scope: Vec<String> = node.path[1..]
            .iter()
            .map(|command| seen(&[command]))
            .collect();
        let mut here = scope.clone();
        if !node.command.commands.is_empty() {
            let children: Vec<&CommandInfo> = node.command.commands.iter().collect();
            here.push(std::format!("not {}", seen(&children)));
        }
        let own = Node {
            path: node.path.clone(),
            command: node.command,
            args: node.command.args.iter().collect(),
        };
        render_node(
            &mut output,
            &name,
            &own,
            &condition(&scope),
            &condition(&here),
        );
        for child in subcommands(node.command) {
            for child_name in command_names(child) {
                let mut line = std::format!(
                    "complete -c {name}{} -f -a {}",
                    condition(&here),
                    word(child_name)
                );
                let help = summary(child.about);
                if !help.is_empty() {
                    line.push_str(&std::format!(" -d {}", quote(help)));
                }
                output.push_str(&line);
                output.push('\n');
            }
        }
    }
    output
}

fn render_node(output: &mut String, name: &str, node: &Node, scope: &str, here: &str) {
    for arg in options(node) {
        let condition = if arg.global { scope } else { here };
        let mut line = std::format!("complete -c {name}{condition}");
        for short in shorts(arg) {
            line.push_str(&std::format!(" -s {}", word(&String::from(short))));
        }
        for long in longs(arg) {
            line.push_str(&std::format!(" -l {}", word(long)));
        }
        let help = summary(arg.help);
        if !help.is_empty() {
            line.push_str(&std::format!(" -d {}", quote(help)));
        }
        if arg.takes_value() {
            line.push_str(" -r");
            if !arg.possible_values.is_empty() {
                line.push_str(&std::format!(" -f -a {}", values(arg.possible_values)));
            }
        }
        output.push_str(&line);
        output.push('\n');
    }
    for arg in positionals(node).filter(|arg| !arg.possible_values.is_empty()) {
        let mut line = std::format!(
            "complete -c {name}{here} -a {}",
            values(arg.possible_values)
        );
        let help = summary(arg.help);
        if !help.is_empty() {
            line.push_str(&std::format!(" -d {}", quote(help)));
        }
        output.push_str(&line);
        output.push('\n');
    }
}
