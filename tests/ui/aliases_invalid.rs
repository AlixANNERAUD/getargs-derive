use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, alias = 'x')]
    first: bool,
    #[arg(long, alias = "--second")]
    second: bool,
    #[arg(long, aliases = ["ok", "--third"])]
    third: bool,
    #[arg(long, aliases = "fourth")]
    fourth: bool,
    #[arg(long, short_alias = "f")]
    fifth: bool,
    #[arg(long, short_aliases = ['-'])]
    sixth: bool,
    #[arg(long, visible_aliases = [""])]
    seventh: bool,
    #[arg(long, visible_short_alias)]
    eighth: bool,
}

fn main() {}
