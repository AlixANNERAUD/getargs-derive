use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, trailing_var_arg = true)]
    verbose: bool,
}

fn main() {}
