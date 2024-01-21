mod expand {
    #[test]
    pub fn pass() {
        tryexpand::expand(["tests/expand/pass/*.rs"]);
    }

    #[test]
    pub fn fail() {
        tryexpand::expand(["tests/expand/fail/*.rs"]).expect_fail();
    }
}
