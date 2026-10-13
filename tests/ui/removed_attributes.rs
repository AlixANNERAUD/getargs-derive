use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Flag {
    #[arg(long, flag)]
    verbose: bool,
}

#[derive(GetArgs)]
struct ArgDefault {
    #[arg(long, default = 1)]
    jobs: u32,
}

#[derive(GetArgs)]
struct DefaultAttribute {
    #[arg(long)]
    #[default(1)]
    jobs: u32,
}

fn main() {}
