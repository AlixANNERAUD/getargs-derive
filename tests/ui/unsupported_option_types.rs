use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(short, long)]
    verbose: Option<bool>,
}

#[derive(GetArgs)]
struct Positional {
    verbose: Option<bool>,
}

#[derive(GetArgs)]
struct Nested {
    #[arg(long)]
    level: Option<Option<u8>>,
}

fn main() {}
