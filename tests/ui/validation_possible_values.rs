use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(long, possible_values)]
    missing: &'a str,
    #[arg(long, possible_values = "fast")]
    single: &'a str,
    #[arg(long, possible_values = [])]
    empty: &'a str,
    #[arg(long, possible_values = ["fast", 1, ""])]
    elements: &'a str,
    #[arg(long, possible_values = ["fast", "slow", "fast"])]
    repeated: &'a str,
    #[arg(long, possible_values = ["fast", "slow"], default_value = "turbo")]
    default: &'a str,
    #[arg(long, possible_values = ["on", "off"], action = SetFalse)]
    color: bool,
}

fn main() {}
