use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, action = Append)]
    first: u8,
    #[arg(long, action = clap::ArgAction::Count)]
    second: u8,
    #[arg(long, action)]
    third: u8,
}

fn main() {}
