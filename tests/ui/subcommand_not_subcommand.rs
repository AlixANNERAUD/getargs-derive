use getargs_derive::{GetArgs, Subcommand};

struct Plain;

#[derive(GetArgs)]
struct Cli {
    #[command(subcommand)]
    command: Plain,
}

#[derive(Subcommand)]
enum Commands {
    Run(Plain),
}

fn main() {}
