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

mod expand_opts {
    #[test]
    pub fn pass() {
        tryexpand::expand_opts(
            ["tests/expand_opts/pass/*.rs"],
            tryexpand::Options::default().args(["--features", "test-feature"]),
        );
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        tryexpand::expand_opts(
            ["tests/expand_opts/fail/*.rs"],
            tryexpand::Options::default().args(["--features", "placebo-test-feature"]),
        );
    }
}

mod expand_opts_fail {
    #[test]
    pub fn pass() {
        tryexpand::expand_opts_fail(
            ["tests/expand_opts/fail/*.rs"],
            tryexpand::Options::default().args(["--features", "test-feature"]),
        );
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        tryexpand::expand_opts_fail(
            ["tests/expand_opts/pass/*.rs"],
            tryexpand::Options::default().args(["--features", "placebo-test-feature"]),
        );
    }
}
