//! Test harness for macro expansion.

use std::{ffi::OsStr, path::Path};

mod cargo;
mod error;
mod expansion;
mod manifest;
mod message;
mod project;
mod run;
mod rustflags;
mod test;

pub(crate) const TRYEXPAND_ENV_KEY: &str = "TRYEXPAND";
pub(crate) const TRYEXPAND_ENV_VAL_OVERWRITE: &str = "overwrite";
pub(crate) const TRYEXPAND_ENV_VAL_EXPECT: &str = "expect";

use self::{project::Project, run::try_run_tests, test::TestExpectation};

/// Attempts to expand macros in files that match glob pattern.
///
/// # Refresh behavior
///
/// If no matching `.expanded.rs` files present, they will be created and result of expansion
/// will be written into them.
///
/// # Panics
///
/// Will panic if matching `.expanded.rs` file is present, but has different expanded code in it.
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand<I, P>(paths: I)
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    run::run_tests!(paths, Option::<Vec<String>>::None, TestExpectation::Success);
}

/// Attempts to expand macros in files that match glob pattern and expects the expansion to fail.
///
/// # Refresh behavior
///
/// If no matching `.expanded.rs` files present, they will be created and result (error) of expansion
/// will be written into them.
///
/// # Panics
///
/// Will panic if matching `.expanded.rs` file is present, but has different expanded code in it.
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand_fail<I, P>(paths: I)
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    run::run_tests!(paths, Option::<Vec<String>>::None, TestExpectation::Failure);
}

/// Same as [`expand`] but allows to pass additional arguments to `cargo-expand`.
///
/// [`expand`]: expand/fn.expand.html
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand_args<Ip, P, Ia, A>(paths: Ip, args: Ia)
where
    Ip: IntoIterator<Item = P>,
    P: AsRef<Path>,
    Ia: IntoIterator<Item = A> + Clone,
    A: AsRef<OsStr>,
{
    run::run_tests!(paths, Some(args), TestExpectation::Success);
}

/// Same as [`expand_fail`] but allows to pass additional arguments to `cargo-expand`.
///
/// [`expand_fail`]: expand/fn.expand_fail.html
#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub fn expand_args_fail<Ip, P, Ia, A>(paths: Ip, args: Ia)
where
    Ip: IntoIterator<Item = P>,
    P: AsRef<Path>,
    Ia: IntoIterator<Item = A> + Clone,
    A: AsRef<OsStr>,
{
    run::run_tests!(paths, Some(args), TestExpectation::Failure);
}
