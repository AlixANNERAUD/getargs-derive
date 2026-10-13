use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, value_enum)]
    port: u16,
}

fn main() {}
