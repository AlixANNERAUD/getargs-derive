use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Server {
    #[arg(long, env = "PORT")]
    port: u16,
}

#[derive(GetArgs)]
struct Cli {
    #[command(flatten)]
    server: Server,
}

fn main() {
    let _ = Cli::from_args_with_env(["--port", "80"].into_iter(), |_| None);
    let _ = Cli::from_args(["--port", "80"].into_iter());
}
