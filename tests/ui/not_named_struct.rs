use getargs_derive::GetArgs;

#[derive(GetArgs)]
enum Mode {
    Fast,
    Slow,
}

#[derive(GetArgs)]
struct Tuple<'a>(&'a str);

#[derive(GetArgs)]
struct Unit;

fn main() {}
