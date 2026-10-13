use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(display_order = 1)]
    input: &'a str,
    #[arg(long, display_order = "1")]
    first: bool,
    #[arg(long, display_order = -1)]
    second: bool,
    #[arg(long, help_heading = " ")]
    third: bool,
    #[arg(long, hide = true)]
    fourth: bool,
    #[arg(long, long_help = 3)]
    fifth: bool,
    #[arg(long, hide, hide)]
    sixth: bool,
}

fn main() {}
