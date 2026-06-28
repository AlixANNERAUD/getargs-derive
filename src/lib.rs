#![doc = include_str!("../README.md")]
#![no_std]

#[doc(hidden)]
extern crate self as getargs_derive;

mod error;

pub use error::*;

pub use getargs_derive_macros::GetArgs;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(GetArgs)]
    struct TestParams<'a> {
        positional_field: &'a str,
        #[arg(flag)]
        bool_field: bool,
    }

    #[test]
    fn test_getargs_derive() {
        let args = ["value1", "--bool-field"];

        let mut options = getargs::Options::new(args.into_iter());

        let params = TestParams::parse(&mut options).unwrap();

        assert_eq!(params.positional_field, "value1");
        assert!(params.bool_field);
    }
}
