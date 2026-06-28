#![doc = include_str!("../README.md")]
#![no_std]
#![deny(missing_docs)]

#[doc(hidden)]
extern crate self as getargs_derive;

mod error;

pub use error::*;

pub use getargs_derive_macros::GetArgs;

#[cfg(test)]
mod tests {
    #![allow(dead_code, missing_docs)]
    extern crate alloc;
    use super::*;
    use core::num::NonZeroU32;

    // ── struct definitions ──────────────────────────────────────────

    #[derive(GetArgs)]
    struct BasicPositionalFlag<'a> {
        positional_field: &'a str,
        #[arg(flag)]
        bool_field: bool,
    }

    #[derive(GetArgs)]
    struct TwoPositionals<'a> {
        first: &'a str,
        second: &'a str,
    }

    #[derive(GetArgs)]
    struct CustomShort<'a> {
        #[arg(short = 'n', default = "default")]
        name: &'a str,
    }

    #[derive(GetArgs)]
    struct CustomLong {
        #[arg(long = "my-flag", flag)]
        my_flag: bool,
    }

    #[derive(GetArgs)]
    struct WithDefault<'a> {
        #[arg(default = "fallback")]
        name: &'a str,
    }

    #[derive(GetArgs)]
    struct WithLegacyDefault {
        #[default(42)]
        value: u32,
    }

    #[derive(GetArgs)]
    struct WithParseable {
        #[arg(short = 'j', default = 1)]
        jobs: u32,
    }

    #[derive(GetArgs)]
    struct WithNonZero {
        #[arg(short = 'p')]
        port: NonZeroU32,
    }

    #[derive(Debug, GetArgs)]
    struct RequiredOption<'a> {
        name: &'a str,
    }

    #[derive(Debug, GetArgs)]
    struct OnePositional<'a> {
        first: &'a str,
    }

    #[derive(Debug, GetArgs)]
    struct ParseInt {
        #[arg(short = 'n')]
        num: u32,
    }

    #[derive(Debug, GetArgs)]
    struct NoOptions<'a> {
        input: &'a str,
    }

    #[derive(GetArgs)]
    struct MultipleFlags {
        #[arg(flag)]
        verbose: bool,
        #[arg(flag)]
        debug: bool,
    }

    #[derive(GetArgs)]
    struct TwoPosNoOptions<'a> {
        a: &'a str,
        b: &'a str,
    }

    #[derive(Debug, GetArgs)]
    struct ExplicitPositional<'a> {
        #[arg(positional)]
        name: &'a str,
    }

    #[derive(GetArgs)]
    struct Mixed<'a> {
        input: &'a str,
        #[arg(flag)]
        verbose: bool,
        #[arg(short = 'o', default = "out")]
        output: &'a str,
    }

    // ── happy-path tests ────────────────────────────────────────────

    #[test]
    fn positional_and_flag() {
        let args = ["value1", "--bool-field"];
        let mut options = getargs::Options::new(args.into_iter());
        let params = BasicPositionalFlag::parse(&mut options).unwrap();
        assert_eq!(params.positional_field, "value1");
        assert!(params.bool_field);
    }

    #[test]
    fn two_positionals() {
        let args = ["hello", "world"];
        let mut options = getargs::Options::new(args.into_iter());
        let params = TwoPositionals::parse(&mut options).unwrap();
        assert_eq!(params.first, "hello");
        assert_eq!(params.second, "world");
    }

    #[test]
    fn custom_short() {
        let args = ["-n", "alice"];
        let mut options = getargs::Options::new(args.into_iter());
        let params = CustomShort::parse(&mut options).unwrap();
        assert_eq!(params.name, "alice");
    }

    #[test]
    fn custom_long() {
        let args = ["--my-flag"];
        let mut options = getargs::Options::new(args.into_iter());
        let params = CustomLong::parse(&mut options).unwrap();
        assert!(params.my_flag);
    }

    #[test]
    fn default_value_fallback() {
        let args: [&str; 0] = [];
        let mut options = getargs::Options::new(args.into_iter());
        let params = WithDefault::parse(&mut options).unwrap();
        assert_eq!(params.name, "fallback");
    }

    #[test]
    fn legacy_default() {
        let args: [&str; 0] = [];
        let mut options = getargs::Options::new(args.into_iter());
        let params = WithLegacyDefault::parse(&mut options).unwrap();
        assert_eq!(params.value, 42);
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
    fn all_positionals() {
        let args = ["x", "y"];
        let mut options = getargs::Options::new(args.into_iter());
        let params = TwoPosNoOptions::parse(&mut options).unwrap();
        assert_eq!(params.a, "x");
        assert_eq!(params.b, "y");
    }

    #[test]
    fn explicit_positional() {
        let args = ["hello"];
        let mut options = getargs::Options::new(args.into_iter());
        let params = ExplicitPositional::parse(&mut options).unwrap();
        assert_eq!(params.name, "hello");
    }

    #[test]
    fn mixed_positionals_and_options() {
        let args = ["in.txt", "--verbose"];
        let mut options = getargs::Options::new(args.into_iter());
        let params = Mixed::parse(&mut options).unwrap();
        assert_eq!(params.input, "in.txt");
        assert!(params.verbose);
        assert_eq!(params.output, "out");
    }

    #[test]
    fn options_can_appear_before_positionals() {
        let args = ["--verbose", "in.txt"];
        let mut options = getargs::Options::new(args.into_iter());
        let params = Mixed::parse(&mut options).unwrap();
        assert_eq!(params.input, "in.txt");
        assert!(params.verbose);
    }

    // ── error-path tests ────────────────────────────────────────────

    #[test]
    fn missing_required_positional() {
        let args: [&str; 0] = [];
        let mut options = getargs::Options::new(args.into_iter());
        let err = OnePositional::parse(&mut options).unwrap_err();
        assert_eq!(err, Error::MissingPositionalArgument("first"));
    }

    #[test]
    fn missing_required_option() {
        let args: [&str; 0] = [];
        let mut options = getargs::Options::new(args.into_iter());
        let err = RequiredOption::parse(&mut options).unwrap_err();
        assert_eq!(err, Error::MissingPositionalArgument("name"));
    }

    #[test]
    fn too_many_positionals() {
        let args = ["one", "two"];
        let mut options = getargs::Options::new(args.into_iter());
        let err = OnePositional::parse(&mut options).unwrap_err();
        assert_eq!(err, Error::InvalidNumberOfArguments);
    }

    #[test]
    fn invalid_option_parse_failure() {
        let args = ["-n", "not-a-number"];
        let mut options = getargs::Options::new(args.into_iter());
        let err = ParseInt::parse(&mut options).unwrap_err();
        assert_eq!(
            err,
            Error::InvalidOption("failed to parse value for option 'num'")
        );
    }

    #[test]
    fn unknown_flag() {
        let args = ["input", "--bogus"];
        let mut options = getargs::Options::new(args.into_iter());
        let err = NoOptions::parse(&mut options).unwrap_err();
        assert_eq!(err, Error::InvalidOption("unknown option"));
    }

    #[test]
    fn unknown_short_flag() {
        let args = ["input", "-z"];
        let mut options = getargs::Options::new(args.into_iter());
        let err = NoOptions::parse(&mut options).unwrap_err();
        assert_eq!(err, Error::InvalidOption("unknown option"));
    }

    #[test]
    fn extra_positional_with_explicit_marker() {
        let args = ["hello", "world"];
        let mut options = getargs::Options::new(args.into_iter());
        let err = ExplicitPositional::parse(&mut options).unwrap_err();
        assert_eq!(err, Error::InvalidNumberOfArguments);
    }

    #[test]
    fn too_many_positionals_when_no_positionals_defined() {
        #[derive(Debug, GetArgs)]
        struct FlagsOnly {
            #[arg(flag)]
            verbose: bool,
        }

        let args = ["extra"];
        let mut options = getargs::Options::new(args.into_iter());
        let err = FlagsOnly::parse(&mut options).unwrap_err();
        assert_eq!(err, Error::InvalidNumberOfArguments);
    }

    // ── Display impl tests ──────────────────────────────────────────

    #[test]
    fn error_display_missing_positional() {
        assert_eq!(
            alloc::format!("{}", Error::MissingPositionalArgument("output")),
            "Missing required argument: 'output'"
        );
    }

    #[test]
    fn error_display_invalid_option() {
        assert_eq!(
            alloc::format!("{}", Error::InvalidOption("unknown option")),
            "unknown option"
        );
    }

    #[test]
    fn error_display_invalid_number() {
        assert_eq!(
            alloc::format!("{}", Error::InvalidNumberOfArguments),
            "Invalid number of arguments provided"
        );
    }

    #[test]
    fn error_debug_and_clone_and_eq() {
        let a = Error::MissingPositionalArgument("x");
        let b = Error::MissingPositionalArgument("x");
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }
}
