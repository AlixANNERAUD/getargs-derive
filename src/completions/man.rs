use super::{Node, longs, options, positionals, shorts, summary, tree};
use crate::{ArgInfo, CommandInfo};
use std::string::String;
use std::vec::Vec;

fn escape(text: &str) -> String {
    let mut escaped = String::new();
    for character in text.chars() {
        match character {
            '\\' => escaped.push_str("\\e"),
            '-' => escaped.push_str("\\-"),
            other => escaped.push(other),
        }
    }
    escaped
}

fn text(output: &mut String, text: &str) {
    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            output.push_str(".sp\n");
            continue;
        }
        if line.starts_with(['.', '\'']) {
            output.push_str("\\&");
        }
        output.push_str(&escape(line));
        output.push('\n');
    }
}

fn bold(text: &str) -> String {
    std::format!("\\fB{}\\fR", escape(text))
}

fn italic(text: &str) -> String {
    std::format!("\\fI{}\\fR", escape(text))
}

fn names(arg: &ArgInfo) -> Vec<String> {
    let mut names: Vec<String> = shorts(arg)
        .into_iter()
        .map(|short| bold(&std::format!("-{short}")))
        .collect();
    names.extend(
        longs(arg)
            .into_iter()
            .map(|long| bold(&std::format!("--{long}"))),
    );
    names
}

fn path(node: &Node) -> String {
    let names: Vec<&str> = node.path.iter().map(|command| command.name).collect();
    names.join(" ")
}

fn synopsis(node: &Node) -> String {
    let mut synopsis = bold(&path(node));
    for arg in options(node) {
        let mut names = names(arg).join("|");
        if arg.takes_value() {
            names.push(' ');
            names.push_str(&italic(arg.value_name));
        }
        if arg.required {
            synopsis.push_str(&std::format!(" {names}"));
        } else {
            synopsis.push_str(&std::format!(" [{names}]"));
        }
    }
    for arg in positionals(node) {
        let mut value = italic(arg.value_name);
        if arg.last {
            value = std::format!("\\-\\- {value}");
        }
        synopsis.push(' ');
        synopsis.push_str(&positional(arg, value));
    }
    if !node.command.commands.is_empty() {
        let command = italic("COMMAND");
        if node.command.subcommand_required {
            synopsis.push_str(&std::format!(" <{command}>"));
        } else {
            synopsis.push_str(&std::format!(" [{command}]"));
        }
    }
    synopsis
}

fn subcommands_section(output: &mut String, command: &CommandInfo) {
    let nodes = tree(command);
    let visible: Vec<&Node> = nodes[1..]
        .iter()
        .filter(|node| !node.path.iter().any(|command| command.hidden))
        .collect();
    if visible.is_empty() {
        return;
    }
    output.push_str(".SH SUBCOMMANDS\n");
    for node in visible {
        output.push_str(".TP\n");
        output.push_str(&bold(&path(node)));
        output.push('\n');
        let about = summary(node.command.about);
        if !about.is_empty() {
            text(output, about);
            output.push_str(".br\n");
        }
        if !node.command.visible_aliases.is_empty() {
            output.push_str(&std::format!(
                "[aliases: {}]\n.br\n",
                escape(&node.command.visible_aliases.join(", "))
            ));
        }
        output.push_str(&synopsis(node));
        output.push('\n');
    }
}

fn positional(arg: &ArgInfo, value: String) -> String {
    let ellipsis = if arg.multiple { "..." } else { "" };
    match (arg.required, arg.last) {
        (true, false) => std::format!("<{value}>{ellipsis}"),
        (false, false) => std::format!("[{value}]{ellipsis}"),
        (true, true) => std::format!("<{value}{ellipsis}>"),
        (false, true) => std::format!("[{value}{ellipsis}]"),
    }
}

fn entry(output: &mut String, header: String, arg: &ArgInfo) {
    output.push_str(".TP\n");
    output.push_str(&header);
    output.push('\n');
    let help = arg.long_or_short_help();
    text(output, help);
    let mut extra = Vec::new();
    if let Some(env) = arg.env {
        extra.push(std::format!("[env: {}=]", escape(env)));
    }
    if !arg.possible_values.is_empty() {
        let values: Vec<String> = arg
            .possible_values
            .iter()
            .map(|value| escape(value))
            .collect();
        extra.push(std::format!(
            "[\\fIpossible values: \\fR{}]",
            values.join(", ")
        ));
    }
    if !extra.is_empty() {
        if !help.is_empty() {
            output.push_str(".sp\n");
        }
        output.push_str(&extra.join("\n.br\n"));
        output.push('\n');
    }
}

pub(super) fn render(command: &CommandInfo) -> String {
    let name = command.name;
    let root = Node::root(command);
    let mut output = String::new();
    match command.version {
        Some(version) => output.push_str(&std::format!(
            ".TH {} 1 \"\" \"{} {}\"\n",
            escape(name),
            escape(name),
            escape(version)
        )),
        None => output.push_str(&std::format!(".TH {} 1\n", escape(name))),
    }
    output.push_str(".SH NAME\n");
    match summary(command.about) {
        "" => output.push_str(&std::format!("{}\n", escape(name))),
        about => output.push_str(&std::format!("{} \\- {}\n", escape(name), escape(about))),
    }
    output.push_str(".SH SYNOPSIS\n");
    output.push_str(&synopsis(&root));
    output.push('\n');
    let description = match command.long_about {
        "" => command.about,
        long_about => long_about,
    };
    if !description.is_empty() {
        output.push_str(".SH DESCRIPTION\n");
        text(&mut output, description);
    }
    if positionals(&root).next().is_some() {
        output.push_str(".SH ARGUMENTS\n");
        for arg in positionals(&root) {
            let header = match (arg.required, arg.multiple) {
                (true, true) => std::format!("<{}>...", italic(arg.value_name)),
                (true, false) => std::format!("<{}>", italic(arg.value_name)),
                (false, true) => std::format!("[{}]...", italic(arg.value_name)),
                (false, false) => std::format!("[{}]", italic(arg.value_name)),
            };
            entry(&mut output, header, arg);
        }
    }
    if options(&root).next().is_some() {
        output.push_str(".SH OPTIONS\n");
        for arg in options(&root) {
            let mut header = names(arg).join(", ");
            if arg.takes_value() {
                header.push_str(&std::format!("={}", italic(arg.value_name)));
            }
            entry(&mut output, header, arg);
        }
    }
    subcommands_section(&mut output, command);
    if let Some(version) = command.version {
        output.push_str(".SH VERSION\n");
        output.push_str(&std::format!("v{}\n", escape(version)));
    }
    output
}
