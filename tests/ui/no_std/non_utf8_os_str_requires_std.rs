extern crate std;

use getargs_derive::GetArgs;
use std::ffi::OsStr;
use std::path::Path;

#[derive(GetArgs)]
struct Cli<'a> {
    input: &'a OsStr,
    #[arg(long)]
    output: Option<&'a Path>,
}

fn main() {}
