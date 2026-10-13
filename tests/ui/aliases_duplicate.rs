use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(short, long)]
    verbose: bool,
    #[arg(long, alias = "verbose", short_alias = 'v')]
    loud: bool,
    #[arg(long, aliases = ["out", "out"], visible_short_aliases = ['o', 'o'])]
    output: &'a str,
    #[arg(long, visible_alias = "dest", short_alias = 'o')]
    destination: &'a str,
}

fn main() {}
