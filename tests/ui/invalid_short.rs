use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(short = '-')]
    dash: bool,
    #[arg(short = ' ')]
    space: bool,
    #[arg(short = "v")]
    verbose: bool,
}

fn main() {}
