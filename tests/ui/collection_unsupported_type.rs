use getargs_derive::GetArgs;

struct Values<T>(T);

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(long)]
    files: Values<&'a str>,
}

fn main() {}
