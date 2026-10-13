use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, flag)]
    count: u32,
}

fn main() {}
