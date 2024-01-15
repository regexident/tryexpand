const UNMATCHED_PATTERN: &str = "no/matches/for/this/path";

mod expand {
    const PASS_PATTERN: &str = "tests/expand/pass/*.rs";
    const FAIL_PATTERN: &str = "tests/expand/fail/*.rs";
    const UNMATCHED_PATTERN: &str = super::UNMATCHED_PATTERN;

    #[test]
    pub fn pass() {
        tryexpand::expand([PASS_PATTERN]);
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        tryexpand::expand([FAIL_PATTERN]);
    }

    #[test]
    #[should_panic(expected = "no file patterns provided")]
    pub fn no_paths_provided() {
        tryexpand::expand(Vec::<&str>::new());
    }

    #[test]
    #[should_panic(expected = "no matching files found for:\n    no/matches/for/this/path")]
    pub fn no_files_found() {
        tryexpand::expand([PASS_PATTERN, UNMATCHED_PATTERN]);
    }
}

mod expand_checking {
    const PASS_PATTERN: &str = "tests/expand/pass/*.rs";
    const UNMATCHED_PATTERN: &str = super::UNMATCHED_PATTERN;

    // Checking for passing/failing tests requires macros,
    // which this test project is not concerned with.
    //
    // This is because the observable behavior of `expand_checking()`
    // will only differ from that of `expand()` if the error
    // gets introduced during expansion.
    // Otherwise the expansion phase would already detect the issue
    // and the function would short-circuit.
    //
    // We thus perform these checks in the `macros-tests`
    // and `proc-macro-tests` test projects (mostly the former).

    #[test]
    #[should_panic(expected = "no file patterns provided")]
    pub fn no_paths_provided() {
        tryexpand::expand_checking(Vec::<&str>::new());
    }

    #[test]
    #[should_panic(expected = "no matching files found for:\n    no/matches/for/this/path")]
    pub fn no_files_found() {
        tryexpand::expand_checking([PASS_PATTERN, UNMATCHED_PATTERN]);
    }
}

mod expand_fail {
    const PASS_PATTERN: &str = "tests/expand/fail/*.rs";
    const FAIL_PATTERN: &str = "tests/expand/pass/*.rs";
    const UNMATCHED_PATTERN: &str = super::UNMATCHED_PATTERN;

    #[test]
    pub fn pass() {
        tryexpand::expand_fail([PASS_PATTERN]);
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        tryexpand::expand_fail([FAIL_PATTERN]);
    }

    #[test]
    #[should_panic(expected = "no file patterns provided")]
    pub fn no_paths_provided() {
        tryexpand::expand(Vec::<&str>::new());
    }

    #[test]
    #[should_panic(expected = "no matching files found for:\n    no/matches/for/this/path")]
    pub fn no_files_found() {
        tryexpand::expand([PASS_PATTERN, UNMATCHED_PATTERN]);
    }
}

mod expand_opts {
    const PASS_PATTERN: &str = "tests/expand_opts/pass/*.rs";
    const FAIL_PATTERN: &str = "tests/expand_opts/fail/*.rs";
    const UNMATCHED_PATTERN: &str = super::UNMATCHED_PATTERN;

    #[test]
    pub fn pass() {
        tryexpand::expand_opts(
            [PASS_PATTERN],
            tryexpand::Options::default().args(["--features", "test-feature"]),
        );
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        tryexpand::expand_opts(
            [FAIL_PATTERN],
            tryexpand::Options::default().args(["--features", "placebo-test-feature"]),
        );
    }

    #[test]
    #[should_panic(expected = "no file patterns provided")]
    pub fn no_paths_provided() {
        tryexpand::expand_opts(
            Vec::<&str>::new(),
            tryexpand::Options::default().args(["--features", "placebo-test-feature"]),
        );
    }

    #[test]
    #[should_panic(expected = "no matching files found for:\n    no/matches/for/this/path")]
    pub fn no_files_found() {
        tryexpand::expand_opts(
            [PASS_PATTERN, UNMATCHED_PATTERN],
            tryexpand::Options::default().args(["--features", "placebo-test-feature"]),
        );
    }
}

mod expand_opts_checking {
    const PASS_PATTERN: &str = "tests/expand_opts/pass/*.rs";
    const UNMATCHED_PATTERN: &str = super::UNMATCHED_PATTERN;

    // Checking for passing/failing tests requires macros,
    // which this test project is not concerned with.
    //
    // This is because the observable behavior of `expand_checking()`
    // will only differ from that of `expand()` if the error
    // gets introduced during expansion.
    // Otherwise the expansion phase would already detect the issue
    // and the function would short-circuit.
    //
    // We thus perform these checks in the `macros-tests`
    // and `proc-macro-tests` test projects (mostly the former).

    #[test]
    #[should_panic(expected = "no file patterns provided")]
    pub fn no_paths_provided() {
        tryexpand::expand_opts_checking(
            Vec::<&str>::new(),
            tryexpand::Options::default().args(["--features", "placebo-test-feature"]),
        );
    }

    #[test]
    #[should_panic(expected = "no matching files found for:\n    no/matches/for/this/path")]
    pub fn no_files_found() {
        tryexpand::expand_opts_checking(
            [PASS_PATTERN, UNMATCHED_PATTERN],
            tryexpand::Options::default().args(["--features", "placebo-test-feature"]),
        );
    }
}

mod expand_opts_fail {
    const PASS_PATTERN: &str = "tests/expand_opts/fail/*.rs";
    const FAIL_PATTERN: &str = "tests/expand_opts/pass/*.rs";
    const UNMATCHED_PATTERN: &str = super::UNMATCHED_PATTERN;

    #[test]
    pub fn pass() {
        tryexpand::expand_opts_fail(
            [PASS_PATTERN],
            tryexpand::Options::default().args(["--features", "test-feature"]),
        );
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn fail() {
        tryexpand::expand_opts_fail(
            [FAIL_PATTERN],
            tryexpand::Options::default().args(["--features", "placebo-test-feature"]),
        );
    }

    #[test]
    #[should_panic(expected = "no file patterns provided")]
    pub fn no_paths_provided() {
        tryexpand::expand_opts_fail(
            Vec::<&str>::new(),
            tryexpand::Options::default().args(["--features", "placebo-test-feature"]),
        );
    }

    #[test]
    #[should_panic(expected = "no matching files found for:\n    no/matches/for/this/path")]
    pub fn no_files_found() {
        tryexpand::expand_opts_fail(
            [PASS_PATTERN, UNMATCHED_PATTERN],
            tryexpand::Options::default().args(["--features", "placebo-test-feature"]),
        );
    }
}
