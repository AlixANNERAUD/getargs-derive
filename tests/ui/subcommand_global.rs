use getargs_derive::{GetArgs, Subcommand};

#[derive(Subcommand)]
enum Commands {
    Run {
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(GetArgs)]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,
    #[arg(long = "help-me", short = 'x', global)]
    help_me: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(GetArgs)]
struct Inner {
    #[command(subcommand)]
    command: Commands,
}

#[derive(GetArgs)]
struct Outer {
    #[command(flatten)]
    inner: Inner,
}

fn main() {}
