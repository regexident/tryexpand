#[test]
pub fn expand() {
    tryexpand::expand(["tests/expand/pass/*.rs"]);
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
