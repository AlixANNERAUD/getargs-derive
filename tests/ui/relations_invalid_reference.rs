use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, conflicts_with = "missing")]
    quiet: bool,
    #[arg(long, requires = "verbose")]
    verbose: bool,
    #[arg(long, requires = "quiet", requires = "quiet")]
    debug: bool,
}

fn main() {}
