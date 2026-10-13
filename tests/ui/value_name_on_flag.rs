use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(short, long, value_name = "LEVEL")]
    verbose: bool,
}

fn main() {}
