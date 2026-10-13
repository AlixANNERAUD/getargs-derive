use getargs_derive::{GetArgs, Values};

#[derive(GetArgs)]
struct Cli<'a> {
    files: Values<&'a str, 4>,
    output: Option<&'a str>,
}

fn main() {}
