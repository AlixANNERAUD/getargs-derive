use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(short, long)]
    host: &'a str,
    #[arg(long = "help")]
    show_help: bool,
}

fn main() {}
