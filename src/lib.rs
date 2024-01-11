//! #### &emsp; Test harness for macro expansion.
//!
//! Similar to [trybuild], but allows you to write tests on how macros are expanded.
//!
//! *Minimal Supported Rust Version: 1.56*
//!
//! <br>
//!
//! # Macro expansion tests
//!
//! A minimal `tryexpand` setup looks like this:
//!
//! ```rust
//! #[test]
//! pub fn pass() {
//!     tryexpand::expand("tests/expand/*.rs");
//! }
//! ```
//!
//! The test can be run with `cargo test`. This test will invoke the [`cargo expand`] command
//! on each of the source files that matches the glob pattern and will compare the expansion result
//! with the corresponding `*.expanded.rs` file.
//!
//! If the environment variable `TRYEXPAND=overwrite` is provided, then `*.expanded.rs` files will
//! be created, or overwritten, if one already exists.
//!
//! Possible test outcomes are:
//! - **Pass**: expansion succeeded and the result is the same as in the `.expanded.rs` file.
//! - **Failure**: expansion is missing or was different from the existing `.expanded.rs` file content.
//!
//! *Note:* when working with multiple expansion test files, it is recommended to
//! specify wildcard (*.rs) instead of doing a multiple calls to `expand` functions for individual files.
//! Usage of wildcards for multiple files will group them under a single temporary crate for which
//! dependencies will be built a single time. In contrast, calling `expand` functions for each
//! source file will create multiple temporary crates and that will reduce performance as depdendencies
//! will be build for each of the temporary crates.
//!
//! ## Passing additional arguments to `cargo expand`
//!
//! It's possible to specify additional arguments for [`cargo expand`] command.
//!
//! In order to do so, use the following functions with `_args` suffix:
//! - [`expand_args`]
//!
//! Example:
//!
//! ```rust
//! pub fn pass() {
//!     tryexpand::expand_args("tests/expand/*.rs", &["--features", "my-feature"]);
//! }
//! ```
//!
//! The `_args` functions will result in the following [`cargo expand`] command being run:
//!
//! ```bash
//! cargo expand --bin <test-name> --theme none --features my-feature
//! ```
//!
//! # Workflow
//!
//! First of all, the [`cargo expand`] tool must be present. You can install it via cargo:
//!
//! ```bash
//! cargo install cargo-expand
//! ```
//!
//! A **nightly** compiler is required for this tool to work, so it must be installed as well.
//!
//! ## Setting up a test project
//!
//! In your crate that provides procedural or declarative macros, under the `tests` directory,
//! create an `expand` directory and populate it with different expansion test cases as
//! rust source files.
//!
//! Then create a `tests.rs` file that will run the tests:
//!
//! ```rust
//! #[test]
//! pub fn pass() {
//!     tryexpand::expand("tests/expand/*.rs");
//! }
//! ```
//!
//! And then you can run `cargo test`, which will
//!
//! 1. Expand macros in source files that match glob pattern
//! 1. In case if [`expand`] function is used:
//!     - On the first run, generate the `*.expanded.rs` files for each of the test cases under
//!     the `expand` directory
//!     - On subsequent runs, compare test cases' expansion result with the
//!     content of the respective `*.expanded.rs` files
//!
//! ## Updating `.expanded.rs`
//!
//! This applicable only to tests that are using [`expand`] or [`expand_args`] function.
//!
//! Run tests with the environment variable `TRYEXPAND=overwrite` or remove the `*.expanded.rs`
//! files and re-run the corresponding tests. Files will be created automatically; hand-writing
//! them is not recommended.
//!
//! [`expand`]: expand/fn.expand.html
//! [`expand_args`]: expand/fn.expand_args.html
//! [trybuild]: https://github.com/dtolnay/trybuild
//! [`cargo expand`]: https://github.com/dtolnay/cargo-expand

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
