use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    input: &'a [u8],
}

fn main() {
    let args: [&[u8]; 1] = [b"input"];
    let _ = Cli::from_args(args.into_iter());
}
