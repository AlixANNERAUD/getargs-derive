use getargs_derive::{GetArgs, Values};

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(long)]
    array: [u8; 2],
    #[arg(long)]
    optional: Option<Values<&'a str, 2>>,
    #[arg(long)]
    nested: Values<Option<u8>, 2>,
}

fn main() {}
