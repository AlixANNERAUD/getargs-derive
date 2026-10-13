# `no_std` Support

This crate is `#![no_std]`-compatible. It only depends on `core` and the `getargs` crate.
The generated code does not allocate: the help text and version line are `&'static str` values built at compile time.

The optional `std` feature lets `parse` and `from_args` read [environment variable fallbacks][guide-env] from the process environment,
and enables the [shell completion and man page generators][guide-completions]. The command metadata they read is available without it.
Without it, structs with `env` fields read them through a lookup function passed to `parse_with_env` or `from_args_with_env`, which does not allocate either.

[guide-completions]: completions.md
[guide-env]: env.md
