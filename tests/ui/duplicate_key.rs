use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(short, short = 'x')]
    extract: bool,
}

#[derive(GetArgs)]
struct Split {
    #[arg(long)]
    #[arg(long = "verbose")]
    verbose: bool,
}

fn main() {}
