use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    output: Option<&'a str>,
    input: &'a str,
}

fn main() {}
