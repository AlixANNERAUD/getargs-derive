use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(group(id = "mode"), group(id = "mode"), group(id = "unused", required))]
struct Cli {
    #[arg(long, group = "mode")]
    fast: bool,
    #[arg(long, group = "mode")]
    slow: bool,
}

fn main() {}
