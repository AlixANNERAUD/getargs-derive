use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(short, action = Count, value_name = "LEVEL")]
    verbose: u8,
    #[arg(long, action = Count, action = Count)]
    quiet: u8,
}

fn main() {}
