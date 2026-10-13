use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Inner {
    #[arg(short, long)]
    verbose: bool,
}

#[derive(GetArgs)]
struct Cli {
    #[command(flatten)]
    #[arg(long)]
    inner: Inner,
    #[command(flatten = true)]
    other: Inner,
    #[command(subcommand)]
    third: Inner,
}

fn main() {}
