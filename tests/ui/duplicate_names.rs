use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
    #[arg(short, long)]
    version: bool,
    #[arg(long = "verbose")]
    loud: bool,
}

fn main() {}
