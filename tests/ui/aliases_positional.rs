use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(alias = "file")]
    input: &'a str,
}

fn main() {}
