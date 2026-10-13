use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    input: Box<[u8]>,
}

fn main() {}
