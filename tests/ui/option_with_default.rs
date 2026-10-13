use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(long, default_value = "out.txt")]
    output: Option<&'a str>,
}

#[derive(GetArgs)]
struct Typed {
    #[arg(short, default_value_t = 4)]
    jobs: Option<u32>,
}

#[derive(GetArgs)]
struct Legacy {
    #[default(4)]
    jobs: Option<u32>,
}

fn main() {}
