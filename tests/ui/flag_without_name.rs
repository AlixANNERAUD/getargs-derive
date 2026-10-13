use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(flag)]
    verbose: bool,
}

#[derive(GetArgs)]
struct WithValue {
    #[arg(long, flag = true)]
    verbose: bool,
}

fn main() {}
