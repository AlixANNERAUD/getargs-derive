use getargs_derive::{Error, GetArgs};

fn lookup<'a>(vars: &'a [(&'a str, &'a str)]) -> impl FnMut(&str) -> Option<&'a str> {
    move |name| {
        vars.iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| *value)
    }
}

#[derive(Debug, GetArgs)]
#[command(name = "server")]
struct Server<'a> {
    /// Address to bind.
    #[arg(long, env = "SERVER_HOST", default_value = "127.0.0.1")]
    host: &'a str,
    /// Port to listen on.
    #[arg(short, long, env = "SERVER_PORT")]
    port: u16,
    #[arg(long, env, default_value_t = 4)]
    workers: u32,
    #[arg(long, env = "SERVER_LOG")]
    log: Option<&'a str>,
    #[arg(env = "SERVER_ROOT")]
    root: Option<&'a str>,
}

#[test]
fn command_line_takes_precedence_over_env() {
    let vars = [("SERVER_PORT", "80"), ("SERVER_HOST", "0.0.0.0")];
    let server = Server::from_args_with_env(
        ["--port", "8080", "--host", "localhost"].into_iter(),
        lookup(&vars),
    )
    .unwrap();
    assert_eq!(server.port, 8080);
    assert_eq!(server.host, "localhost");
}

#[test]
fn env_takes_precedence_over_default() {
    let vars = [
        ("SERVER_PORT", "80"),
        ("SERVER_HOST", "0.0.0.0"),
        ("WORKERS", "16"),
        ("SERVER_LOG", "debug"),
        ("SERVER_ROOT", "/srv"),
    ];
    let server = Server::from_args_with_env([].into_iter(), lookup(&vars)).unwrap();
    assert_eq!(server.port, 80);
    assert_eq!(server.host, "0.0.0.0");
    assert_eq!(server.workers, 16);
    assert_eq!(server.log, Some("debug"));
    assert_eq!(server.root, Some("/srv"));
}

#[test]
fn default_used_when_env_unset() {
    let vars = [("SERVER_PORT", "80")];
    let server = Server::from_args_with_env([].into_iter(), lookup(&vars)).unwrap();
    assert_eq!(server.host, "127.0.0.1");
    assert_eq!(server.workers, 4);
    assert_eq!(server.log, None);
    assert_eq!(server.root, None);
}

#[test]
fn missing_without_env_or_default() {
    let err = Server::from_args_with_env([].into_iter(), lookup(&[])).unwrap_err();
    assert_eq!(err, Error::MissingArgument("--port <PORT>"));
}

#[test]
fn invalid_env_value_names_the_variable() {
    let vars = [("SERVER_PORT", "eighty")];
    let err = Server::from_args_with_env([].into_iter(), lookup(&vars)).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "--port <PORT> [env: SERVER_PORT]",
            value: "eighty",
        }
    );
    assert_eq!(
        err.to_string(),
        "invalid value 'eighty' for '--port <PORT> [env: SERVER_PORT]'"
    );
}

#[test]
fn env_not_read_when_given_on_command_line() {
    let mut calls = Vec::new();
    let args = [
        "-p",
        "1",
        "--host",
        "h",
        "--workers",
        "2",
        "--log",
        "l",
        "r",
    ];
    Server::from_args_with_env(args.into_iter(), |name| {
        calls.push(name.to_owned());
        None
    })
    .unwrap();
    assert!(calls.is_empty());
}

#[test]
fn parse_with_env_uses_existing_options() {
    let vars = [("SERVER_PORT", "443")];
    let mut options = getargs::Options::new(["--host", "example.org"].into_iter());
    let server = Server::parse_with_env(&mut options, lookup(&vars)).unwrap();
    assert_eq!(server.host, "example.org");
    assert_eq!(server.port, 443);
}

#[test]
fn help_shows_env() {
    assert_eq!(
        Server::HELP,
        "Usage: server [OPTIONS] [ROOT]

Arguments:
  [ROOT]  [env: SERVER_ROOT=]

Options:
      --host <HOST>        Address to bind [env: SERVER_HOST=] [default: 127.0.0.1]
  -p, --port <PORT>        Port to listen on [env: SERVER_PORT=]
      --workers <WORKERS>  [env: WORKERS=] [default: 4]
      --log <LOG>          [env: SERVER_LOG=]
  -h, --help               Print help"
    );
}

#[cfg(feature = "std")]
mod process {
    use getargs_derive::{Error, GetArgs};

    #[derive(Debug, GetArgs)]
    struct Process<'a> {
        #[arg(long, env = "GETARGS_DERIVE_TEST_PROCESS_NAME")]
        name: &'a str,
        #[arg(long, env = "GETARGS_DERIVE_TEST_PROCESS_COUNT", default_value_t = 1)]
        count: u32,
    }

    #[derive(Debug, GetArgs)]
    struct Invalid {
        #[arg(long, env = "GETARGS_DERIVE_TEST_INVALID_COUNT")]
        count: u32,
    }

    #[derive(Debug, GetArgs)]
    struct Unset {
        #[arg(long, env = "GETARGS_DERIVE_TEST_UNSET_COUNT", default_value_t = 7)]
        count: u32,
    }

    #[test]
    fn reads_process_environment() {
        unsafe {
            std::env::set_var("GETARGS_DERIVE_TEST_PROCESS_NAME", "from-env");
            std::env::set_var("GETARGS_DERIVE_TEST_PROCESS_COUNT", "3");
        }
        let process = Process::from_args([].into_iter()).unwrap();
        assert_eq!(process.name, "from-env");
        assert_eq!(process.count, 3);

        let mut options = getargs::Options::new(["--count", "9"].into_iter());
        let process = Process::parse(&mut options).unwrap();
        assert_eq!(process.name, "from-env");
        assert_eq!(process.count, 9);
    }

    #[test]
    fn reports_invalid_process_environment_value() {
        unsafe {
            std::env::set_var("GETARGS_DERIVE_TEST_INVALID_COUNT", "many");
        }
        let err = Invalid::from_args([].into_iter()).unwrap_err();
        assert_eq!(
            err,
            Error::InvalidValue {
                name: "--count <COUNT> [env: GETARGS_DERIVE_TEST_INVALID_COUNT]",
                value: "many",
            }
        );

        let valid = Invalid::from_args(["--count", "2"].into_iter()).unwrap();
        assert_eq!(valid.count, 2);
    }

    #[test]
    fn falls_back_to_default_when_unset() {
        let unset = Unset::from_args([].into_iter()).unwrap();
        assert_eq!(unset.count, 7);
    }
}
