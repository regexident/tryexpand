#[test]
pub fn expand() {
    tryexpand::expand(["tests/expand/pass/*.rs"]);
}

#[test]
pub fn expand_fail() {
    tryexpand::expand_fail(["tests/expand/fail/*.rs"]);
}

#[test]
pub fn expand_args() {
    tryexpand::expand_args(
        ["tests/expand_args/pass/*.rs"],
        ["--features", "test-feature"],
    );
}

#[test]
pub fn expand_args_fail() {
    tryexpand::expand_args_fail(
        ["tests/expand_args/fail/*.rs"],
        ["--features", "placebo-test-feature"],
    );
}
