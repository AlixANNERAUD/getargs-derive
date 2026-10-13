use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(short, long, value_enum)]
    verbose: bool,
    #[arg(long, value_enum = true)]
    mode: u8,
    #[arg(long, value_enum, value_enum)]
    level: u8,
    #[arg(long, action = Count, value_enum)]
    count: u8,
}

fn main() {}
