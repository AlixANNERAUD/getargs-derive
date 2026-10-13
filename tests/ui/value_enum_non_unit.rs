use getargs_derive::ValueEnum;

#[derive(ValueEnum)]
enum Mode {
    Fast,
    Slow(u8),
    Custom { speed: u8 },
}

fn main() {}
