mod expand {
    #[test]
    pub fn pass() {
        tryexpand::expand("tests/pass/*.rs");
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        // This will fail due to syntax error caused expansion to fail
        // and that failure not being explicitly expected using `_fail` function
        tryexpand::expand("tests/fail/*.rs");
    }
}

mod expand_fail {
    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn pass() {
        tryexpand::expand_fail("tests/pass/*.rs");
    }

    #[test]
    pub fn fail() {
        // This will fail due to syntax error caused expansion to fail
        // and that failure not being explicitly expected using `_fail` function
        tryexpand::expand_fail("tests/fail/*.rs");
    }
}

mod expand_args {
    #[test]
    pub fn pass() {
        tryexpand::expand_args("tests/pass/*.rs", &["--features", "test-feature"]);
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        tryexpand::expand_args("tests/fail/*.rs", &["--features", "placebo-test-feature"]);
    }
}

mod expand_args_fail {
    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn pass() {
        tryexpand::expand_args_fail("tests/pass/*.rs", &["--features", "test-feature"]);
    }

    #[test]
    pub fn fail() {
        tryexpand::expand_args_fail("tests/fail/*.rs", &["--features", "placebo-test-feature"]);
    }
}
