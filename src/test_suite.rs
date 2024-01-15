use core::panic;
use std::{
    collections::hash_map::DefaultHasher,
    collections::HashSet,
    env,
    fmt::Write,
    hash::{Hash as _, Hasher as _},
    iter::FromIterator,
    path::{Path, PathBuf},
};

use crate::{
    error::{Error, Result},
    message,
    project::Project,
    test::{Test, TestBehavior, TestPlan, TestStatus},
    Options, TRYEXPAND_ENV_KEY, TRYEXPAND_ENV_VAL_EXPECT, TRYEXPAND_ENV_VAL_OVERWRITE,
};

#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub(crate) fn try_run_tests<I, P>(
    location: &std::panic::Location,
    patterns: I,
    options: Options,
    plan: TestPlan,
) -> Result<()>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let test_suite_id = test_suite_id_from_location(location);

    let test_suite = TestSuite::new(patterns, plan, options, &test_suite_id)?;

    test_suite.try_run()?;

    Ok(())
}

#[derive(Debug)]
pub(crate) struct TestSuite {
    pub project: Project,
    pub plan: TestPlan,
    pub tests: Vec<Test>,
    pub options: Options,
}

impl TestSuite {
    #[track_caller] // LOAD-BEARING, DO NOT REMOVE!
    pub(crate) fn new<I, P>(
        patterns: I,
        plan: TestPlan,
        options: Options,
        test_suite_id: &str,
    ) -> Result<Self>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let patterns = Vec::from_iter(patterns);

        if patterns.is_empty() {
            panic!("no file patterns provided");
        }

        let paths_per_pattern: Vec<(PathBuf, Vec<PathBuf>)> = patterns
            .into_iter()
            .map(|pattern| {
                expand_globs(&pattern).map(|paths| {
                    (
                        pattern.as_ref().to_owned(),
                        paths
                            .into_iter()
                            .filter(|path| {
                                !path
                                    .to_string_lossy()
                                    .ends_with(crate::EXPAND_OUT_RS_FILE_SUFFIX)
                            })
                            .collect(),
                    )
                })
            })
            .collect::<Result<_>>()?;

        let (without_matches, with_matches): (Vec<_>, Vec<_>) = paths_per_pattern
            .iter()
            .partition(|(_, paths)| paths.is_empty());

        if !without_matches.is_empty() {
            let unique_patterns: HashSet<&PathBuf> = without_matches
                .into_iter()
                .map(|(pattern, _)| pattern)
                .collect();

            let sorted_patterns = Vec::from_iter(unique_patterns);

            let mut error = String::new();
            writeln!(&mut error, "no matching files found for:").unwrap();
            for pattern in sorted_patterns {
                writeln!(&mut error, "    {}", pattern.display()).unwrap();
            }

            panic!("{}", error);
        }

        let unique_paths: HashSet<PathBuf> = with_matches
            .into_iter()
            .flat_map(|(_, paths)| paths.clone())
            .collect();

        let paths: Vec<PathBuf> = Vec::from_iter(unique_paths);

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

        let tests = Self::tests_for(&crate_name, paths);

        let project =
            Project::new(package, test_suite_id, &target_dir, &tests).unwrap_or_else(|err| {
                panic!("Could not create test project: {:#?}", err);
            });

        Ok(Self {
            project,
            plan,
            tests,
            options,
        })
    }

    #[track_caller] // LOAD-BEARING, DO NOT REMOVE!
    pub(crate) fn try_run(self) -> Result<()> {
        let TestSuite {
            project,
            plan,
            tests,
            options,
        } = self;

        let total_tests = tests.len();

        println!("Running {} macro expansion tests ...!", total_tests);

        let mut failures = HashSet::new();

        let max_errors = 2;
        let mut command_errors = 0;

        for mut test in tests {
            let test_path = test.path.to_owned();
            let result = test.run(&plan, &project, &options, &mut |outcome| {
                message::report_outcome(&test_path, &outcome);

                match outcome.as_result() {
                    TestStatus::Success => {}
                    TestStatus::Failure => {
                        failures.insert(test_path.clone());
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
            writeln!(
                &mut message,
                "{} of {} tests failed:",
                failures.len(),
                total_tests
            )
            .unwrap();
            writeln!(&mut message).unwrap();

            let mut sorted_failures = Vec::from_iter(failures);
            sorted_failures.sort();

            for failure in sorted_failures {
                writeln!(&mut message, "    {}", failure.display()).unwrap();
            }

            panic!("{}", message);
        }

        // Ensure the project (and with it its corresponding on-dist project)
        // doesn't get dropped prematurely, unintentionally:
        drop(project);

        Ok(())
    }

    fn tests_for<I, P>(crate_name: &str, paths: I) -> Vec<Test>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut tests: Vec<_> = paths
            .into_iter()
            .map(|path| {
                let path = path.as_ref().to_path_buf();
                let bin = {
                    let mut hasher = DefaultHasher::default();
                    path.hash(&mut hasher);
                    // Taking the lower-case of a base62 hash leads to collisions
                    // but the number of tests we're dealing with shouldn't
                    // realistically cause any issues in practice:
                    let test_id = base62::encode(hasher.finish()).to_lowercase();
                    format!("{crate_name}_{test_id}")
                };
                Test { bin, path }
            })
            .collect();

        tests.sort_by_cached_key(|test| test.path.clone());

        tests
    }
}

fn test_suite_id_from_location(caller_location: &std::panic::Location) -> String {
    use std::hash::{Hash as _, Hasher as _};
    let mut hasher = ::std::collections::hash_map::DefaultHasher::default();
    caller_location.file().hash(&mut hasher);
    caller_location.line().hash(&mut hasher);
    caller_location.column().hash(&mut hasher);
    // Taking the lower-case of a base62 hash leads to collisions
    // but the number of tests we're dealing with shouldn't
    // realistically cause any issues in practice:
    base62::encode(hasher.finish()).to_lowercase()
}

pub(crate) fn test_behavior_from_env() -> Result<TestBehavior> {
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

fn expand_globs(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
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
