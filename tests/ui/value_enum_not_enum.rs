use getargs_derive::ValueEnum;

#[derive(ValueEnum)]
struct Mode {
    fast: bool,
}

#[derive(ValueEnum)]
union Level {
    low: u8,
}

fn main() {}
