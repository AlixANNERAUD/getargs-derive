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
    #[command(next_help_heading = "Other")]
    third: Inner,
}

fn main() {}
