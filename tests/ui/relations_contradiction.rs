use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, conflicts_with = "verbose", requires = "verbose")]
    quiet: bool,
    #[arg(long)]
    verbose: bool,
    #[arg(long, group = "mode")]
    fast: bool,
    #[arg(long, group = "mode", requires = "fast")]
    slow: bool,
}

fn main() {}
