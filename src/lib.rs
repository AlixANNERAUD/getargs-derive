#![doc = include_str!("../README.md")]
#![no_std]
#![deny(missing_docs)]

#[doc(hidden)]
extern crate self as getargs_derive;

mod error;

pub use error::*;

pub use getargs_derive_macros::GetArgs;
