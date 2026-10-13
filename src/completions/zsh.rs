use super::{
    Node, all_command_names, command_names, flags, identifier, longs, options, positionals, shorts,
    subcommands, summary, tree,
};
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

fn function_start(output: &mut String, function: &str) {
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
}

fn arguments(output: &mut String, node: &Node) -> bool {
    for arg in options(node) {
        option_lines(arg, output);
    }
    for arg in positionals(node) {
        let optional = match (arg.multiple, arg.required) {
            (true, _) => "*::",
            (false, true) => ":",
            (false, false) => "::",
        };
        let description = match summary(arg.help) {
            "" => String::new(),
            text => std::format!(" -- {}", help(text)),
        };
        output.push_str(&std::format!(
            "'{optional}{}{description}:{}' \\\n",
            escape(arg.id, &[]),
            action(arg)
        ));
        if arg.multiple {
            return false;
        }
    }
    true
}

fn footer(output: &mut String, function: &str, name: &str) {
    output.push_str(&std::format!(
        "if [ \"$funcstack[1]\" = \"{function}\" ]; then\n"
    ));
    output.push_str(&std::format!("    {function} \"$@\"\n"));
    output.push_str("else\n");
    output.push_str(&std::format!("    compdef {function} {name}\n"));
    output.push_str("fi\n");
}

fn node_function(output: &mut String, node: &Node) {
    let function = std::format!("_{}", node.id());
    function_start(output, &function);
    let open = arguments(output, node);
    let commands = !node.command.commands.is_empty() && open;
    if commands {
        let optional = if node.command.subcommand_required {
            ""
        } else {
            ":"
        };
        output.push_str(&std::format!(
            "'{optional}:command:{function}_commands' \\\n"
        ));
        output.push_str("'*::arg:->args' \\\n");
    }
    output.push_str("&& ret=0\n");
    if commands {
        output.push_str("    case $state in\n");
        output.push_str("    (args)\n");
        output.push_str(&std::format!(
            "        curcontext=\"${{curcontext%:*:*}}:{}-command-$words[1]:\"\n",
            node.id()
        ));
        output.push_str("        case $words[1] in\n");
        for child in node.command.commands {
            let mut path = node.path.clone();
            path.push(child);
            let id = Node {
                path,
                command: child,
                args: Vec::new(),
            }
            .id();
            let names: Vec<String> = all_command_names(child)
                .iter()
                .map(|name| escape(name, &['(', ')', '|', ' ']))
                .collect();
            output.push_str(&std::format!("        ({})\n", names.join("|")));
            output.push_str(&std::format!("            _{id} && ret=0\n"));
            output.push_str("            ;;\n");
        }
        output.push_str("        esac\n");
        output.push_str("        ;;\n");
        output.push_str("    esac\n");
    }
    output.push_str("    return ret\n");
    output.push_str("}\n\n");
    if commands {
        output.push_str(&std::format!("{function}_commands() {{\n"));
        output.push_str("    local commands; commands=(\n");
        for child in subcommands(node.command) {
            let description = help(child.about);
            for name in command_names(child) {
                output.push_str(&std::format!(
                    "        '{}:{description}'\n",
                    escape(name, &[])
                ));
            }
        }
        output.push_str("    )\n");
        output.push_str(&std::format!(
            "    _describe -t commands '{} commands' commands \"$@\"\n",
            escape(
                &node
                    .path
                    .iter()
                    .map(|command| command.name)
                    .collect::<Vec<_>>()
                    .join(" "),
                &[]
            )
        ));
        output.push_str("}\n\n");
    }
}

pub(super) fn render(command: &CommandInfo) -> String {
    let name = command.name;
    let function = std::format!("_{}", identifier(name));
    let mut output = String::new();
    output.push_str(&std::format!("#compdef {name}\n\n"));
    output.push_str("autoload -U is-at-least\n\n");
    if command.commands.is_empty() {
        function_start(&mut output, &function);
        arguments(&mut output, &Node::root(command));
        output.push_str("&& ret=0\n");
        output.push_str("}\n\n");
    } else {
        for node in tree(command) {
            node_function(&mut output, &node);
        }
    }
    footer(&mut output, &function, name);
    output
}
