# Positional and Optional Arguments

Every field without `short` or `long` is a positional argument, filled in declaration order.
A positional argument is required unless it has a default (see [Default Values][guide-defaults]) or is an `Option<T>`.

## Optional Arguments

Wrap a field's type in `Option<T>` to make it optional without a default, for named options and positional arguments alike.
The inner value is borrowed for `Option<&str>` and parsed with `FromStr` otherwise:

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Fetch<'a> {
    url: &'a str,
    destination: Option<&'a str>,
    #[arg(short, long)]
    retries: Option<u8>,
}

# fn main() -> Result<(), getargs_derive::Error> {
let fetch = Fetch::from_args(["https://example.com"].into_iter())?;
assert_eq!(fetch.destination, None);
assert_eq!(fetch.retries, None);

let fetch = Fetch::from_args(["https://example.com", "page.html", "-r", "3"].into_iter())?;
assert_eq!(fetch.destination, Some("page.html"));
assert_eq!(fetch.retries, Some(3));
# Ok(())
# }
```

- An `Option<T>` field cannot have a default: use a `T` field with `default_value` or `default_value_t` instead.
- An `Option<T>` positional argument is optional, so it must come after all required positional arguments, like a positional argument with a default.
- `Option<bool>` is rejected: use a `bool` flag, which is `false` when absent. `Option<Option<T>>` is rejected as well.
- In the generated help, `Option<T>` positionals are shown as `[NAME]`, and no default is displayed.

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute or field, for:

- positional `bool` fields, and `Option<bool>` or `Option<Option<T>>` fields;
- any default on an `Option<T>` field;
- a required positional argument following an optional one.

[guide-defaults]: defaults.md
