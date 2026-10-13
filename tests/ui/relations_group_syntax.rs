use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(group(required))]
#[command(group(id = "a", exclusive))]
#[command(group = "b")]
#[command(group(id = "c", multiple = true))]
struct Cli {
    #[arg(long, group = "a")]
    #[arg(group = "c")]
    fast: bool,
    #[arg(long)]
    #[arg(conflicts_with = "")]
    #[arg(requires = 1)]
    slow: bool,
}

fn main() {}
