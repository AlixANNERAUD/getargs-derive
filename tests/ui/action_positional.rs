use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(action = Count)]
    verbose: u8,
    #[arg(action = SetFalse)]
    color: bool,
}

fn main() {}
