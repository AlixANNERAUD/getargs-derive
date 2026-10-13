use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,
}

fn main() {}
