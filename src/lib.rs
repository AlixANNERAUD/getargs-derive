#![doc = include_str!("../docs/rustdoc-links.md")]
#![doc = include_str!("../README.md")]
#![no_std]
#![deny(missing_docs)]

#[doc(hidden)]
extern crate self as getargs_derive;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod collection;
mod command;
#[cfg(feature = "std")]
pub mod completions;
mod env;
mod error;
pub mod guide;
mod parsing;
mod suggest;
mod value_enum;

#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

pub use collection::Values;
pub use command::{ArgInfo, ArgKind, Command, CommandInfo};
pub use error::*;
pub use value_enum::{ParseValueEnumError, ValueEnum};

#[doc(hidden)]
pub use collection::{
    Collection as __Collection, finish_values as __finish_values, push_values as __push_values,
    values_len as __values_len,
};

#[doc(hidden)]
pub use env::{__EnvSupport, __env};

#[doc(hidden)]
pub use parsing::{
    RawArgs as __RawArgs, accepts_value as __accepts_value, negative_number as __negative_number,
};
#[doc(hidden)]
pub use suggest::suggest as __suggest;

#[doc(hidden)]
pub use value_enum::{
    HelpPart as __HelpPart, help_bytes as __help_bytes, help_len as __help_len,
    help_str as __help_str, value_name as __value_name,
};

pub use getargs_derive_macros::{GetArgs, ValueEnum};
