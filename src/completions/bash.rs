use super::{
    Node, all_command_names, command_names, flags, identifier, options, positionals, subcommands,
    tree,
};
use crate::CommandInfo;
use std::string::String;
use std::vec::Vec;

fn quote(text: &str) -> String {
    let mut quoted = String::new();
    for character in text.chars() {
        if matches!(character, '\\' | '"' | '$' | '`') {
            quoted.push('\\');
        }
        quoted.push(character);
    }
    quoted
}

fn words(values: &[&str]) -> String {
    let values: Vec<String> = values.iter().map(|value| quote(value)).collect();
    values.join(" ")
}

fn reply(output: &mut String, indent: &str, values: &[&str]) {
    let generator = if values.is_empty() {
        String::from("-f")
    } else {
        std::format!("-W \"{}\"", words(values))
    };
    output.push_str(&std::format!(
        "{indent}COMPREPLY=($(compgen {generator} -- \"${{cur}}\"))\n{indent}return 0\n"
    ));
}

fn body(output: &mut String, node: &Node, indent: &str, first: &str) {
    let all_flags: Vec<String> = options(node).flat_map(flags).collect();
    let valued: Vec<_> = options(node)
        .filter(|arg| arg.takes_value())
        .map(|arg| (flags(arg).join("|"), arg.possible_values))
        .collect();

    output.push_str(&std::format!("{indent}opts=\"{}\"\n", all_flags.join(" ")));

    if !valued.is_empty() {
        output.push_str(&std::format!("\n{indent}case \"${{prev}}\" in\n"));
        for (pattern, values) in &valued {
            output.push_str(&std::format!("{indent}    {pattern})\n"));
            reply(output, &std::format!("{indent}        "), values);
            output.push_str(&std::format!("{indent}        ;;\n"));
        }
        output.push_str(&std::format!("{indent}esac\n"));
    }

    output.push_str(&std::format!("\n{indent}if [[ ${{cur}} == -* ]]; then\n"));
    output.push_str(&std::format!(
        "{indent}    COMPREPLY=($(compgen -W \"${{opts}}\" -- \"${{cur}}\"))\n"
    ));
    output.push_str(&std::format!("{indent}    return 0\n"));
    output.push_str(&std::format!("{indent}fi\n"));

    let positional_values: Vec<_> = positionals(node)
        .enumerate()
        .filter(|(_, arg)| !arg.possible_values.is_empty())
        .collect();
    if !positional_values.is_empty() {
        output.push_str(&std::format!("\n{indent}local i position=0\n"));
        output.push_str(&std::format!(
            "{indent}for (( i = {first}; i < COMP_CWORD; i++ )); do\n"
        ));
        output.push_str(&std::format!(
            "{indent}    case \"${{COMP_WORDS[i]}}\" in\n"
        ));
        for (pattern, _) in &valued {
            output.push_str(&std::format!("{indent}        {pattern}) (( i++ )) ;;\n"));
        }
        output.push_str(&std::format!("{indent}        -*) ;;\n"));
        output.push_str(&std::format!("{indent}        *) (( position++ )) ;;\n"));
        output.push_str(&std::format!("{indent}    esac\n"));
        output.push_str(&std::format!("{indent}done\n"));
        if let Some(index) = positionals(node).position(|arg| arg.multiple && !arg.last) {
            output.push_str(&std::format!(
                "{indent}(( position > {index} )) && position={index}\n"
            ));
        }
        output.push_str(&std::format!("{indent}case \"${{position}}\" in\n"));
        for (index, arg) in positional_values {
            output.push_str(&std::format!("{indent}    {index})\n"));
            reply(
                output,
                &std::format!("{indent}        "),
                arg.possible_values,
            );
            output.push_str(&std::format!("{indent}        ;;\n"));
        }
        output.push_str(&std::format!("{indent}esac\n"));
    }

    output.push('\n');
    let names: Vec<&str> = subcommands(node.command).flat_map(command_names).collect();
    reply(output, indent, &names);
}

fn header(output: &mut String, function: &str, locals: &str) {
    output.push_str(&std::format!("{function}() {{\n"));
    output.push_str(&std::format!("    local {locals}\n"));
    output.push_str("    COMPREPLY=()\n");
    output.push_str("    cur=\"${COMP_WORDS[COMP_CWORD]}\"\n");
    output.push_str("    prev=\"${COMP_WORDS[COMP_CWORD-1]}\"\n");
}

pub(super) fn render(command: &CommandInfo) -> String {
    let name = command.name;
    let function = std::format!("_{}", identifier(name));
    let mut output = String::new();
    if command.commands.is_empty() {
        header(&mut output, &function, "cur prev opts");
        body(&mut output, &Node::root(command), "    ", "1");
    } else {
        let nodes = tree(command);
        header(&mut output, &function, "cur prev opts cmd start index");
        output.push_str(&std::format!("    cmd=\"{}\"\n", nodes[0].id()));
        output.push_str("    start=0\n");
        output.push_str("    for (( index = 1; index < COMP_CWORD; index++ )); do\n");
        output.push_str("        case \"${cmd},${COMP_WORDS[index]}\" in\n");
        for node in &nodes {
            for child in node.command.commands {
                let mut path = node.path.clone();
                path.push(child);
                let id = Node {
                    path,
                    command: child,
                    args: Vec::new(),
                }
                .id();
                let patterns: Vec<String> = all_command_names(child)
                    .iter()
                    .map(|name| std::format!("\"{},{}\"", node.id(), quote(name)))
                    .collect();
                output.push_str(&std::format!(
                    "            {})\n                cmd=\"{id}\"\n                start=${{index}}\n                ;;\n",
                    patterns.join("|")
                ));
            }
        }
        output.push_str("        esac\n");
        output.push_str("    done\n\n");
        output.push_str("    case \"${cmd}\" in\n");
        for node in &nodes {
            output.push_str(&std::format!("        {})\n", node.id()));
            body(&mut output, node, "            ", "start + 1");
            output.push_str("            ;;\n");
        }
        output.push_str("    esac\n");
    }
    output.push_str("}\n\n");
    output.push_str(&std::format!(
        "complete -F {function} -o bashdefault -o default {name}\n"
    ));
    output
}
