use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(name = "tool", version = "1.0")]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}

fn main() {}
