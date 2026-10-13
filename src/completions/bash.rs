use super::{flags, identifier, options, positionals};
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

pub(super) fn render(command: &CommandInfo) -> String {
    let name = command.name;
    let function = std::format!("_{}", identifier(name));
    let all_flags: Vec<String> = options(command).flat_map(flags).collect();
    let valued: Vec<_> = options(command)
        .filter(|arg| arg.takes_value())
        .map(|arg| (flags(arg).join("|"), arg.possible_values))
        .collect();

    let mut output = String::new();
    output.push_str(&std::format!("{function}() {{\n"));
    output.push_str("    local cur prev opts\n");
    output.push_str("    COMPREPLY=()\n");
    output.push_str("    cur=\"${COMP_WORDS[COMP_CWORD]}\"\n");
    output.push_str("    prev=\"${COMP_WORDS[COMP_CWORD-1]}\"\n");
    output.push_str(&std::format!("    opts=\"{}\"\n", all_flags.join(" ")));

    if !valued.is_empty() {
        output.push_str("\n    case \"${prev}\" in\n");
        for (pattern, values) in &valued {
            output.push_str(&std::format!("        {pattern})\n"));
            reply(&mut output, "            ", values);
            output.push_str("            ;;\n");
        }
        output.push_str("    esac\n");
    }

    output.push_str("\n    if [[ ${cur} == -* ]]; then\n");
    output.push_str("        COMPREPLY=($(compgen -W \"${opts}\" -- \"${cur}\"))\n");
    output.push_str("        return 0\n");
    output.push_str("    fi\n");

    let positional_values: Vec<_> = positionals(command)
        .enumerate()
        .filter(|(_, arg)| !arg.possible_values.is_empty())
        .collect();
    if !positional_values.is_empty() {
        output.push_str("\n    local i position=0\n");
        output.push_str("    for (( i = 1; i < COMP_CWORD; i++ )); do\n");
        output.push_str("        case \"${COMP_WORDS[i]}\" in\n");
        for (pattern, _) in &valued {
            output.push_str(&std::format!("            {pattern}) (( i++ )) ;;\n"));
        }
        output.push_str("            -*) ;;\n");
        output.push_str("            *) (( position++ )) ;;\n");
        output.push_str("        esac\n");
        output.push_str("    done\n");
        if let Some(index) = positionals(command).position(|arg| arg.multiple && !arg.last) {
            output.push_str(&std::format!(
                "    (( position > {index} )) && position={index}\n"
            ));
        }
        output.push_str("    case \"${position}\" in\n");
        for (index, arg) in positional_values {
            output.push_str(&std::format!("        {index})\n"));
            reply(&mut output, "            ", arg.possible_values);
            output.push_str("            ;;\n");
        }
        output.push_str("    esac\n");
    }

    output.push('\n');
    reply(&mut output, "    ", &[]);
    output.push_str("}\n\n");
    output.push_str(&std::format!(
        "complete -F {function} -o bashdefault -o default {name}\n"
    ));
    output
}
