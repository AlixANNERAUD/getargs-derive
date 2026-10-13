use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, action = SetFalse)]
    color: u8,
    #[arg(long, action = SetTrue)]
    fast: u32,
}

fn main() {}
