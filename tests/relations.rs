#![allow(dead_code)]

use getargs_derive::{Error, GetArgs};

#[derive(Debug, GetArgs)]
struct Conflicts {
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,
    #[arg(short, long)]
    verbose: bool,
    #[arg(long, default_value_t = 1, conflicts_with = "quiet")]
    level: u8,
}

#[test]
fn conflicts_with_allows_either_argument() {
    let params = Conflicts::from_args(["-q"].into_iter()).unwrap();
    assert!(params.quiet);
    let params = Conflicts::from_args(["-v"].into_iter()).unwrap();
    assert!(params.verbose);
}

#[test]
fn conflicts_with_rejects_both_arguments() {
    let err = Conflicts::from_args(["-v", "--quiet"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::ArgumentConflict {
            name: "--quiet",
            other: "--verbose",
        }
    );
    assert_eq!(
        err.to_string(),
        "the argument '--quiet' cannot be used with '--verbose'"
    );
}

#[test]
fn conflicts_ignore_defaults() {
    let params = Conflicts::from_args(["--quiet"].into_iter()).unwrap();
    assert_eq!(params.level, 1);
    let err = Conflicts::from_args(["--quiet", "--level", "2"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::ArgumentConflict {
            name: "--level <LEVEL>",
            other: "--quiet",
        }
    );
}

#[derive(Debug, GetArgs)]
struct Requires<'a> {
    #[arg(short, long, requires = "output", requires = "level")]
    compress: bool,
    #[arg(short, long)]
    output: Option<&'a str>,
    #[arg(long, default_value_t = 6)]
    level: u8,
}

#[test]
fn requires_accepts_present_requirements() {
    let args = ["-c", "-o", "out.gz", "--level", "9"];
    let params = Requires::from_args(args.into_iter()).unwrap();
    assert!(params.compress);
    assert_eq!(params.output, Some("out.gz"));
    assert_eq!(params.level, 9);

    let params = Requires::from_args([].into_iter()).unwrap();
    assert!(!params.compress);
}

#[test]
fn requires_rejects_missing_requirement() {
    let err = Requires::from_args(["-c", "--level", "9"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::MissingRequiredArgument {
            name: "--output <OUTPUT>",
            required_by: "--compress",
        }
    );
    assert_eq!(
        err.to_string(),
        "the following required argument was not provided: --output <OUTPUT> (required by '--compress')"
    );
}

#[test]
fn requires_is_not_satisfied_by_a_default() {
    let err = Requires::from_args(["-c", "-o", "out.gz"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::MissingRequiredArgument {
            name: "--level <LEVEL>",
            required_by: "--compress",
        }
    );
}

#[derive(Debug, GetArgs)]
struct RequiredUnless<'a> {
    #[arg(short, long, required_unless_present = "stdin")]
    config: Option<&'a str>,
    #[arg(long)]
    stdin: bool,
    #[arg(required_unless_present = "stdin", required_unless_present = "config")]
    input: Option<&'a str>,
}

#[test]
fn required_unless_present_accepts_either() {
    let params = RequiredUnless::from_args(["-c", "app.toml", "in"].into_iter()).unwrap();
    assert_eq!(params.config, Some("app.toml"));
    assert_eq!(params.input, Some("in"));

    let params = RequiredUnless::from_args(["--stdin"].into_iter()).unwrap();
    assert_eq!(params.config, None);
    assert_eq!(params.input, None);
}

#[test]
fn required_unless_present_with_several_alternatives() {
    let params = RequiredUnless::from_args(["-c", "app.toml"].into_iter()).unwrap();
    assert_eq!(params.input, None);
}

#[test]
fn required_unless_present_rejects_none() {
    let err = RequiredUnless::from_args([].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOneOf(&["--config <CONFIG>", "--stdin"]));
    assert_eq!(
        err.to_string(),
        "the following required arguments were not provided: <--config <CONFIG>|--stdin>"
    );
}

#[derive(Debug, GetArgs)]
#[command(group(id = "mode", required))]
struct ExclusiveGroup {
    #[arg(long, group = "mode")]
    fast: bool,
    #[arg(long, group = "mode")]
    slow: bool,
    #[arg(long, group = "mode")]
    speed: Option<u32>,
}

#[test]
fn exclusive_group_accepts_one_member() {
    let params = ExclusiveGroup::from_args(["--slow"].into_iter()).unwrap();
    assert!(params.slow);
    let params = ExclusiveGroup::from_args(["--speed", "3"].into_iter()).unwrap();
    assert_eq!(params.speed, Some(3));
}

#[test]
fn exclusive_group_rejects_two_members() {
    let err = ExclusiveGroup::from_args(["--speed=3", "--fast"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::ArgumentConflict {
            name: "--fast",
            other: "--speed <SPEED>",
        }
    );
}

#[test]
fn required_group_rejects_no_member() {
    let err = ExclusiveGroup::from_args([].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::MissingOneOf(&["--fast", "--slow", "--speed <SPEED>"])
    );
    assert_eq!(
        err.to_string(),
        "the following required arguments were not provided: <--fast|--slow|--speed <SPEED>>"
    );
}

#[derive(Debug, GetArgs)]
#[command(group(id = "source", required, multiple))]
struct MultipleGroup<'a> {
    #[arg(long, group = "source")]
    file: Option<&'a str>,
    #[arg(long, group = "source")]
    url: Option<&'a str>,
}

#[test]
fn multiple_group_accepts_several_members() {
    let params = MultipleGroup::from_args(["--file", "a", "--url", "b"].into_iter()).unwrap();
    assert_eq!(params.file, Some("a"));
    assert_eq!(params.url, Some("b"));
}

#[test]
fn multiple_group_still_requires_one_member() {
    let err = MultipleGroup::from_args([].into_iter()).unwrap_err();
    assert_eq!(err, Error::MissingOneOf(&["--file <FILE>", "--url <URL>"]));
}

#[derive(Debug, GetArgs)]
struct ImplicitGroup {
    #[arg(short, group = "format")]
    json: bool,
    #[arg(short, group = "format")]
    yaml: bool,
}

#[test]
fn implicit_group_is_exclusive_and_optional() {
    let params = ImplicitGroup::from_args([].into_iter()).unwrap();
    assert!(!params.json && !params.yaml);
    let err = ImplicitGroup::from_args(["-jy"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::ArgumentConflict {
            name: "-j",
            other: "-y",
        }
    );
}

#[derive(Debug, GetArgs)]
struct Ordering {
    #[arg(long, conflicts_with = "b", requires = "c")]
    a: bool,
    #[arg(long)]
    b: bool,
    #[arg(long)]
    c: bool,
}

#[test]
fn conflicts_are_reported_before_requirements() {
    let err = Ordering::from_args(["--a", "--b"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::ArgumentConflict {
            name: "--a",
            other: "--b",
        }
    );
}
