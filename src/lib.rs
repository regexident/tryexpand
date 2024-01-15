//! Test harness for macro expansion.

use std::{collections::HashMap, path::Path};

mod cargo;
mod error;
mod manifest;
mod message;
mod normalization;
mod project;
mod test;
mod test_suite;
mod utils;

pub(crate) const TRYEXPAND_ENV_KEY: &str = "TRYEXPAND";
pub(crate) const TRYEXPAND_ENV_VAL_OVERWRITE: &str = "overwrite";
pub(crate) const TRYEXPAND_ENV_VAL_EXPECT: &str = "expect";

pub(crate) const TRYEXPAND_KEEP_ARTIFACTS_ENV_KEY: &str = "TRYEXPAND_KEEP_ARTIFACTS";

pub(crate) const EXPAND_OUT_RS_FILE_SUFFIX: &str = "expand.out.rs";
pub(crate) const EXPAND_ERR_TXT_FILE_SUFFIX: &str = "expand.err.txt";

use crate::{test::TestPlan, test_suite::test_behavior_from_env};

use self::{project::Project, test::TestExpectation};

macro_rules! run_test_suite {
    (
        patterns: $patterns:expr,
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
/// # Refresh behavior
///
/// If no matching `.expand.out.rs` files present, they will be created and result of expansion
/// will be written into them.
///
/// # Panics
///
/// Will panic if matching `.expand.out.rs` file is present, but has different expanded code in it.
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand<I, P>(paths: I)
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    run_test_suite!(
        patterns: paths,
        options: Options::default(),
        expectation: TestExpectation::Success
    )
}

#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand_checked<I, P>(paths: I)
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    run_test_suite!(
        patterns: paths,
        options: Options::default(),
        expectation: TestExpectation::Success
    )
}

/// Attempts to expand macros in files that match glob pattern and expects the expansion to fail.
///
/// # Refresh behavior
///
/// If no matching `.expand.out.rs` files present, they will be created and result (error) of expansion
/// will be written into them.
///
/// # Panics
///
/// Will panic if matching `.expand.out.rs` file is present, but has different expanded code in it.
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand_fail<I, P>(paths: I)
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    run_test_suite!(
        patterns: paths,
        options: Options::default(),
        expectation: TestExpectation::Failure
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
        options: options,
        expectation: TestExpectation::Success
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
        options: options,
        expectation: TestExpectation::Failure
    )
}

#[derive(Clone, Default, Debug)]
pub struct Options {
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

impl Options {
    pub fn args<I, T>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        self.args = Vec::from_iter(args.into_iter().map(|arg| arg.as_ref().to_owned()));
        self
    }

    pub fn env<I, K, V>(mut self, env: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.env = HashMap::from_iter(env.into_iter().map(|(key, value)| {
            let key = key.as_ref().to_owned();
            let value = value.as_ref().to_owned();
            (key, value)
        }));
        self
    }
}
