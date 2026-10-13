# Default Values

Use `default_value = "..."` when the default is most naturally written as command-line text, and `default_value_t = expr` when it is a typed value:

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Server<'a> {
    #[arg(long, default_value = "127.0.0.1")]
    host: &'a str,
    #[arg(short, long, default_value = "8080")]
    port: u16,
    #[arg(short, long, default_value_t = 4)]
    workers: u32,
}

# fn main() -> Result<(), getargs_derive::Error<'static>> {
let server = Server::from_args(["-p", "3000"].into_iter())?;
assert_eq!(server.host, "127.0.0.1");
assert_eq!(server.port, 3000);
assert_eq!(server.workers, 4);
# Ok(())
# }
```

A `default_value` string is only parsed when the argument is absent; if it does not parse, `parse` returns `Error::InvalidValue`.
A field may have at most one default.

Arguments without a default are required, unless they are `Option<T>`. Positional arguments with a default are optional, and must come after all required positional arguments:

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Copy<'a> {
    source: &'a str,
    #[arg(default_value = "out.txt")]
    destination: &'a str,
}

# fn main() -> Result<(), getargs_derive::Error<'static>> {
let copy = Copy::from_args(["in.txt"].into_iter())?;
assert_eq!(copy.destination, "out.txt");
# Ok(())
# }
```

In the generated help, defaults are shown as `[default: value]`, except on flags. See [Generated Help][guide-help].

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute or field, for:

- more than one default on the same field, or any default on an `Option<T>` field.

[guide-help]: help.md
