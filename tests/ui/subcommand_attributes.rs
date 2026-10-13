use getargs_derive::{GetArgs, Subcommand};

#[derive(GetArgs)]
struct Build {
    #[arg(short, long)]
    release: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(version)]
    Run,
    #[command(before_help = "Read this")]
    Build(Build),
    Test(#[arg(short)] Build),
    #[command(hide = true)]
    Check,
}

fn main() {}
