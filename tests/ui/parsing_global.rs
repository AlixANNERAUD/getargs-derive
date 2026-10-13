use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(global = true)]
    input: &'a str,
    #[arg(short, long, global = "yes")]
    verbose: bool,
}

fn main() {}
