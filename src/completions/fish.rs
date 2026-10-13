use super::{longs, options, positionals, shorts, summary};
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

pub(super) fn render(command: &CommandInfo) -> String {
    let name = word(command.name);
    let mut output = String::new();
    for arg in options(command) {
        let mut line = std::format!("complete -c {name}");
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
    for arg in positionals(command).filter(|arg| !arg.possible_values.is_empty()) {
        let mut line = std::format!("complete -c {name} -a {}", values(arg.possible_values));
        let help = summary(arg.help);
        if !help.is_empty() {
            line.push_str(&std::format!(" -d {}", quote(help)));
        }
        output.push_str(&line);
        output.push('\n');
    }
    output
}
