#[test]
pub fn expand() {
    tryexpand::expand(["tests/expand/pass/*.rs"]);
}

mod expand_checking {
    #[test]
    pub fn pass() {
        tryexpand::expand_checking(["tests/expand_checking/pass/*.rs"]);
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        // We need to use the `_opts` variant here as we need to
        // run the test with `options.skip_overwrite = true`
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand_opts_checking(
            ["tests/expand_checking/fail/*.rs"],
            tryexpand::Options::default().skip_overwrite(),
        );
    }
}

#[test]
pub fn expand_fail() {
    tryexpand::expand_fail(["tests/expand/fail/*.rs"]);
}

#[test]
pub fn expand_opts() {
    tryexpand::expand_opts(
        ["tests/expand_opts/pass/*.rs"],
        tryexpand::Options::default().args(["--features", "test-feature"]),
    );
}

mod expand_opts_checking {
    #[test]
    pub fn pass() {
        tryexpand::expand_opts_checking(
            ["tests/expand_opts_checking/pass/*.rs"],
            tryexpand::Options::default().args(["--features", "test-feature"]),
        );
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        tryexpand::expand_opts_checking(
            ["tests/expand_opts_checking/fail/*.rs"],
            tryexpand::Options::default()
                .args(["--features", "test-feature"])
                .skip_overwrite(),
        );
    }
}
