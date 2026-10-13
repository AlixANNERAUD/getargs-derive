use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(max_term_width = 0)]
#[command(max_term_width = "80")]
#[command(long_about = 1)]
#[command(before_help)]
#[command(after_help = "One", after_help = "Two")]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}

fn main() {}
