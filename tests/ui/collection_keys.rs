use getargs_derive::{GetArgs, Values};

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(long, value_delimiter = ',')]
    single: &'a str,
    #[arg(num_args = 1..)]
    positional: u8,
    #[arg(long, default_values = ["a"])]
    plain: &'a str,
    #[arg(long, num_args = 1..)]
    option: Values<&'a str, 2>,
    #[arg(long, default_value_t = Values::new())]
    typed: Values<u8, 2>,
    #[arg(long, default_value = "a", default_values = ["b"])]
    both: Values<&'a str, 2>,
    #[arg(long, possible_values = ["a", "b"], value_delimiter = ',', default_value = "a,c")]
    checked: Values<&'a str, 2>,
}

fn main() {}
