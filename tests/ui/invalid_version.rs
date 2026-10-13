use getargs_derive::GetArgs;

#[derive(GetArgs)]
#[command(version = "")]
struct Empty {}

#[derive(GetArgs)]
#[command(version = 1)]
struct NotString {}

#[derive(GetArgs)]
#[command(version, version = "1.0")]
struct Duplicate {}

fn main() {}
