use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Inner<'a> {
    #[arg(default_value = "out")]
    output: &'a str,
}

#[derive(GetArgs)]
struct Cli<'a> {
    #[command(flatten)]
    inner: Inner<'a>,
    input: &'a str,
}

fn main() {}
