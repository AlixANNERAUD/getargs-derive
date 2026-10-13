use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, default_value = "4", default_value_t = 4)]
    jobs: u32,
}

#[derive(GetArgs)]
struct Repeated {
    #[arg(long, default_value_t = 4, default_value_t = 5)]
    jobs: u32,
}

fn main() {}
