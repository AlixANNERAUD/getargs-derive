use getargs_derive::{GetArgs, ValueEnum};

#[derive(Clone, Copy, ValueEnum)]
enum Mode {
    Fast,
    Slow,
}

fn parse_mode(_: &str) -> Result<Mode, ()> {
    Ok(Mode::Fast)
}

#[derive(GetArgs)]
struct Cli {
    #[arg(long, value_enum, possible_values = ["fast"])]
    first: Mode,
    #[arg(long, value_enum, value_parser = parse_mode)]
    second: Mode,
}

fn main() {}
