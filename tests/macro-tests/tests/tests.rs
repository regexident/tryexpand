mod expand {
    #[test]
    pub fn pass() {
        tryexpand::expand(["tests/expand/pass/*.rs"]);
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        // This will fail due to syntax error caused expansion to fail
        // and that failure not being explicitly expected using `_fail` function
        tryexpand::expand(["tests/expand/fail/*.rs"]);
    }
}

mod expand_fail {
    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn pass() {
        tryexpand::expand_fail(["tests/expand/pass/*.rs"]);
    }

    #[test]
    pub fn fail() {
        // This will fail due to syntax error caused expansion to fail
        // and that failure not being explicitly expected using `_fail` function
        tryexpand::expand_fail(["tests/expand/fail/*.rs"]);
    }
}

mod expand_args {
    #[test]
    pub fn pass() {
        tryexpand::expand_args(
            ["tests/expand_args/pass/*.rs"],
            &["--features", "test-feature"],
        );
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        tryexpand::expand_args(
            ["tests/expand_args/fail/*.rs"],
            &["--features", "placebo-test-feature"],
        );
    }
}

mod expand_args_fail {
    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn pass() {
        tryexpand::expand_args_fail(
            ["tests/expand_args/pass/*.rs"],
            &["--features", "test-feature"],
        );
    }

    #[test]
    pub fn fail() {
        tryexpand::expand_args_fail(
            ["tests/expand_args/fail/*.rs"],
            &["--features", "placebo-test-feature"],
        );
    }
}
