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
        #[arg(long)]
        bool_field: bool,
    }

    #[derive(GetArgs)]
    struct TwoPositionals<'a> {
        first: &'a str,
        second: &'a str,
    }

    #[derive(GetArgs)]
    struct CustomShort<'a> {
        #[arg(short = 'n', default_value = "default")]
        name: &'a str,
    }

    #[derive(GetArgs)]
    struct CustomLong {
        #[arg(long = "my-flag")]
        my_flag: bool,
    }

    #[derive(GetArgs)]
    struct WithDefault<'a> {
        #[arg(long, default_value = "fallback")]
        name: &'a str,
    }

    #[derive(GetArgs)]
    struct WithLegacyDefault {
        #[arg(long)]
        #[default(42)]
        value: u32,
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
    struct RequiredOption<'a> {
        #[arg(long)]
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
        #[arg(long)]
        verbose: bool,
        #[arg(long)]
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
        #[arg(short, long)]
        verbose: bool,
        #[arg(short, long, default_value = "out")]
        output: &'a str,
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

    #[derive(Debug, GetArgs)]
    struct DefaultForms<'a> {
        #[arg(long, default_value = "8")]
        parsed: u32,
        #[arg(long, default_value_t = 3)]
        typed: u32,
        #[arg(long, default_value_t = "typed")]
        typed_str: &'a str,
        #[arg(long, default_value_t = NonZeroU32::new(5).unwrap())]
        non_zero: NonZeroU32,
    }

    #[derive(Debug, GetArgs)]
    struct InvalidDefaultValue {
        #[arg(long, default_value = "abc")]
        jobs: u32,
    }

    #[derive(Debug, GetArgs)]
    struct LegacyForms {
        #[arg(long, default = 5)]
        arg_default: u32,
        #[arg(long)]
        #[default = 7]
        name_value_default: u32,
        #[arg(long, flag)]
        legacy_flag: bool,
    }

    #[derive(Debug, GetArgs)]
    struct LegacyPositionalDefault {
        #[default(9)]
        count: u32,
    }

    #[derive(Debug, GetArgs)]
    struct OptionalPositionals<'a> {
        input: &'a str,
        #[arg(default_value = "out.txt")]
        output: &'a str,
        #[arg(default_value_t = 2)]
        level: u8,
    }

    #[derive(Debug, GetArgs)]
    struct FlagWithDefault {
        #[arg(long, default_value_t = true)]
        color: bool,
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

    #[test]
    fn from_args_convenience_method() {
        let args = ["value1", "--bool-field"];
        let params = BasicPositionalFlag::from_args(args.into_iter()).unwrap();
        assert_eq!(params.positional_field, "value1");
        assert!(params.bool_field);
    }

    #[test]
    fn from_args_empty_defaults() {
        let args: [&str; 0] = [];
        let params = WithDefault::from_args(args.into_iter()).unwrap();
        assert_eq!(params.name, "fallback");
    }

    #[test]
    fn from_args_parse_error() {
        let args = ["-n", "not-a-number"];
        let err = ParseInt::from_args(args.into_iter()).unwrap_err();
        assert_eq!(err, Error::ParseError("num"));
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
    fn default_value_and_default_value_t() {
        let params = DefaultForms::from_args([].into_iter()).unwrap();
        assert_eq!(params.parsed, 8);
        assert_eq!(params.typed, 3);
        assert_eq!(params.typed_str, "typed");
        assert_eq!(params.non_zero.get(), 5);

        let args = [
            "--parsed",
            "1",
            "--typed",
            "2",
            "--typed-str",
            "given",
            "--non-zero",
            "9",
        ];
        let params = DefaultForms::from_args(args.into_iter()).unwrap();
        assert_eq!(params.parsed, 1);
        assert_eq!(params.typed, 2);
        assert_eq!(params.typed_str, "given");
        assert_eq!(params.non_zero.get(), 9);
    }

    #[test]
    fn invalid_default_value_reports_parse_error() {
        let err = InvalidDefaultValue::from_args([].into_iter()).unwrap_err();
        assert_eq!(err, Error::ParseError("jobs"));
    }

    #[test]
    fn invalid_default_value_unused_when_provided() {
        let params = InvalidDefaultValue::from_args(["--jobs", "2"].into_iter()).unwrap();
        assert_eq!(params.jobs, 2);
    }

    #[test]
    fn legacy_forms() {
        let params = LegacyForms::from_args([].into_iter()).unwrap();
        assert_eq!(params.arg_default, 5);
        assert_eq!(params.name_value_default, 7);
        assert!(!params.legacy_flag);

        let args = [
            "--arg-default",
            "1",
            "--name-value-default",
            "2",
            "--legacy-flag",
        ];
        let params = LegacyForms::from_args(args.into_iter()).unwrap();
        assert_eq!(params.arg_default, 1);
        assert_eq!(params.name_value_default, 2);
        assert!(params.legacy_flag);
    }

    #[test]
    fn legacy_default_without_name_is_optional_positional() {
        let params = LegacyPositionalDefault::from_args([].into_iter()).unwrap();
        assert_eq!(params.count, 9);

        let params = LegacyPositionalDefault::from_args(["3"].into_iter()).unwrap();
        assert_eq!(params.count, 3);

        let args = ["--count", "3"];
        let err = LegacyPositionalDefault::from_args(args.into_iter()).unwrap_err();
        assert_eq!(err, Error::UnknownOption);
    }

    #[test]
    fn optional_positionals() {
        let params = OptionalPositionals::from_args(["in.txt"].into_iter()).unwrap();
        assert_eq!(params.input, "in.txt");
        assert_eq!(params.output, "out.txt");
        assert_eq!(params.level, 2);

        let args = ["in.txt", "res.txt"];
        let params = OptionalPositionals::from_args(args.into_iter()).unwrap();
        assert_eq!(params.output, "res.txt");
        assert_eq!(params.level, 2);

        let args = ["in.txt", "res.txt", "7"];
        let params = OptionalPositionals::from_args(args.into_iter()).unwrap();
        assert_eq!(params.level, 7);
    }

    #[test]
    fn optional_positionals_still_require_leading_positional() {
        let err = OptionalPositionals::from_args([].into_iter()).unwrap_err();
        assert_eq!(err, Error::MissingPositionalArgument("input"));
    }

    #[test]
    fn optional_positionals_reject_extra_arguments() {
        let args = ["a", "b", "1", "extra"];
        let err = OptionalPositionals::from_args(args.into_iter()).unwrap_err();
        assert_eq!(err, Error::InvalidNumberOfArguments);
    }

    #[test]
    fn flag_with_default() {
        let params = FlagWithDefault::from_args([].into_iter()).unwrap();
        assert!(params.color);
    }

    #[test]
    fn nonzero_without_default_is_required() {
        let err = WithNonZero::from_args([].into_iter()).unwrap_err();
        assert_eq!(err, Error::MissingPositionalArgument("port"));
    }

    #[test]
    fn missing_option_value() {
        let err = ExplicitNames::from_args(["--directory"].into_iter()).unwrap_err();
        assert_eq!(err, Error::MissingOptionValue("directory"));
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
        assert_eq!(err, Error::ParseError("num"));
    }

    #[test]
    fn unknown_flag() {
        let args = ["input", "--bogus"];
        let mut options = getargs::Options::new(args.into_iter());
        let err = NoOptions::parse(&mut options).unwrap_err();
        assert_eq!(err, Error::UnknownOption);
    }

    #[test]
    fn unknown_short_flag() {
        let args = ["input", "-z"];
        let mut options = getargs::Options::new(args.into_iter());
        let err = NoOptions::parse(&mut options).unwrap_err();
        assert_eq!(err, Error::UnknownOption);
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
            #[arg(long)]
            verbose: bool,
        }

        let args = ["extra"];
        let mut options = getargs::Options::new(args.into_iter());
        let err = FlagsOnly::parse(&mut options).unwrap_err();
        assert_eq!(err, Error::InvalidNumberOfArguments);
    }

    /// Copy files around.
    ///
    /// This second paragraph is only shown in long help, which is not generated.
    #[derive(Debug, GetArgs)]
    #[command(name = "copy")]
    struct HelpFull<'a> {
        /// Input file.
        input: &'a str,
        /// Output file
        #[arg(default_value = "out.txt")]
        output: &'a str,
        /// Verbose
        /// output.
        #[arg(short, long)]
        verbose: bool,
        /// Number of jobs
        #[arg(short, long, default_value_t = 4)]
        jobs: u32,
        /// Ignored doc comment
        #[arg(long, help = "Mode", value_name = "KIND")]
        mode: Mode<'a>,
        #[arg(short = 'x')]
        extract: bool,
        #[arg(short = 'C', default_value = "a dir")]
        directory: &'a str,
        #[arg(long, default_value_t = -3)]
        offset: i32,
        #[arg(long, default_value_t = NonZeroU32::new(5).unwrap())]
        threads: NonZeroU32,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Mode<'a>(&'a str);

    impl<'a> core::str::FromStr for Mode<'a> {
        type Err = ();

        fn from_str(_: &str) -> Result<Self, ()> {
            Ok(Mode("parsed"))
        }
    }

    #[derive(Debug, GetArgs)]
    #[command(name = "tool", about = "Overrides the doc comment")]
    /// Doc comment about text
    struct HelpCommand<'a> {
        #[arg(value_name = "FILE")]
        input: &'a str,
    }

    #[derive(Debug, GetArgs)]
    struct HelpDefaultName {
        #[arg(long)]
        dry_run: bool,
    }

    #[derive(Debug, GetArgs)]
    #[command(name = "bare", disable_help_flag)]
    struct HelpDisabled<'a> {
        input: &'a str,
    }

    #[derive(Debug, GetArgs)]
    #[command(disable_help_flag)]
    #[command(name = "host")]
    struct HelpDisabledWithShortH<'a> {
        /// Host to connect to
        #[arg(short, long = "host")]
        host: &'a str,
    }

    #[derive(Debug, GetArgs)]
    #[command(name = "empty")]
    struct HelpEmpty {}

    #[test]
    fn help_full_snapshot() {
        assert_eq!(
            HelpFull::HELP,
            "Copy files around

Usage: copy [OPTIONS] <INPUT> [OUTPUT]

Arguments:
  <INPUT>   Input file
  [OUTPUT]  Output file [default: out.txt]

Options:
  -v, --verbose            Verbose output
  -j, --jobs <JOBS>        Number of jobs [default: 4]
      --mode <KIND>        Mode
  -x
  -C <DIRECTORY>           [default: \"a dir\"]
      --offset <OFFSET>    [default: -3]
      --threads <THREADS>  [default: NonZeroU32 :: new(5).unwrap()]
  -h, --help               Print help"
        );
    }

    #[test]
    fn help_command_name_and_about() {
        assert_eq!(
            HelpCommand::HELP,
            "Overrides the doc comment

Usage: tool [OPTIONS] <FILE>

Arguments:
  <FILE>

Options:
  -h, --help  Print help"
        );
    }

    #[test]
    fn help_default_name_is_package_name() {
        assert_eq!(
            HelpDefaultName::HELP,
            "Usage: getargs-derive [OPTIONS]

Options:
      --dry-run
  -h, --help     Print help"
        );
    }

    #[test]
    fn help_disabled_snapshot() {
        assert_eq!(
            HelpDisabled::HELP,
            "Usage: bare <INPUT>\n\nArguments:\n  <INPUT>"
        );
    }

    #[test]
    fn help_empty_struct() {
        assert_eq!(
            HelpEmpty::HELP,
            "Usage: empty [OPTIONS]\n\nOptions:\n  -h, --help  Print help"
        );
    }

    #[test]
    fn help_long_and_short_flags() {
        for flag in ["-h", "--help"] {
            let err = HelpFull::from_args([flag].into_iter()).unwrap_err();
            assert_eq!(err, Error::Help(HelpFull::HELP));
            assert!(err.is_help());
        }
    }

    #[test]
    fn help_takes_priority_over_missing_required() {
        let err = HelpFull::from_args(["-v", "--help"].into_iter()).unwrap_err();
        assert_eq!(err, Error::Help(HelpFull::HELP));
        let err = RequiredOption::from_args(["--help"].into_iter()).unwrap_err();
        assert!(err.is_help());
    }

    #[test]
    fn help_after_double_dash_is_positional() {
        let params = OnePositional::from_args(["--", "--help"].into_iter()).unwrap();
        assert_eq!(params.first, "--help");
    }

    #[test]
    fn help_disabled_treats_help_as_unknown() {
        for flag in ["-h", "--help"] {
            let err = HelpDisabled::from_args(["in", flag].into_iter()).unwrap_err();
            assert_eq!(err, Error::UnknownOption);
            assert!(!err.is_help());
        }
    }

    #[test]
    fn help_disabled_allows_short_h() {
        let params = HelpDisabledWithShortH::from_args(["-h", "example.com"].into_iter()).unwrap();
        assert_eq!(params.host, "example.com");
        assert_eq!(
            HelpDisabledWithShortH::HELP,
            "Usage: host [OPTIONS]\n\nOptions:\n  -h, --host <HOST>  Host to connect to"
        );
    }

    #[test]
    fn is_help_false_for_other_errors() {
        assert!(!Error::UnknownOption.is_help());
        assert!(!Error::MissingPositionalArgument("x").is_help());
    }

    // ── Display impl tests ──────────────────────────────────────────

    #[test]
    fn error_display_help() {
        assert_eq!(
            alloc::format!("{}", Error::Help(HelpCommand::HELP)),
            HelpCommand::HELP
        );
    }

    #[test]
    fn error_display_missing_positional() {
        assert_eq!(
            alloc::format!("{}", Error::MissingPositionalArgument("output")),
            "Missing required argument: 'output'"
        );
    }

    #[test]
    fn error_display_unknown_option() {
        assert_eq!(alloc::format!("{}", Error::UnknownOption), "Unknown option");
    }

    #[test]
    fn error_display_missing_option_value() {
        assert_eq!(
            alloc::format!("{}", Error::MissingOptionValue("output")),
            "Option 'output' requires a value"
        );
    }

    #[test]
    fn error_display_parse_error() {
        assert_eq!(
            alloc::format!("{}", Error::ParseError("port")),
            "Failed to parse value for 'port'"
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
