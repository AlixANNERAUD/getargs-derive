use getargs_derive::GetArgs;

struct Inner {
    verbose: bool,
}

#[derive(GetArgs)]
struct Cli {
    #[command(flatten)]
    inner: Inner,
}

fn main() {}
