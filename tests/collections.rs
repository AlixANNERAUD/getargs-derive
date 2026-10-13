use getargs_derive::{Error, GetArgs, ValueEnum, Values};

#[derive(Debug, GetArgs)]
#[command(name = "build")]
struct Build<'a> {
    #[arg(short = 'I', long)]
    include: Values<&'a str, 3>,
    #[arg(short, long, value_delimiter = ',')]
    feature: Values<&'a str, 4>,
    files: Values<&'a str, 2>,
}

#[test]
fn repeated_option_appends() {
    let build = Build::from_args(["-I", "a", "--include=b", "-Ic"].into_iter()).unwrap();
    assert_eq!(build.include, ["a", "b", "c"]);
    assert!(build.feature.is_empty());
    assert!(build.files.is_empty());
}

#[test]
fn option_exceeding_capacity() {
    let error =
        Build::from_args(["-I", "a", "-I", "b", "-I", "c", "-I", "d"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::TooManyValues {
            name: "--include <INCLUDE>",
            value: "d",
            max: 3,
        }
    );
    assert_eq!(
        error.to_string(),
        "unexpected value 'd' for '--include <INCLUDE>' found; no more were expected"
    );
}

#[test]
fn value_delimiter_splits() {
    let build = Build::from_args(["-f", "a,b", "--feature", "c"].into_iter()).unwrap();
    assert_eq!(build.feature, ["a", "b", "c"]);

    let error = Build::from_args(["-f", "a,b,c", "-f", "d,e"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::TooManyValues {
            name: "--feature <FEATURE>",
            value: "e",
            max: 4,
        }
    );
}

#[test]
fn positional_collects_remaining() {
    let build = Build::from_args(["x", "-I", "a", "y"].into_iter()).unwrap();
    assert_eq!(build.files, ["x", "y"]);
    assert_eq!(build.include, ["a"]);

    let error = Build::from_args(["x", "y", "z"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::TooManyValues {
            name: "<FILES>...",
            value: "z",
            max: 2,
        }
    );
}

#[test]
fn help_shows_repeated_values() {
    assert_eq!(
        Build::HELP,
        "Usage: build [OPTIONS] [FILES]...\n\nArguments:\n  [FILES]...\n\nOptions:\n  -I, --include <INCLUDE>\n  -f, --feature <FEATURE>\n  -h, --help               Print help"
    );
}

#[derive(Debug, GetArgs)]
#[command(name = "copy")]
struct Copy<'a> {
    #[arg(num_args = 2..=3)]
    paths: Values<&'a str, 8>,
}

#[test]
fn num_args_bounds() {
    let copy = Copy::from_args(["a", "b"].into_iter()).unwrap();
    assert_eq!(copy.paths, ["a", "b"]);

    let error = Copy::from_args([].into_iter()).unwrap_err();
    assert_eq!(error, Error::MissingArgument("<PATHS>..."));

    let error = Copy::from_args(["a"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::TooFewValues {
            name: "<PATHS>...",
            min: 2,
            actual: 1,
        }
    );
    assert_eq!(
        error.to_string(),
        "2 values required by '<PATHS>...'; only 1 was provided"
    );

    let error = Copy::from_args(["a", "b", "c", "d"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::TooManyValues {
            name: "<PATHS>...",
            value: "d",
            max: 3,
        }
    );

    assert!(
        Copy::HELP.starts_with("Usage: copy [OPTIONS] <PATHS>...\n\nArguments:\n  <PATHS>...\n")
    );
}

#[derive(Debug, GetArgs)]
#[command(name = "run")]
struct Run<'a> {
    #[arg(short, long)]
    verbose: bool,
    program: &'a str,
    #[arg(last = true)]
    arguments: Values<&'a str, 4>,
}

#[test]
fn last_collects_after_double_dash() {
    let run = Run::from_args(["cargo", "--", "-v", "--release"].into_iter()).unwrap();
    assert_eq!(run.program, "cargo");
    assert_eq!(run.arguments, ["-v", "--release"]);
    assert!(!run.verbose);

    let error = Run::from_args(["cargo", "build"].into_iter()).unwrap_err();
    assert_eq!(error, Error::UnexpectedArgument("build"));

    assert!(Run::HELP.starts_with(
        "Usage: run [OPTIONS] <PROGRAM> [-- <ARGUMENTS>...]\n\nArguments:\n  <PROGRAM>\n  [ARGUMENTS]...\n"
    ));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Level {
    Low,
    High,
}

fn parse_even(value: &str) -> Result<u8, ()> {
    match value.parse::<u8>() {
        Ok(number) if number % 2 == 0 => Ok(number),
        _ => Err(()),
    }
}

#[derive(Debug, GetArgs)]
struct Checked<'a> {
    #[arg(long, value_parser = 1..=9, value_delimiter = ',')]
    port: Values<u8, 4>,
    #[arg(long, value_enum)]
    level: Values<Level, 2>,
    #[arg(long, possible_values = ["a", "b"])]
    mode: Values<&'a str, 2>,
    #[arg(long, value_parser = parse_even)]
    even: Values<u8, 2>,
}

#[test]
fn elements_are_validated() {
    let checked = Checked::from_args(
        [
            "--port", "1,9", "--level", "high", "--level", "low", "--mode", "b", "--even", "4",
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(checked.port, [1, 9]);
    assert_eq!(checked.level, [Level::High, Level::Low]);
    assert_eq!(checked.mode, ["b"]);
    assert_eq!(checked.even, [4]);

    let error = Checked::from_args(["--port", "1,10"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::ValueOutOfRange {
            name: "--port <PORT>",
            value: "10",
            range: "1..=9",
        }
    );
    let error = Checked::from_args(["--level", "mid"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::InvalidValue {
            name: "--level <LEVEL>",
            value: "mid",
        }
    );
    let error = Checked::from_args(["--mode", "c"].into_iter()).unwrap_err();
    assert!(matches!(
        error,
        Error::InvalidPossibleValue { value: "c", .. }
    ));
    let error = Checked::from_args(["--even", "3"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::InvalidValue {
            name: "--even <EVEN>",
            value: "3",
        }
    );
}

#[derive(Debug, GetArgs)]
#[command(name = "tags")]
struct Defaults<'a> {
    #[arg(long, value_delimiter = ',', default_value = "a,b")]
    tag: Values<&'a str, 4>,
    #[arg(long, default_values = ["1", "2"])]
    level: Values<u8, 4>,
    #[arg(long, env = "NAMES", value_delimiter = ':')]
    name: Values<&'a str, 2>,
}

#[test]
fn defaults_and_env() {
    let defaults = Defaults::from_args_with_env([].into_iter(), |_| None).unwrap();
    assert_eq!(defaults.tag, ["a", "b"]);
    assert_eq!(defaults.level, [1, 2]);
    assert!(defaults.name.is_empty());

    let defaults =
        Defaults::from_args_with_env(["--tag", "c", "--level", "7"].into_iter(), |name| {
            (name == "NAMES").then_some("x:y")
        })
        .unwrap();
    assert_eq!(defaults.tag, ["c"]);
    assert_eq!(defaults.level, [7]);
    assert_eq!(defaults.name, ["x", "y"]);

    let defaults =
        Defaults::from_args_with_env(["--name", "z"].into_iter(), |_| Some("x:y")).unwrap();
    assert_eq!(defaults.name, ["z"]);

    let error = Defaults::from_args_with_env([].into_iter(), |_| Some("x:y:z")).unwrap_err();
    assert_eq!(
        error,
        Error::TooManyValues {
            name: "--name <NAME> [env: NAMES]",
            value: "z",
            max: 2,
        }
    );
}

#[test]
fn help_shows_defaults() {
    assert!(Defaults::HELP.contains("--tag <TAG>      [default: a,b]"));
    assert!(Defaults::HELP.contains("--level <LEVEL>  [default: 1, 2]"));
    assert!(Defaults::HELP.contains("--name <NAME>    [env: NAMES=]"));
}

#[derive(Debug, GetArgs)]
struct Shared<'a> {
    #[arg(short = 'D', long)]
    define: Values<&'a str, 2>,
}

#[derive(Debug, GetArgs)]
#[command(name = "compile")]
struct Compile<'a> {
    #[command(flatten)]
    shared: Shared<'a>,
    #[arg(num_args = 1..)]
    inputs: Values<&'a str, 4>,
}

#[test]
fn flattened_collections() {
    let compile = Compile::from_args(["a", "-D", "x", "b", "-Dy"].into_iter()).unwrap();
    assert_eq!(compile.shared.define, ["x", "y"]);
    assert_eq!(compile.inputs, ["a", "b"]);

    let error = Compile::from_args(["-D", "x"].into_iter()).unwrap_err();
    assert_eq!(error, Error::MissingArgument("<INPUTS>..."));

    assert!(Compile::HELP.starts_with(
        "Usage: compile [OPTIONS] <INPUTS>...\n\nArguments:\n  <INPUTS>...\n\nOptions:\n  -D, --define <DEFINE>\n"
    ));
}

#[derive(Debug, GetArgs)]
#[command(allow_negative_numbers)]
struct Numbers {
    #[arg(short, long)]
    verbose: bool,
    values: Values<i32, 4>,
}

#[test]
fn negative_numbers_are_collected() {
    let numbers = Numbers::from_args(["-1", "2", "-v", "-3"].into_iter()).unwrap();
    assert_eq!(numbers.values, [-1, 2, -3]);
    assert!(numbers.verbose);
}

#[derive(Debug, GetArgs)]
struct Related<'a> {
    #[arg(long, conflicts_with = "all")]
    only: Values<&'a str, 2>,
    #[arg(long)]
    all: bool,
}

#[test]
fn relations_see_collections() {
    let error = Related::from_args(["--only", "a", "--all"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::ArgumentConflict {
            name: "--only <ONLY>",
            other: "--all",
        }
    );
    let related = Related::from_args(["--all"].into_iter()).unwrap();
    assert!(related.all);
    assert!(related.only.is_empty());
}

#[test]
fn values_api() {
    let mut values: Values<u8, 2> = Values::default();
    assert_eq!(values.capacity(), 2);
    assert_eq!(values.push(1), Ok(()));
    assert_eq!(values.push(2), Ok(()));
    assert_eq!(values.push(3), Err(3));
    values[0] = 5;
    assert_eq!(values.as_slice(), &[5, 2]);
    assert_eq!(values.clone(), values);
    assert_eq!(format!("{values:?}"), "[5, 2]");
    for value in &mut values {
        *value += 1;
    }
    assert_eq!(values.iter().copied().sum::<u8>(), 9);
}

#[test]
fn values_drop_their_items() {
    let item = std::rc::Rc::new(());
    {
        let mut values: Values<std::rc::Rc<()>, 3> = Values::new();
        values.push(item.clone()).unwrap();
        values.push(item.clone()).unwrap();
        assert_eq!(std::rc::Rc::strong_count(&item), 3);
    }
    assert_eq!(std::rc::Rc::strong_count(&item), 1);
}

#[cfg(feature = "heapless")]
#[derive(Debug, GetArgs)]
struct Heapless<'a> {
    #[arg(short = 'I', long, value_delimiter = ',')]
    include: heapless::Vec<&'a str, 2>,
    #[arg(num_args = 1..)]
    files: heapless::Vec<u32, 3>,
}

#[cfg(feature = "heapless")]
#[test]
fn heapless_storage() {
    let parsed = Heapless::from_args(["-I", "a,b", "1", "2"].into_iter()).unwrap();
    assert_eq!(parsed.include.as_slice(), ["a", "b"]);
    assert_eq!(parsed.files.as_slice(), [1, 2]);

    let error = Heapless::from_args(["-I", "a,b,c", "1"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::TooManyValues {
            name: "--include <INCLUDE>",
            value: "c",
            max: 2,
        }
    );
    let error = Heapless::from_args(["1", "2", "3", "4"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::TooManyValues {
            name: "<FILES>...",
            value: "4",
            max: 3,
        }
    );
}

#[cfg(feature = "alloc")]
#[derive(Debug, GetArgs)]
struct Allocated<'a> {
    #[arg(short = 'I', long, value_delimiter = ',')]
    include: Vec<&'a str>,
    #[arg(last = true)]
    rest: Vec<String>,
}

#[cfg(feature = "alloc")]
#[test]
fn alloc_storage() {
    let arguments: Vec<String> = (0..100).map(|index| index.to_string()).collect();
    let parsed = Allocated::from_args(
        ["-I", "a,b", "-Ic", "--"]
            .into_iter()
            .chain(arguments.iter().map(String::as_str)),
    )
    .unwrap();
    assert_eq!(parsed.include, ["a", "b", "c"]);
    assert_eq!(parsed.rest, arguments);
}
