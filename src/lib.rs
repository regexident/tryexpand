//! Test harness for macro expansion.

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

pub(crate) const OUT_RS_FILE_SUFFIX: &str = "out.rs";
pub(crate) const OUT_TXT_FILE_SUFFIX: &str = "out.txt";
pub(crate) const ERR_TXT_FILE_SUFFIX: &str = "err.txt";

use crate::test_suite::TestSuite;

/// Attempts to expand macros in files that match the provided paths/glob patterns.
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
///     .and_check() // type-check the expanded code
///     .expect_pass();
/// }
/// ```
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand<I, P>(patterns: I) -> TestSuite
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    match TestSuite::new(patterns, Location::caller()) {
        Ok(test_suite) => test_suite,
        Err(err) => panic!("Error: {err:?}"),
    }
}
