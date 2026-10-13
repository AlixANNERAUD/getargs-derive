#![allow(dead_code)]

use getargs_derive::{Error, GetArgs};

#[derive(Debug, GetArgs)]
#[command(name = "copy", version)]
struct VersionPackage<'a> {
    /// Input file
    input: &'a str,
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, GetArgs)]
#[command(version = "2.1.0-beta")]
struct VersionLiteral {}

#[derive(Debug, GetArgs)]
#[command(name = "nohelp", version, disable_help_flag)]
struct VersionWithoutHelp {}

#[derive(Debug, GetArgs)]
#[command(name = "plain")]
struct VersionDisabled<'a> {
    #[arg(short = 'V', long)]
    version: &'a str,
}

#[test]
fn version_package_constants() {
    assert_eq!(VersionPackage::VERSION, env!("CARGO_PKG_VERSION"));
    assert_eq!(
        VersionPackage::VERSION_TEXT,
        concat!("copy ", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn version_literal_constants() {
    assert_eq!(VersionLiteral::VERSION, "2.1.0-beta");
    assert_eq!(VersionLiteral::VERSION_TEXT, "getargs-derive 2.1.0-beta");
}

#[test]
fn version_help_snapshot() {
    assert_eq!(
        VersionPackage::HELP,
        "Usage: copy [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input file

Options:
  -v, --verbose
  -h, --help     Print help
  -V, --version  Print version"
    );
    assert_eq!(
        VersionWithoutHelp::HELP,
        "Usage: nohelp [OPTIONS]\n\nOptions:\n  -V, --version  Print version"
    );
}

#[test]
fn version_long_and_short_flags() {
    for flag in ["-V", "--version"] {
        let err = VersionLiteral::from_args([flag].into_iter()).unwrap_err();
        assert_eq!(err, Error::Version(VersionLiteral::VERSION_TEXT));
        assert!(err.is_version());
        assert!(!err.is_help());
    }
}

#[test]
fn version_takes_priority_over_missing_required() {
    let err = VersionPackage::from_args(["-v", "--version"].into_iter()).unwrap_err();
    assert_eq!(err, Error::Version(VersionPackage::VERSION_TEXT));
}

#[test]
fn version_and_help_first_one_wins() {
    let err = VersionPackage::from_args(["--help", "-V"].into_iter()).unwrap_err();
    assert!(err.is_help());
    let err = VersionPackage::from_args(["-V", "--help"].into_iter()).unwrap_err();
    assert!(err.is_version());
}

#[test]
fn version_without_help_flag() {
    let err = VersionWithoutHelp::from_args(["--version"].into_iter()).unwrap_err();
    assert!(err.is_version());
    let err = VersionWithoutHelp::from_args(["--help"].into_iter()).unwrap_err();
    assert_eq!(err, Error::UnknownOption);
}

#[test]
fn version_after_double_dash_is_positional() {
    let params = VersionPackage::from_args(["--", "--version"].into_iter()).unwrap();
    assert_eq!(params.input, "--version");
}

#[test]
fn version_disabled_allows_version_names() {
    let params = VersionDisabled::from_args(["-V", "1.0"].into_iter()).unwrap();
    assert_eq!(params.version, "1.0");
    let params = VersionDisabled::from_args(["--version", "2.0"].into_iter()).unwrap();
    assert_eq!(params.version, "2.0");
    assert!(!VersionDisabled::HELP.contains("Print version"));
}

#[test]
fn error_display_version() {
    assert_eq!(
        format!("{}", Error::Version(VersionLiteral::VERSION_TEXT)),
        "getargs-derive 2.1.0-beta"
    );
}
