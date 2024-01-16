//! Test harness for macro expansion.

use std::path::Path;

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
pub(crate) const ERR_TXT_FILE_SUFFIX: &str = "err.txt";

pub use self::options::Options;

use crate::{
    test::{TestAction, TestPlan},
    test_suite::test_behavior_from_env,
};

use self::{project::Project, test::TestStatus};

macro_rules! run_test_suite {
    (
        patterns: $patterns:expr,
        action: $action:expr,
        options: $options:expr,
        expectation: $expectation:expr
    ) => {{
        // IMPORTANT: This only works as lone as all functions between
        // the public API and this call are marked with `#[track_caller]`:
        let location = ::std::panic::Location::caller();

        let fallible_block = || {
            $crate::test_suite::try_run_tests(
                location,
                $patterns,
                $options,
                TestPlan {
                    action: $action,
                    behavior: test_behavior_from_env()?,
                    expectation: $expectation,
                },
            )
        };

        match fallible_block() {
            Ok(()) => {}
            Err(err) => panic!("{}", err),
        }
    }};
}

/// Attempts to expand macros in files that match glob pattern.
///
/// # Panics
///
/// Will panic if matching `.out.rs` file is missing, or present but has different expanded code in it.
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand<I, P>(paths: I)
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    run_test_suite!(
        patterns: paths,
        action: TestAction::Expand,
        options: Options::default(),
        expectation: TestStatus::Success
    )
}

/// Attempts to expand macros in files that match glob pattern, as well as check their expansion.
///
/// # Panics
///
/// Will panic if matching `.out.rs` file is missing, or present but has different expanded code in it.
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand_checking<I, P>(paths: I)
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    run_test_suite!(
        patterns: paths,
        action: TestAction::ExpandAndCheck,
        options: Options::default(),
        expectation: TestStatus::Success
    )
}

/// Attempts to expand macros in files that match glob pattern and expects the expansion to fail.
///
/// # Panics
///
/// Will panic if matching `.out.rs` file is missing, or present but has different expanded code in it.
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand_fail<I, P>(paths: I)
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    run_test_suite!(
        patterns: paths,
        action: TestAction::Expand,
        options: Options::default(),
        expectation: TestStatus::Failure
    )
}

/// Same as [`expand`] but allows to pass additional arguments to `cargo-expand`.
///
/// [`expand`]: expand/fn.expand.html
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand_opts<I, P>(paths: I, options: Options)
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    run_test_suite!(
        patterns: paths,
        action: TestAction::Expand,
        options: options,
        expectation: TestStatus::Success
    )
}

/// Same as [`expand_checking`] but allows to pass additional arguments to `cargo-expand`.
///
/// [`expand_checking`]: expand/fn.expand_checking.html
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand_opts_checking<I, P>(paths: I, options: Options)
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    run_test_suite!(
        patterns: paths,
        action: TestAction::ExpandAndCheck,
        options: options,
        expectation: TestStatus::Success
    )
}

/// Same as [`expand_fail`] but allows to pass additional arguments to `cargo-expand`.
///
/// [`expand_fail`]: expand/fn.expand_fail.html
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand_opts_fail<I, P>(paths: I, options: Options)
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    run_test_suite!(
        patterns: paths,
        action: TestAction::Expand,
        options: options,
        expectation: TestStatus::Failure
    )
}
