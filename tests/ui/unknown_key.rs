use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, help = "verbose output")]
    verbose: bool,
}

fn main() {}
