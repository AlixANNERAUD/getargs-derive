use getargs_derive::ValueEnum;

#[derive(ValueEnum)]
#[value(name = "mode")]
enum Mode {
    #[value(rename = "quick")]
    Fast,
    #[value(name = "slow", name = "slower")]
    Slow,
    #[value(name = "")]
    Empty,
    #[value(alias)]
    Bare,
    #[value]
    Path,
}

#[derive(ValueEnum)]
enum Clash {
    #[value(alias = "fast")]
    Quick,
    Fast,
    #[value(name = "quick")]
    Other,
}

#[derive(ValueEnum)]
enum Generic<T> {
    Value,
    Marker(T),
}

#[derive(ValueEnum)]
enum Empty {}

fn main() {}
