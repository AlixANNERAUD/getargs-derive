use getargs_derive::Subcommand;

#[derive(Subcommand)]
enum Duplicates {
    Build,
    #[command(alias = "build")]
    Test,
    #[command(name = "test")]
    Check,
}

#[derive(Subcommand)]
enum Invalid {
    #[command(name = "-run")]
    Run,
    #[command(visible_alias = "two words")]
    Walk,
    #[command(aliases = ["", "ok"])]
    Fly,
}

fn main() {}
