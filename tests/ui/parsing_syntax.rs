use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(allow_negative_numbers = 1)]
struct Cli<'a> {
    #[arg(short, allow_hyphen_values = "yes")]
    pattern: Option<&'a str>,
    #[arg(last(true))]
    rest: Option<&'a str>,
    #[arg(short, allow_negative_numbers, allow_negative_numbers = false)]
    offset: Option<i32>,
}

fn main() {}
