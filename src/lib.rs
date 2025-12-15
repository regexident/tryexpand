//! Test harness for macro expansion.

#![warn(missing_docs)]
#![allow(clippy::test_attr_in_doctest)]

use std::{panic::Location, path::Path};

mod cargo;
mod error;
mod manifest;
mod message;
mod normalization;
mod options;
mod project;
mod test;
mod test_suite;
mod utils;

pub(crate) const TRYEXPAND_ENV_KEY: &str = "TRYEXPAND";
pub(crate) const TRYEXPAND_ENV_VAL_OVERWRITE: &str = "overwrite";
pub(crate) const TRYEXPAND_ENV_VAL_EXPECT: &str = "expect";

pub(crate) const TRYEXPAND_KEEP_ARTIFACTS_ENV_KEY: &str = "TRYEXPAND_KEEP_ARTIFACTS";
pub(crate) const TRYEXPAND_TRUNCATE_OUTPUT_ENV_KEY: &str = "TRYEXPAND_TRUNCATE_OUTPUT";
pub(crate) const TRYEXPAND_DEBUG_LOG_ENV_KEY: &str = "TRYEXPAND_DEBUG_LOG";

pub(crate) const OUT_RS_FILE_SUFFIX: &str = "out.rs";
pub(crate) const OUT_TXT_FILE_SUFFIX: &str = "out.txt";
pub(crate) const ERR_TXT_FILE_SUFFIX: &str = "err.txt";

use crate::{
    test::Action,
    test_suite::{BuildTestSuite, ExpandTestSuite, TestSuite},
};

/// Run snapshot tests on files that match the provided paths/glob patterns,
/// snapshotting the source code as it is produced by `cargo expand`.
///
/// # Examples
///
/// Simple:
///
/// ```
/// #[test]
/// pub fn pass() {
///     tryexpand::expand(
///         ["tests/expand/pass/*.rs"]
///     ).expect_pass();
/// }
///
/// #[test]
/// pub fn fail() {
///     tryexpand::expand(
///         ["tests/expand/fail/*.rs"]
///     ).expect_fail();
/// }
/// ```
///
/// Advanced:
///
/// ```
/// #[test]
/// pub fn pass() {
///     tryexpand::expand(
///         [
///             "tests/expand/foo/pass/*.rs",
///             "tests/expand/bar/pass/*.rs"
///         ]
///     )
///     .args(["--features", "test-feature"])
///     .envs([("MY_ENV", "my env var value")])
///     .and_check() // also type-check the code on successful macro-expansion
///     .expect_pass();
/// }
/// ```
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand<I, P>(patterns: I) -> ExpandTestSuite
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    match TestSuite::new(patterns, Action::Expand, Location::caller()) {
        Ok(test_suite) => ExpandTestSuite(test_suite),
        Err(err) => panic!("Error: {err:?}"),
    }
}

/// Run snapshot tests on files that match the provided paths/glob patterns,
/// snapshotting the stdout/stderr output as it is produced by `cargo check [ARGS]`.
///
/// # Examples
///
/// Simple:
///
/// ```
/// #[test]
/// pub fn pass() {
///     tryexpand::check(
///         ["tests/expand/pass/*.rs"]
///     ).expect_pass();
/// }
///
/// #[test]
/// pub fn fail() {
///     tryexpand::check(
///         ["tests/expand/fail/*.rs"]
///     ).expect_fail();
/// }
/// ```
///
/// Advanced:
///
/// ```
/// #[test]
/// pub fn pass() {
///     tryexpand::check(
///         [
///             "tests/expand/foo/pass/*.rs",
///             "tests/expand/bar/pass/*.rs"
///         ]
///     )
///     .args(["--features", "test-feature"])
///     .envs([("MY_ENV", "my env var value")])
///     .expect_pass();
/// }
/// ```
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn check<I, P>(patterns: I) -> BuildTestSuite
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    match TestSuite::new(patterns, Action::Check, Location::caller()) {
        Ok(test_suite) => BuildTestSuite(test_suite),
        Err(err) => panic!("Error: {err:?}"),
    }
}

/// Run snapshot tests on files that match the provided paths/glob patterns,
/// snapshotting the stdout/stderr output as it is produced by `cargo run [ARGS]`.
///
/// # Examples
///
/// Simple:
///
/// ```
/// #[test]
/// pub fn pass() {
///     tryexpand::run(
///         ["tests/expand/pass/*.rs"]
///     ).expect_pass();
/// }
///
/// #[test]
/// pub fn fail() {
///     tryexpand::run(
///         ["tests/expand/fail/*.rs"]
///     ).expect_fail();
/// }
/// ```
///
/// Advanced:
///
/// ```
/// #[test]
/// pub fn pass() {
///     tryexpand::run(
///         [
///             "tests/expand/foo/pass/*.rs",
///             "tests/expand/bar/pass/*.rs"
///         ]
///     )
///     .args(["--features", "test-feature"])
///     .envs([("MY_ENV", "my env var value")])
///     .expect_pass();
/// }
/// ```
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn run<I, P>(patterns: I) -> BuildTestSuite
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    match TestSuite::new(patterns, Action::Run, Location::caller()) {
        Ok(test_suite) => BuildTestSuite(test_suite),
        Err(err) => panic!("Error: {err:?}"),
    }
}

/// Run snapshot tests on files that match the provided paths/glob patterns,
/// snapshotting the stdout/stderr output as it is produced by `cargo test [ARGS]`.
///
/// # Examples
///
/// Simple:
///
/// ```
/// #[test]
/// pub fn pass() {
///     tryexpand::run_tests(
///         ["tests/expand/pass/*.rs"]
///     ).expect_pass();
/// }
///
/// #[test]
/// pub fn fail() {
///     tryexpand::run_tests(
///         ["tests/expand/fail/*.rs"]
///     ).expect_fail();
/// }
/// ```
///
/// Advanced:
///
/// ```
/// #[test]
/// pub fn pass() {
///     tryexpand::run_tests(
///         [
///             "tests/expand/foo/pass/*.rs",
///             "tests/expand/bar/pass/*.rs"
///         ]
///     )
///     .args(["--features", "test-feature"])
///     .envs([("MY_ENV", "my env var value")])
///     .expect_pass();
/// }
/// ```
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn run_tests<I, P>(patterns: I) -> BuildTestSuite
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    match TestSuite::new(patterns, Action::Test, Location::caller()) {
        Ok(test_suite) => BuildTestSuite(test_suite),
        Err(err) => panic!("Error: {err:?}"),
    }
}
