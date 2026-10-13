use getargs_derive::{GetArgs, Subcommand};

#[derive(Subcommand)]
enum Commands {
    Run,
}

#[derive(GetArgs)]
struct Twice {
    #[command(subcommand)]
    first: Commands,
    #[command(subcommand)]
    second: Commands,
}

#[derive(GetArgs)]
struct WithArg {
    #[arg(short)]
    #[command(subcommand)]
    command: Commands,
}

#[derive(GetArgs)]
struct WithFlatten {
    #[command(subcommand, flatten)]
    command: Commands,
}

#[derive(GetArgs)]
struct WithValue {
    #[command(subcommand = true)]
    command: Commands,
}

#[derive(GetArgs)]
struct Generic<T> {
    value: T,
    #[command(subcommand)]
    command: Commands,
}

fn main() {}
