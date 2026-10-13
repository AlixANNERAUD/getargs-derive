use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(version)]
struct Cli {
    #[arg(long, alias = "help", short_alias = 'h')]
    usage: bool,
    #[arg(long, visible_alias = "version", visible_short_alias = 'V')]
    show_version: bool,
}

fn main() {}
