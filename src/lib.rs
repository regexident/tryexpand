//! Test harness for macro expansion.

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

pub use self::options::Options;

use crate::test_suite::TestSuite;

/// Attempts to expand macros in files that match glob pattern, expecting a pass.
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
