const UNMATCHED_PATTERN: &str = "no/matches/for/this/path";

const EMPTY_PATTERNS: [&str; 0] = [];

mod expand {
    use super::*;

    const PASS_PATTERN: &str = "tests/expand/pass/*.rs";
    const FAIL_PATTERN: &str = "tests/expand/fail/*.rs";
    const UNMATCHED_PATTERN: &str = super::UNMATCHED_PATTERN;

    #[test]
    pub fn pass() {
        tryexpand::expand([PASS_PATTERN]);
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_pass() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([FAIL_PATTERN]).skip_overwrite();
    }

    #[test]
    pub fn expect_fail() {
        tryexpand::expand([FAIL_PATTERN]).expect_fail();
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_expect_fail() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([PASS_PATTERN])
            .skip_overwrite()
            .expect_fail();
    }

    #[test]
    #[should_panic(expected = "no file patterns provided")]
    pub fn no_paths_provided() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand(EMPTY_PATTERNS).skip_overwrite();
    }

    #[test]
    #[should_panic(expected = "no matching files found for:\n    no/matches/for/this/path")]
    pub fn no_files_found() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([PASS_PATTERN, UNMATCHED_PATTERN]).skip_overwrite();
    }
}

mod check {
    use super::*;

    const PASS_PATTERN: &str = "tests/check/pass/*.rs";
    const FAIL_PATTERN: &str = "tests/check/fail/*.rs";
    const UNMATCHED_PATTERN: &str = super::UNMATCHED_PATTERN;

    #[test]
    pub fn pass() {
        tryexpand::check([PASS_PATTERN]);
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_pass() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::check([FAIL_PATTERN]).skip_overwrite();
    }

    #[test]
    pub fn expect_fail() {
        tryexpand::check([FAIL_PATTERN]).expect_fail();
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_expect_fail() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::check([PASS_PATTERN])
            .skip_overwrite()
            .expect_fail();
    }

    #[test]
    #[should_panic(expected = "no file patterns provided")]
    pub fn no_paths_provided() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::check(EMPTY_PATTERNS).skip_overwrite();
    }

    #[test]
    #[should_panic(expected = "no matching files found for:\n    no/matches/for/this/path")]
    pub fn no_files_found() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::check([PASS_PATTERN, UNMATCHED_PATTERN]).skip_overwrite();
    }
}

mod run {
    const PASS_PATTERN: &str = "tests/run/pass/*.rs";
    const FAIL_PATTERN: &str = "tests/run/fail/*.rs";

    #[test]
    pub fn expect_pass() {
        tryexpand::run([PASS_PATTERN]);
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_expect_pass() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::run([FAIL_PATTERN]).skip_overwrite();
    }

    #[test]
    pub fn expect_fail() {
        tryexpand::run([FAIL_PATTERN]).expect_fail();
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_expect_fail() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::run([PASS_PATTERN])
            .skip_overwrite()
            .expect_fail();
    }
}

mod run_tests {
    const PASS_PATTERN: &str = "tests/run_tests/pass/*.rs";
    const FAIL_PATTERN: &str = "tests/run_tests/fail/*.rs";

    #[test]
    pub fn expect_pass() {
        tryexpand::run_tests([PASS_PATTERN]);
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_expect_pass() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::run_tests([FAIL_PATTERN]).skip_overwrite();
    }

    #[test]
    pub fn expect_fail() {
        tryexpand::run_tests([FAIL_PATTERN]).expect_fail();
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_expect_fail() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::run_tests([PASS_PATTERN])
            .skip_overwrite()
            .expect_fail();
    }
}

mod and_check {
    const PASS_PATTERN: &str = "tests/and_check/pass/*.rs";
    const FAIL_PATTERN: &str = "tests/and_check/fail/*.rs";

    #[test]
    pub fn expect_pass() {
        tryexpand::expand([PASS_PATTERN]).and_check();
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_expect_pass() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([FAIL_PATTERN])
            .and_check()
            .skip_overwrite();
    }

    #[test]
    pub fn expect_fail() {
        tryexpand::expand([FAIL_PATTERN]).and_check().expect_fail();
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_expect_fail() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([PASS_PATTERN])
            .and_check()
            .skip_overwrite()
            .expect_fail();
    }
}

mod and_run {
    const PASS_PATTERN: &str = "tests/and_run/pass/*.rs";
    const FAIL_PATTERN: &str = "tests/and_run/fail/*.rs";

    #[test]
    pub fn expect_pass() {
        tryexpand::expand([PASS_PATTERN]).and_run();
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_expect_pass() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([FAIL_PATTERN]).and_run().skip_overwrite();
    }

    #[test]
    pub fn expect_fail() {
        tryexpand::expand([FAIL_PATTERN]).and_run().expect_fail();
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_expect_fail() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([PASS_PATTERN])
            .and_run()
            .skip_overwrite()
            .expect_fail();
    }
}

mod and_run_tests {
    const PASS_PATTERN: &str = "tests/and_run_tests/pass/*.rs";
    const FAIL_PATTERN: &str = "tests/and_run_tests/fail/*.rs";

    #[test]
    pub fn expect_pass() {
        tryexpand::expand([PASS_PATTERN]).and_run_tests();
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_expect_pass() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([FAIL_PATTERN])
            .and_run_tests()
            .skip_overwrite();
    }

    #[test]
    pub fn expect_fail() {
        tryexpand::expand([FAIL_PATTERN])
            .and_run_tests()
            .expect_fail();
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_expect_fail() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([PASS_PATTERN])
            .and_run_tests()
            .skip_overwrite()
            .expect_fail();
    }
}

mod args {
    const PASS_PATTERN: &str = "tests/args/pass/*.rs";
    const FAIL_PATTERN: &str = "tests/args/fail/*.rs";

    #[test]
    pub fn pass() {
        tryexpand::expand([PASS_PATTERN]).args(["--features", "test-feature"]);
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_pass() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([FAIL_PATTERN])
            .args(["--features", "test-feature"])
            .skip_overwrite();
    }

    #[test]
    pub fn fail() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([FAIL_PATTERN])
            .args(["--features", "placebo-feature"])
            .expect_fail();
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_fail() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([PASS_PATTERN])
            .args(["--features", "placebo-feature"])
            .skip_overwrite()
            .expect_fail();
    }
}

mod envs {
    const PASS_PATTERN: &str = "tests/envs/pass/*.rs";
    const FAIL_PATTERN: &str = "tests/envs/fail/*.rs";

    #[test]
    pub fn pass() {
        tryexpand::expand([PASS_PATTERN]).envs([("TEST_ENV", "test-env-var-value")]);
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_pass() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([FAIL_PATTERN])
            .envs([("TEST_ENV", "test-env-var-value")])
            .skip_overwrite();
    }

    #[test]
    pub fn fail() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([FAIL_PATTERN])
            .envs([("PLACEBO_ENV", "placebo-env-var-value")])
            .expect_fail();
    }

    #[test]
    #[should_panic(expected = "tests failed")]
    pub fn verify_fail() {
        // We need to test with .skip_overwrite()
        // to avoid overwriting snapshots of `pass()`:
        tryexpand::expand([PASS_PATTERN])
            .envs([("PLACEBO_ENV", "placebo-env-var-value")])
            .skip_overwrite()
            .expect_fail();
    }
}
