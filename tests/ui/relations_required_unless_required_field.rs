use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(long, required_unless_present = "stdin")]
    config: &'a str,
    #[arg(long)]
    stdin: bool,
}

fn main() {}
