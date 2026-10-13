use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, env = "VERBOSE")]
    verbose: bool,
}

fn main() {}
