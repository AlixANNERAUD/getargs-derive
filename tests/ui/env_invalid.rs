use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(long, env = "")]
    empty: &'a str,
    #[arg(long, env = "A=B")]
    equals: &'a str,
    #[arg(long, env = PORT)]
    unquoted: u16,
    #[arg(long, env = "ONE", env = "TWO")]
    twice: u16,
}

fn main() {}
