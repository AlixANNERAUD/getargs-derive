# `no_std` Support

This crate is `#![no_std]`-compatible. It only depends on `core` and the `getargs` crate, plus `heapless` with the `heapless` feature.
The generated code does not allocate: the help text and version line are `&'static str` values built at compile time.
The only exception is a `Vec<T>` field, which requires the `alloc` feature.
[Raw `&[u8]` arguments][guide-non-utf8] are parsed with `from_bytes` and `parse_bytes` without any feature, into `&[u8]` fields that borrow them.

The optional `std` feature lets `parse` and `from_args` read [environment variable fallbacks][guide-env] from the process environment,
and enables the [shell completion and man page generators][guide-completions]. The command metadata they read is available without it.
Without it, structs with `env` fields read them through a lookup function passed to `parse_with_env` or `from_args_with_env`, which does not allocate either.

## Features

| Feature    | Enables                                                                                   | Requires                |
| ---------- | ----------------------------------------------------------------------------------------- | ----------------------- |
| none       | Everything except the items below, including [repeated values][guide-collections] stored in `Values<T, N>`. | `core`                  |
| `heapless` | `heapless::Vec<T, N>` collection fields, stored inline without allocation.                | `core`, `heapless` 0.9  |
| `alloc`    | `Vec<T>` collection fields.                                                               | `alloc` (an allocator)  |
| `std`      | Reading `env` fallbacks from the process environment in `parse` and `from_args`, the completion and man page generators, `&OsStr`, `&Path`, `OsString` and `PathBuf` fields accepting [non-UTF-8 arguments][guide-non-utf8], and `from_args_os`. Implies `alloc`. | `std`                   |

Without `alloc`, collecting repeated values needs a bounded collection: its capacity is part of the struct, and extra values are reported as
`Error::TooManyValues` instead of growing the storage. See [Repeated Values][guide-collections] for the trade-offs.

[guide-collections]: collections.md
[guide-completions]: completions.md
[guide-env]: env.md
[guide-non-utf8]: non_utf8.md
