use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(last = true)]
    rest: Option<&'a str>,
    input: Option<&'a str>,
}

#[derive(GetArgs)]
struct Twice<'a> {
    #[arg(last = true)]
    first: Option<&'a str>,
    #[arg(last = true)]
    second: Option<&'a str>,
}

fn main() {}
