#[test]
pub fn pass() {
    tryexpand::expand("tests/expand/*.rs");
}

#[test]
#[should_panic]
pub fn pass_fail() {
    tryexpand::expand("tests/expand-fail/*.rs");
}

#[test]
pub fn fail() {
    tryexpand::expand_fail("tests/expand-fail/*.rs");
}
