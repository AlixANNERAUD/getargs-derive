use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Inner {
    #[arg(short, long)]
    verbose: bool,
}

#[derive(GetArgs)]
struct Cli {
    #[command(flatten)]
    inner: Option<Inner>,
}

fn main() {}
