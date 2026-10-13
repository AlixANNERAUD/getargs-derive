use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct NotString {
    #[arg(long, default_value = 4)]
    jobs: u32,
}

#[derive(GetArgs)]
struct MissingValue {
    #[arg(long, default_value_t)]
    jobs: u32,
}

fn main() {}
