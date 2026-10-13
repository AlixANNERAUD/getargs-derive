use getargs_derive::{GetArgs, Values};

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(long, value_delimiter = ",")]
    delimiter: Values<&'a str, 2>,
    #[arg(long, default_values = [])]
    empty: Values<&'a str, 2>,
    #[arg(long, default_values = "a")]
    string: Values<&'a str, 2>,
    #[arg(num_args = 0)]
    zero: Values<&'a str, 2>,
}

#[derive(GetArgs)]
struct Ranges<'a> {
    #[arg(num_args = 3..2)]
    reversed: Values<&'a str, 2>,
}

#[derive(GetArgs)]
struct Bounds<'a> {
    #[arg(num_args = 1..N)]
    named: Values<&'a str, 2>,
}

const N: usize = 2;

fn main() {}
