use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(positional, long)]
    input: &'a str,
}

fn main() {}
