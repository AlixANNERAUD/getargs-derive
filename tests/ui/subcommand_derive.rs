use getargs_derive::Subcommand;

#[derive(Subcommand)]
struct NotAnEnum {
    value: u32,
}

#[derive(Subcommand)]
enum Empty {}

#[derive(Subcommand)]
enum Generic<T> {
    Value(T),
}

#[derive(Subcommand)]
#[command(name = "tool")]
enum EnumAttribute {
    Run,
}

#[derive(Subcommand)]
enum Tuple {
    Pair(u32, u32),
}

fn main() {}
