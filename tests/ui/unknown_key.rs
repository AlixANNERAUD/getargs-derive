use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, global = true)]
    verbose: bool,
}

fn main() {}
