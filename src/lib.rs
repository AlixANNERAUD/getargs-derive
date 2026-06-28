//! This crate provides a derive macro for the `getargs` crate, allowing you to easily generate argument parsing code for your structs.

#![no_std]

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
