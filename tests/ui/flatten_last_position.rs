use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Trailing<'a> {
    #[arg(last = true)]
    rest: Option<&'a str>,
}

#[derive(GetArgs)]
struct Cli<'a> {
    #[command(flatten)]
    trailing: Trailing<'a>,
    input: Option<&'a str>,
}

fn main() {}
