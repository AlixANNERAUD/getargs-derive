use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(long, value_parser)]
    missing: u8,
    #[arg(long, value_parser = "1..=64")]
    literal: u8,
    #[arg(long, value_parser = ..)]
    unbounded: u8,
    #[arg(long, value_parser = 1..=64, value_parser = 1..=8)]
    duplicate: u8,
    #[arg(long, value_parser = 1..=64)]
    text: &'a str,
    #[arg(long, value_parser = 0..10)]
    ratio: Option<f64>,
    #[arg(long, value_parser = 1..=64)]
    verbose: bool,
}

fn main() {}
