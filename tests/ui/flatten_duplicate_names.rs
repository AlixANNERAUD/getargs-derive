use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Logging {
    #[arg(short, long)]
    verbose: bool,
}

#[derive(GetArgs)]
#[command(disable_help_flag)]
struct Help {
    #[arg(short, long)]
    help: bool,
}

#[derive(GetArgs)]
struct Output<'a> {
    #[arg(long)]
    verbose: Option<&'a str>,
}

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(short, long)]
    verbose: bool,
    #[command(flatten)]
    logging: Logging,
    #[command(flatten)]
    help: Help,
    #[command(flatten)]
    output: Output<'a>,
}

fn main() {}
