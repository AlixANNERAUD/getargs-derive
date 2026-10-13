#![doc = include_str!("../docs/rustdoc-links.md")]
#![doc = include_str!("../README.md")]
#![no_std]
#![deny(missing_docs)]

#[doc(hidden)]
extern crate self as getargs_derive;

#[cfg(feature = "std")]
extern crate std;

mod env;
mod error;
pub mod guide;
mod suggest;

pub use error::*;

#[doc(hidden)]
pub use env::{__EnvSupport, __env};

#[doc(hidden)]
pub use suggest::suggest as __suggest;

pub use getargs_derive_macros::GetArgs;
