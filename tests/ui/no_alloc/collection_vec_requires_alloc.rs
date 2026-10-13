use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(long)]
    include: Vec<&'a str>,
}

fn main() {}
