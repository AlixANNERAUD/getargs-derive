#[test]
fn ui() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
    #[cfg(not(feature = "std"))]
    cases.compile_fail("tests/ui/no_std/*.rs");
}
