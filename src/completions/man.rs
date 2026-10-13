use super::{longs, options, positionals, shorts, summary};
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

fn synopsis(command: &CommandInfo) -> String {
    let mut synopsis = bold(command.name);
    for arg in options(command) {
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
    for arg in positionals(command) {
        let mut value = italic(arg.value_name);
        if arg.last {
            value = std::format!("\\-\\- {value}");
        }
        if arg.required {
            synopsis.push_str(&std::format!(" <{value}>"));
        } else {
            synopsis.push_str(&std::format!(" [{value}]"));
        }
    }
    synopsis
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
    output.push_str(&synopsis(command));
    output.push('\n');
    let description = match command.long_about {
        "" => command.about,
        long_about => long_about,
    };
    if !description.is_empty() {
        output.push_str(".SH DESCRIPTION\n");
        text(&mut output, description);
    }
    if positionals(command).next().is_some() {
        output.push_str(".SH ARGUMENTS\n");
        for arg in positionals(command) {
            let header = match arg.required {
                true => std::format!("<{}>", italic(arg.value_name)),
                false => std::format!("[{}]", italic(arg.value_name)),
            };
            entry(&mut output, header, arg);
        }
    }
    if options(command).next().is_some() {
        output.push_str(".SH OPTIONS\n");
        for arg in options(command) {
            let mut header = names(arg).join(", ");
            if arg.takes_value() {
                header.push_str(&std::format!("={}", italic(arg.value_name)));
            }
            entry(&mut output, header, arg);
        }
    }
    if let Some(version) = command.version {
        output.push_str(".SH VERSION\n");
        output.push_str(&std::format!("v{}\n", escape(version)));
    }
    output
}
