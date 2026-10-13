# `no_std` Support

This crate is `#![no_std]`-compatible. It only depends on `core` and the `getargs` crate.
The generated code does not allocate: the help text and version line are `&'static str` values built at compile time.
