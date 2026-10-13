use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(default_value = "in.txt")]
    input: &'a str,
    output: &'a str,
}

fn main() {}
