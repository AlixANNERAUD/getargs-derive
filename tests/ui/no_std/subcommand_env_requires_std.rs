use getargs_derive::{GetArgs, Subcommand};

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(long, env = "PORT")]
        port: u16,
    },
}

#[derive(GetArgs)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let _ = Cli::from_args_with_env(["serve"].into_iter(), |_| None);
    let _ = Cli::from_args(["serve"].into_iter());
}
