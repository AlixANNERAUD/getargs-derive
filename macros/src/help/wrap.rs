pub(super) fn width(text: &str) -> usize {
    text.lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
}

pub(super) fn wrap(text: &str, width: Option<usize>) -> String {
    let Some(width) = width else {
        return text.to_owned();
    };
    text.split('\n')
        .map(|line| wrap_line(line, width.max(1)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn wrap_line(line: &str, width: usize) -> String {
    if line.chars().count() <= width {
        return line.to_owned();
    }
    let body = line.trim_start();
    let mut output = line[..line.len() - body.len()].to_owned();
    let mut current = output.chars().count();
    let mut line_start = true;
    for word in body.split_whitespace() {
        let length = word.chars().count();
        if !line_start && current + 1 + length > width {
            output.push('\n');
            current = 0;
        } else if !line_start {
            output.push(' ');
            current += 1;
        }
        output.push_str(word);
        current += length;
        line_start = false;
    }
    output
}

pub(super) fn indent(text: &str, indent: &str) -> String {
    let mut output = String::new();
    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            output.push('\n');
            if !line.is_empty() {
                output.push_str(indent);
            }
        }
        output.push_str(line);
    }
    output
}
