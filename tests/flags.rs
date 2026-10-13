use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct CustomLong {
    #[arg(long = "my-flag")]
    my_flag: bool,
}

#[derive(GetArgs)]
struct MultipleFlags {
    #[arg(long)]
    verbose: bool,
    #[arg(long)]
    debug: bool,
}

#[derive(Debug, GetArgs)]
struct FlagWithDefault {
    #[arg(long, default_value_t = true)]
    color: bool,
}

#[test]
fn custom_long() {
    let args = ["--my-flag"];
    let mut options = getargs::Options::new(args.into_iter());
    let params = CustomLong::parse(&mut options).unwrap();
    assert!(params.my_flag);
}

#[test]
fn multiple_flags() {
    let args = ["--verbose", "--debug"];
    let mut options = getargs::Options::new(args.into_iter());
    let params = MultipleFlags::parse(&mut options).unwrap();
    assert!(params.verbose);
    assert!(params.debug);
}

#[test]
fn flags_default_to_false() {
    let args: [&str; 0] = [];
    let mut options = getargs::Options::new(args.into_iter());
    let params = MultipleFlags::parse(&mut options).unwrap();
    assert!(!params.verbose);
    assert!(!params.debug);
}

#[test]
fn flag_with_default() {
    let params = FlagWithDefault::from_args([].into_iter()).unwrap();
    assert!(params.color);
}
