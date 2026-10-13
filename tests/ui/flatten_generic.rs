use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Inner {
    #[arg(short, long)]
    verbose: bool,
}

#[derive(GetArgs)]
struct Cli<T: core::str::FromStr> {
    #[arg(long)]
    value: T,
    #[command(flatten)]
    inner: Inner,
}

fn main() {}
