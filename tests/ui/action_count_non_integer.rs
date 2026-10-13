use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(short, long, action = Count)]
    verbose: bool,
    #[arg(long, action = ArgAction::Count)]
    level: &'a str,
}

fn main() {}
