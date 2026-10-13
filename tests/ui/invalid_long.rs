use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long = "")]
    empty: bool,
    #[arg(long = "--verbose")]
    verbose: bool,
    #[arg(long = "dry run")]
    dry_run: bool,
    #[arg(long = "key=value")]
    key: bool,
    #[arg(long = 'q')]
    quiet: bool,
}

fn main() {}
