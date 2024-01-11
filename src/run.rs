use core::panic;
use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    fmt::Write,
    iter::FromIterator,
    path::{Path, PathBuf},
};

use crate::{
    error::{Error, Result},
    message,
    project::{setup_project, teardown_project},
    test::{TestBehavior, TestExpectation, TestResult},
    TRYEXPAND_ENV_KEY, TRYEXPAND_ENV_VAL_EXPECT, TRYEXPAND_ENV_VAL_OVERWRITE,
};

pub(crate) type TestSuiteExpectation = TestExpectation;

/// An extension for files containing `cargo expand` result.
const EXPANDED_RS_SUFFIX: &str = "expanded.rs";

macro_rules! run_tests {
    ($paths:expr, $args:expr, $expectation:expr) => {
        // IMPORTANT: This only works as lone as all functions between
        // the public API and this call are marked with `#[track_caller]`:
        let caller_location = ::std::panic::Location::caller();

        use std::hash::{Hash as _, Hasher as _};

        let mut hasher = ::std::collections::hash_map::DefaultHasher::default();
        caller_location.file().hash(&mut hasher);
        caller_location.line().hash(&mut hasher);
        caller_location.column().hash(&mut hasher);
        // Taking the lower-case of a base62 hash leads to collisions
        // but the number of tests we're dealing with shouldn't
        // realistically cause any issues in practice:
        let test_suite_id = ::base62::encode(hasher.finish()).to_lowercase();

        match $crate::try_run_tests($paths, $args, &test_suite_id, $expectation) {
            Ok(()) => {}
            Err(err) => panic!("{}", err),
        }
    };
}

pub(crate) use run_tests;

#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub(crate) fn try_run_tests<Ip, P, Ia, A>(
    paths: Ip,
    args: Option<Ia>,
    test_suite_id: &str,
    expectation: TestSuiteExpectation,
) -> Result<()>
where
    Ip: IntoIterator<Item = P>,
    P: AsRef<Path>,
    Ia: IntoIterator<Item = A> + Clone,
    A: AsRef<OsStr>,
{
    let unique_paths: HashSet<PathBuf> = paths
        .into_iter()
        .filter_map(|path| expand_globs(path).ok())
        .flatten()
        .filter(|path| !path.to_string_lossy().ends_with(EXPANDED_RS_SUFFIX))
        .collect();

    let paths: Vec<PathBuf> = Vec::from_iter(unique_paths);
    let len = paths.len();

    let crate_name = env::var("CARGO_PKG_NAME").map_err(|_| Error::CargoPkgName)?;

    let project = setup_project(&crate_name, test_suite_id, paths).unwrap_or_else(|err| {
        panic!("prepare failed: {:#?}", err);
    });

    let _guard = scopeguard::guard((), |_| {
        let _ = teardown_project(project.dir.clone());
    });

    let behavior = test_behavior()?;

    println!("Running {} macro expansion tests ...!", len);

    let mut failures = vec![];

    for test in &project.tests {
        // let path = test.path.as_path();
        // let expanded_path = test.expanded_path.as_path();

        let outcome = test.run(&project, &args, behavior, expectation)?;

        message::report_outcome(&test.path, &test.expanded_path, &outcome);

        match outcome.as_result() {
            TestResult::Success => {}
            TestResult::Failure => {
                failures.push(test.path.to_owned());
            }
        }
    }

    if !failures.is_empty() {
        let mut message = String::new();

        writeln!(&mut message).unwrap();
        writeln!(&mut message, "{} of {} tests failed:", failures.len(), len).unwrap();
        writeln!(&mut message).unwrap();

        for failure in failures {
            writeln!(&mut message, "    {}", failure.display()).unwrap();
        }

        panic!("{}", message);
    }

    Ok(())
}

fn test_behavior() -> Result<TestBehavior> {
    let Some(var) = std::env::var_os(TRYEXPAND_ENV_KEY) else {
        return Ok(TestBehavior::ExpectFiles);
    };

    match var.as_os_str().to_string_lossy().as_ref() {
        TRYEXPAND_ENV_VAL_EXPECT => Ok(TestBehavior::ExpectFiles),
        TRYEXPAND_ENV_VAL_OVERWRITE => Ok(TestBehavior::OverwriteFiles),
        _ => Err(Error::UnrecognizedEnv(var)),
    }
}

pub(crate) fn expand_globs(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let path = path.as_ref();

    fn glob(pattern: &str) -> Result<Vec<PathBuf>> {
        let mut paths = glob::glob(pattern)?
            .map(|entry| entry.map_err(Error::from))
            .collect::<Result<Vec<PathBuf>>>()?;
        paths.sort();
        Ok(paths)
    }

    let path_string = path.as_os_str().to_string_lossy();

    Ok(glob(&path_string)?.into_iter().collect())
}
