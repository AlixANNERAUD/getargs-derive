use core::num::NonZeroU32;
use getargs_derive::{Error, GetArgs};

#[derive(GetArgs)]
struct CustomShort<'a> {
    #[arg(short = 'n', default_value = "default")]
    name: &'a str,
}

#[derive(GetArgs)]
struct WithParseable {
    #[arg(short = 'j', default_value_t = 1)]
    jobs: u32,
}

#[derive(Debug, GetArgs)]
struct WithNonZero {
    #[arg(short = 'p')]
    port: NonZeroU32,
}

#[derive(Debug, GetArgs)]
struct InferredNames {
    #[arg(short, long)]
    verbose: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(short, default_value_t = 1)]
    jobs: u32,
}

#[derive(Debug, GetArgs)]
struct ExplicitNames<'a> {
    #[arg(short = 'x', long = "extract")]
    archive: bool,
    #[arg(short = 'C', long = "directory", default_value = ".")]
    dir: &'a str,
}

#[derive(Debug, GetArgs)]
struct RawIdentifier<'a> {
    #[arg(long, default_value = "file")]
    r#type: &'a str,
}

#[test]
fn custom_short() {
    let args = ["-n", "alice"];
    let mut options = getargs::Options::new(args.into_iter());
    let params = CustomShort::parse(&mut options).unwrap();
    assert_eq!(params.name, "alice");
}

#[test]
fn parseable_type() {
    let args = ["-j", "8"];
    let mut options = getargs::Options::new(args.into_iter());
    let params = WithParseable::parse(&mut options).unwrap();
    assert_eq!(params.jobs, 8);
}

#[test]
fn nonzero_type() {
    let args = ["-p", "8080"];
    let mut options = getargs::Options::new(args.into_iter());
    let params = WithNonZero::parse(&mut options).unwrap();
    assert_eq!(params.port.get(), 8080);
}

#[test]
fn nonzero_without_default_is_required() {
    let err = WithNonZero::from_args([].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingPositionalArgument("port"));
}

#[test]
fn inferred_short_and_long() {
    let args = ["-v", "--dry-run", "-j", "4"];
    let params = InferredNames::from_args(args.into_iter()).unwrap();
    assert!(params.verbose);
    assert!(params.dry_run);
    assert_eq!(params.jobs, 4);

    let params = InferredNames::from_args(["--verbose"].into_iter()).unwrap();
    assert!(params.verbose);
    assert!(!params.dry_run);
    assert_eq!(params.jobs, 1);
}

#[test]
fn short_only_rejects_long() {
    let err = InferredNames::from_args(["--jobs", "4"].into_iter()).unwrap_err();
    assert_eq!(err, Error::UnknownOption);
}

#[test]
fn long_only_rejects_short() {
    let err = InferredNames::from_args(["-d"].into_iter()).unwrap_err();
    assert_eq!(err, Error::UnknownOption);
}

#[test]
fn explicit_short_and_long() {
    let params = ExplicitNames::from_args(["-x", "-C", "/tmp"].into_iter()).unwrap();
    assert!(params.archive);
    assert_eq!(params.dir, "/tmp");

    let args = ["--extract", "--directory=/srv"];
    let params = ExplicitNames::from_args(args.into_iter()).unwrap();
    assert!(params.archive);
    assert_eq!(params.dir, "/srv");

    let params = ExplicitNames::from_args([].into_iter()).unwrap();
    assert!(!params.archive);
    assert_eq!(params.dir, ".");
}

#[test]
fn explicit_names_replace_field_name() {
    let err = ExplicitNames::from_args(["--archive"].into_iter()).unwrap_err();
    assert_eq!(err, Error::UnknownOption);
}

#[test]
fn raw_identifier_long_name() {
    let params = RawIdentifier::from_args(["--type", "dir"].into_iter()).unwrap();
    assert_eq!(params.r#type, "dir");
}

#[test]
fn missing_option_value() {
    let err = ExplicitNames::from_args(["--directory"].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOptionValue("directory"));
}
