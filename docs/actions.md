# Flag Actions

Like Clap, a flag's behaviour can be chosen with `action`. Only this subset of Clap's `ArgAction` is supported, written as a bare name or as `ArgAction::Name`:

| Action     | Field type                            | Absent         | Each occurrence                       |
| ---------- | ------------------------------------- | -------------- | ------------------------------------- |
| `SetTrue`  | `bool`                                | `false`        | sets `true` (the default for `bool`)  |
| `SetFalse` | `bool`                                | `true`         | sets `false`                          |
| `Count`    | integer (`u8`, `u32`, `i64`, `usize`, ...) | `0`       | adds one, saturating at the type's maximum |

`Count` counts every occurrence, including repeated short flags grouped in one argument (`-vvv`) and mixed short and long forms (`-vv --verbose`).
A default (`default_value_t = ...`) replaces the value used when the flag is absent; as in Clap, it is not a starting point for counting.
Flags never take a value, so `--verbose=2` is rejected as an unknown option.

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    /// More output, repeat for even more.
    #[arg(short, long, action = Count)]
    verbose: u8,
    /// Disable colored output.
    #[arg(long = "no-color", action = SetFalse)]
    color: bool,
}

# fn main() -> Result<(), getargs_derive::Error<'static>> {
let cli = Cli::from_args(["-vvv", "--verbose", "--no-color"].into_iter())?;
assert_eq!(cli.verbose, 4);
assert!(!cli.color);

let cli = Cli::from_args([].into_iter())?;
assert_eq!(cli.verbose, 0);
assert!(cli.color);
# Ok(())
# }
```

In the generated help, a `Count` flag is followed by `...` (`-v, --verbose...`), as in Clap. See [Generated Help][guide-help].

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute or field, for:

- unsupported `action` values, and `action` without a value;
- `action` without `short`/`long`;
- `SetTrue`/`SetFalse` on a non-`bool` field, and `Count` on a non-integer field;
- `value_name` on a field with an `action`.

[guide-help]: help.md
