mod expand {
    #[test]
    pub fn pass() {
        tryexpand::expand(["tests/expand/pass/*.rs"]);
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        tryexpand::expand(["tests/expand/fail/*.rs"]);
    }
}

mod expand_fail {
    #[test]
    pub fn pass() {
        tryexpand::expand_fail(["tests/expand/fail/*.rs"]);
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        tryexpand::expand_fail(["tests/expand/pass/*.rs"]);
    }
}

mod expand_args {
    #[test]
    pub fn pass() {
        tryexpand::expand_args(
            ["tests/expand_args/pass/*.rs"],
            ["--features", "test-feature"],
        );
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        tryexpand::expand_args(
            ["tests/expand_args/fail/*.rs"],
            ["--features", "placebo-test-feature"],
        );
    }
}

mod expand_args_fail {
    #[test]
    pub fn pass() {
        tryexpand::expand_args_fail(
            ["tests/expand_args/fail/*.rs"],
            ["--features", "test-feature"],
        );
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        tryexpand::expand_args_fail(
            ["tests/expand_args/pass/*.rs"],
            ["--features", "placebo-test-feature"],
        );
    }
}
