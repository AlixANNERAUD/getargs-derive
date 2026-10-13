use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, env = "PORT")]
    port: u16,
}

fn main() {
    let _ = Cli::from_args(["--port", "80"].into_iter());
    let _ = Cli::from_args_with_env(["--port", "80"].into_iter(), |_| None);
}
