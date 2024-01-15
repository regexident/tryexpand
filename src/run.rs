use core::panic;
use std::{
    collections::HashSet,
    env,
    fmt::Write,
    iter::FromIterator,
    path::{Path, PathBuf},
};

use crate::{
    error::{Error, Result},
    message,
    project::Project,
    test::{TestBehavior, TestExpectation, TestResult},
    Options, TRYEXPAND_ENV_KEY, TRYEXPAND_ENV_VAL_EXPECT, TRYEXPAND_ENV_VAL_OVERWRITE,
};

pub(crate) type TestSuiteExpectation = TestExpectation;

macro_rules! run_tests {
    ($paths:expr, $args:expr, $expectation:expr) => {
        use std::hash::{Hash as _, Hasher as _};

        // IMPORTANT: This only works as lone as all functions between
        // the public API and this call are marked with `#[track_caller]`:
        let caller_location = ::std::panic::Location::caller();

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
pub(crate) fn try_run_tests<I, P>(
    patterns: I,
    args: Option<Options>,
    test_suite_id: &str,
    expectation: TestSuiteExpectation,
) -> Result<()>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let patterns = Vec::from_iter(patterns);

    if patterns.is_empty() {
        panic!("no file patterns provided");
    }

    let unique_paths: HashSet<PathBuf> = patterns
        .into_iter()
        .filter_map(|path| expand_globs(path).ok())
        .flatten()
        .filter(|path| {
            !path
                .to_string_lossy()
                .ends_with(crate::EXPAND_OUT_RS_FILE_SUFFIX)
        })
        .collect();

    let paths: Vec<PathBuf> = Vec::from_iter(unique_paths);
    let len = paths.len();

    let metadata = cargo_metadata::MetadataCommand::new()
        .exec()
        .map_err(|source| Error::CargoMetadata {
            directory: std::env::current_dir().unwrap(),
            source,
        })?;

    let crate_name = env::var("CARGO_PKG_NAME").map_err(|_| Error::CargoPkgName)?;

    let package = metadata
        .packages
        .iter()
        .find(|package| package.name == crate_name)
        .ok_or_else(|| Error::CargoPackageNotFound)?;

    let target_dir = metadata.target_directory.as_std_path().to_owned();

    let project = Project::new(package, test_suite_id, &target_dir, paths).unwrap_or_else(|err| {
        panic!("Could not create test project: {:#?}", err);
    });

    let behavior = test_behavior()?;

    println!("Running {} macro expansion tests ...!", len);

    let mut failures = HashSet::new();

    let max_errors = 2;
    let mut command_errors = 0;

    for test in &project.tests {
        let result = test.run(&project, &args, behavior, expectation, &mut |outcome| {
            message::report_outcome(&test.path, &outcome);

            match outcome.as_result() {
                TestResult::Success => {}
                TestResult::Failure => {
                    failures.insert(test.path.to_owned());
                }
            }
        });

        if let Err(err) = result {
            let error = err.to_string();
            message::command_failure(&test.path, &error);
            command_errors += 1;

            if command_errors > max_errors {
                message::command_abortion(command_errors);
            }
        }
    }

    if !failures.is_empty() {
        let mut message = String::new();

        writeln!(&mut message).unwrap();
        writeln!(&mut message, "{} of {} tests failed:", failures.len(), len).unwrap();
        writeln!(&mut message).unwrap();

        let mut sorted_failures = Vec::from_iter(failures);
        sorted_failures.sort();

        for failure in sorted_failures {
            writeln!(&mut message, "    {}", failure.display()).unwrap();
        }

        panic!("{}", message);
    }

    drop(project);

    Ok(())
}

fn test_behavior() -> Result<TestBehavior> {
    let key = TRYEXPAND_ENV_KEY;
    let Some(var) = std::env::var_os(key) else {
        return Ok(TestBehavior::ExpectFiles);
    };
    let value = var.to_string_lossy().into_owned();
    match value.as_str() {
        TRYEXPAND_ENV_VAL_EXPECT => Ok(TestBehavior::ExpectFiles),
        TRYEXPAND_ENV_VAL_OVERWRITE => Ok(TestBehavior::OverwriteFiles),
        _ => Err(Error::UnrecognizedEnv {
            key: key.to_owned(),
            value,
        }),
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
