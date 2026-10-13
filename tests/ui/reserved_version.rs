use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(version)]
struct Cli<'a> {
    #[arg(short = 'V', long = "verbose")]
    verbose: bool,
    #[arg(long)]
    version: &'a str,
}

fn main() {}
