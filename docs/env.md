# Environment Variables

Like Clap, an option or positional argument can fall back to an environment variable when it is absent from the command line.
Add `env = "NAME"` to its `#[arg(...)]` attribute, or a bare `env` to use the upper-cased field name (`workers` → `WORKERS`):

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Server<'a> {
    #[arg(long, env = "SERVER_HOST", default_value = "127.0.0.1")]
    host: &'a str,
    #[arg(short, long, env = "SERVER_PORT")]
    port: u16,
    #[arg(long, env)]
    workers: Option<u32>,
}

# fn main() -> Result<(), getargs_derive::Error<'static>> {
let vars = [("SERVER_PORT", "8080"), ("SERVER_HOST", "0.0.0.0")];
let lookup = |name: &str| vars.iter().find(|(key, _)| *key == name).map(|(_, value)| *value);

let server = Server::from_args_with_env(["--host", "localhost"].into_iter(), lookup)?;
assert_eq!(server.host, "localhost");
assert_eq!(server.port, 8080);
assert_eq!(server.workers, None);
# Ok(())
# }
```

The value is taken from the command line first, then from the environment variable, then from the [default][guide-defaults].
An `Option<T>` field is `None` only when it is absent from both the command line and the environment, and a field without a default is reported as missing only when the variable is not set either.
The variable is only read when the argument is absent from the command line, and its value is used as-is for `&str` fields, parsed with `ValueEnum::from_value` for `value_enum` fields, and otherwise parsed with `FromStr`, with the same [value validation][guide-validation] as command-line values.
If it does not parse, `parse` returns `Error::InvalidValue` with the argument name followed by the variable, e.g. `"--port <PORT> [env: SERVER_PORT]"`.

`env` is not supported on flags. This is a Clap-inspired subset: Clap's `hide_env`, `hide_env_values` and boolean parsing of environment values for flags are not supported.

## Reading the Process Environment

A struct with at least one `env` field, or that [flattens][guide-flatten] other structs, gets two more methods, `parse_with_env` and `from_args_with_env`, taking a lookup function `FnMut(&str) -> Option<&'a str>` called with a variable name.
They work in `no_std` builds: the lookup can read a configuration block, a bootloader command line, or a test fixture.

With the `std` feature of this crate, `parse` and `from_args` read the process environment with `std::env::var`:

```toml
[dependencies]
getargs-derive = { version = "0.1", features = ["std"] }
```

```rust
# #[cfg(feature = "std")]
# fn main() -> Result<(), getargs_derive::Error<'static>> {
use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Server {
    #[arg(short, long, env = "GETARGS_DERIVE_GUIDE_PORT", default_value_t = 80)]
    port: u16,
}

let server = Server::from_args(["-p", "8080"].into_iter())?;
assert_eq!(server.port, 8080);
# Ok(())
# }
# #[cfg(not(feature = "std"))]
# fn main() {}
```

Without the `std` feature, calling `parse` or `from_args` on such a struct is a compile error naming the feature; use `parse_with_env` or `from_args_with_env` instead.
Structs without `env` fields are not affected by the feature.

The process environment holds owned strings, while parsed arguments borrow for the lifetime `'a` of the command line.
To keep `&'a str` fields and the value in `Error::InvalidValue` working without copying the command line, `parse` and `from_args` leak each environment value they read
(`Box::leak`), once per variable read, and only when the argument is absent from the command line.
This is negligible when parsing once at startup; when parsing repeatedly, prefer `parse_with_env` with your own lookup.
Variables whose value is not valid Unicode are treated as unset.

In the generated help, the variable is shown as `[env: NAME=]`, before the default, as in Clap. The value is not shown, as the help is a compile-time constant. See [Generated Help][guide-help].

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute or field, for:

- `env` values that are not a string literal, are empty, or contain `=` or NUL characters, and `env` given more than once on the same field;
- `env` on a flag;
- calling `parse` or `from_args` on a struct with `env` fields without the `std` feature.

[guide-defaults]: defaults.md
[guide-flatten]: flatten.md
[guide-help]: help.md
[guide-validation]: validation.md
