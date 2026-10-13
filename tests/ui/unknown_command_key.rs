use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(name = "tool", disable_version_flag)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}

fn main() {}
