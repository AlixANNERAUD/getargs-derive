use super::{flags, identifier, longs, options, positionals, shorts, summary};
use crate::{ArgInfo, CommandInfo};
use std::string::String;
use std::vec::Vec;

fn escape(text: &str, extra: &[char]) -> String {
    let mut escaped = String::new();
    for character in text.chars() {
        match character {
            '\'' => escaped.push_str("'\\''"),
            '\n' => escaped.push(' '),
            '\\' | '[' | ']' | ':' | '$' | '`' => {
                escaped.push('\\');
                escaped.push(character);
            }
            other if extra.contains(&other) => {
                escaped.push('\\');
                escaped.push(other);
            }
            other => escaped.push(other),
        }
    }
    escaped
}

fn help(text: &str) -> String {
    escape(summary(text), &[])
}

fn action(arg: &ArgInfo) -> String {
    if arg.possible_values.is_empty() {
        return String::from("_default");
    }
    let values: Vec<String> = arg
        .possible_values
        .iter()
        .map(|value| escape(value, &['(', ')', ' ']))
        .collect();
    std::format!("({})", values.join(" "))
}

fn option_lines(arg: &ArgInfo, output: &mut String) {
    let exclusion = if arg.multiple {
        String::from("*")
    } else {
        std::format!("({})", flags(arg).join(" "))
    };
    let value = if arg.takes_value() {
        std::format!(":{}:{}", escape(arg.value_name, &[]), action(arg))
    } else {
        String::new()
    };
    let help = help(arg.help);
    let mut names: Vec<String> = shorts(arg)
        .into_iter()
        .map(|short| match arg.takes_value() {
            true => std::format!("-{short}+"),
            false => std::format!("-{short}"),
        })
        .collect();
    names.extend(longs(arg).into_iter().map(|long| match arg.takes_value() {
        true => std::format!("--{long}="),
        false => std::format!("--{long}"),
    }));
    for name in names {
        output.push_str(&std::format!("'{exclusion}{name}[{help}]{value}' \\\n"));
    }
}

pub(super) fn render(command: &CommandInfo) -> String {
    let name = command.name;
    let function = std::format!("_{}", identifier(name));
    let mut output = String::new();
    output.push_str(&std::format!("#compdef {name}\n\n"));
    output.push_str("autoload -U is-at-least\n\n");
    output.push_str(&std::format!("{function}() {{\n"));
    output.push_str("    typeset -A opt_args\n");
    output.push_str("    typeset -a _arguments_options\n");
    output.push_str("    local ret=1\n\n");
    output.push_str("    if is-at-least 5.2; then\n");
    output.push_str("        _arguments_options=(-s -S -C)\n");
    output.push_str("    else\n");
    output.push_str("        _arguments_options=(-s -C)\n");
    output.push_str("    fi\n\n");
    output.push_str("    local context curcontext=\"$curcontext\" state line\n");
    output.push_str("    _arguments \"${_arguments_options[@]}\" : \\\n");
    for arg in options(command) {
        option_lines(arg, &mut output);
    }
    for arg in positionals(command) {
        let optional = if arg.required { ":" } else { "::" };
        let description = match summary(arg.help) {
            "" => String::new(),
            text => std::format!(" -- {}", help(text)),
        };
        output.push_str(&std::format!(
            "'{optional}{}{description}:{}' \\\n",
            escape(arg.id, &[]),
            action(arg)
        ));
    }
    output.push_str("&& ret=0\n");
    output.push_str("}\n\n");
    output.push_str(&std::format!(
        "if [ \"$funcstack[1]\" = \"{function}\" ]; then\n"
    ));
    output.push_str(&std::format!("    {function} \"$@\"\n"));
    output.push_str("else\n");
    output.push_str(&std::format!("    compdef {function} {name}\n"));
    output.push_str("fi\n");
    output
}
