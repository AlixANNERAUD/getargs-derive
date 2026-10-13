use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, env = "VERBOSE")]
    verbose: bool,
    #[arg(short, action = Count, env)]
    quiet: u8,
}

fn main() {}
