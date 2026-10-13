use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(short, long, allow_hyphen_values = true)]
    verbose: bool,
    #[arg(short, long, allow_negative_numbers = true)]
    quiet: bool,
    #[arg(allow_hyphen_values = true)]
    input: &'a str,
    #[arg(short, last = true)]
    output: Option<&'a str>,
}

fn main() {}
