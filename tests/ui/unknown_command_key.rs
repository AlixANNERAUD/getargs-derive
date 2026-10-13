use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(name = "tool", long_about = "More")]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}

fn main() {}
