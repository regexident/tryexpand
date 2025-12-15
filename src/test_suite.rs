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
    options::Options,
    project::Project,
    test::{Action, Test, TestBehavior, TestPlan, TestStatus},
    TRYEXPAND_ENV_KEY, TRYEXPAND_ENV_VAL_EXPECT, TRYEXPAND_ENV_VAL_OVERWRITE,
};

/// A completed test suite where all tests passed.
///
/// This type is returned by [`ExpandTestSuite::expect_pass`] or [`BuildTestSuite::expect_pass`]
/// after successfully running all tests.
///
/// This is a marker type that serves as proof that the tests completed successfully.
/// It has no public methods and simply indicates successful test execution.
pub struct TestSuitePass {
    #[allow(dead_code)]
    test_suite: TestSuite,
}

/// A completed test suite where all tests failed as expected.
///
/// This type is returned by [`ExpandTestSuite::expect_fail`] or [`BuildTestSuite::expect_fail`]
/// after successfully running all tests that were expected to fail.
///
/// This is a marker type that serves as proof that the tests completed as expected (with failures).
/// It has no public methods and simply indicates successful test execution of failing cases.
pub struct TestSuiteFail {
    #[allow(dead_code)]
    test_suite: TestSuite,
}

/// A test suite builder for macro expansion tests created by [`crate::expand`].
///
/// This type provides methods to configure how tests are run and what happens after expansion.
/// Use the builder pattern to chain configuration methods before calling
/// [`expect_pass`](Self::expect_pass) or [`expect_fail`](Self::expect_fail) to execute the tests.
///
/// # Examples
///
/// ```no_run
/// #[test]
/// fn test_expansion() {
///     tryexpand::expand(["tests/expand/*.rs"])
///         .args(["--features", "test-feature"])
///         .and_check()
///         .expect_pass();
/// }
/// ```
pub struct ExpandTestSuite(pub(crate) TestSuite);

impl ExpandTestSuite {
    /// Adds a single argument to pass to `cargo expand`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::expand(["tests/*.rs"])
    ///     .arg("--verbose")
    ///     .expect_pass();
    /// ```
    pub fn arg<T>(self, arg: T) -> Self
    where
        T: AsRef<str>,
    {
        Self(self.0.arg(arg))
    }

    /// Adds multiple arguments to pass to `cargo expand`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::expand(["tests/*.rs"])
    ///     .args(["--features", "test-feature"])
    ///     .expect_pass();
    /// ```
    pub fn args<T, I>(self, args: I) -> Self
    where
        T: AsRef<str>,
        I: IntoIterator<Item = T>,
    {
        Self(self.0.args(args))
    }

    /// Sets an environment variable for the test execution.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::expand(["tests/*.rs"])
    ///     .env("MY_VAR", "value")
    ///     .expect_pass();
    /// ```
    pub fn env<K, V>(self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Self(self.0.env(key, value))
    }

    /// Sets multiple environment variables for the test execution.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::expand(["tests/*.rs"])
    ///     .envs([("VAR1", "value1"), ("VAR2", "value2")])
    ///     .expect_pass();
    /// ```
    pub fn envs<K, V, I>(self, envs: I) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
        I: IntoIterator<Item = (K, V)>,
    {
        Self(self.0.envs(envs))
    }

    /// Prevents overwriting existing snapshot files even when `TRYEXPAND=overwrite` is set.
    ///
    /// This is useful when you want to preserve specific snapshots while allowing
    /// others to be updated during test runs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::expand(["tests/*.rs"])
    ///     .skip_overwrite()
    ///     .expect_pass();
    /// ```
    pub fn skip_overwrite(self) -> Self {
        Self(self.0.skip_overwrite())
    }

    /// Applies a regex filter to normalize stdout output before snapshotting.
    ///
    /// This is useful for removing non-deterministic content like timestamps or paths
    /// from the expanded output.
    ///
    /// # Panics
    ///
    /// Panics if the regex pattern is invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::expand(["tests/*.rs"])
    ///     .filter_stdout(r"/path/to/\S+", "/path/to/file")
    ///     .expect_pass();
    /// ```
    pub fn filter_stdout<P, R>(self, pattern: P, replacement: R) -> Self
    where
        P: AsRef<str>,
        R: AsRef<str>,
    {
        Self(self.0.filter_stdout(pattern, replacement))
    }

    /// Applies a regex filter to normalize stderr output before snapshotting.
    ///
    /// This is useful for removing non-deterministic content like timestamps or paths
    /// from error messages.
    ///
    /// # Panics
    ///
    /// Panics if the regex pattern is invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::expand(["tests/*.rs"])
    ///     .filter_stderr(r"thread '\S+'", "thread 'test'")
    ///     .expect_pass();
    /// ```
    pub fn filter_stderr<P, R>(self, pattern: P, replacement: R) -> Self
    where
        P: AsRef<str>,
        R: AsRef<str>,
    {
        Self(self.0.filter_stderr(pattern, replacement))
    }

    /// Adds a type-checking step after macro expansion.
    ///
    /// After expanding macros, this will run `cargo check` on the expanded code
    /// and snapshot the compiler output.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::expand(["tests/*.rs"])
    ///     .and_check()
    ///     .expect_pass();
    /// ```
    pub fn and_check(self) -> BuildTestSuite {
        BuildTestSuite(self.0.and_check())
    }

    /// Adds a run step after macro expansion.
    ///
    /// After expanding macros, this will run `cargo run` on the expanded code
    /// and snapshot the program output.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::expand(["tests/*.rs"])
    ///     .and_run()
    ///     .expect_pass();
    /// ```
    pub fn and_run(self) -> BuildTestSuite {
        BuildTestSuite(self.0.and_run())
    }

    /// Adds a test execution step after macro expansion.
    ///
    /// After expanding macros, this will run `cargo test` on the expanded code
    /// and snapshot the test output.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::expand(["tests/*.rs"])
    ///     .and_run_tests()
    ///     .expect_pass();
    /// ```
    pub fn and_run_tests(self) -> BuildTestSuite {
        BuildTestSuite(self.0.and_run_tests())
    }

    /// Expects all tests to pass (macro expansion succeeds).
    ///
    /// This consumes the test suite and executes all tests, expecting successful
    /// macro expansion and matching snapshots.
    ///
    /// # Panics
    ///
    /// Panics if any test fails or if snapshots don't match.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #[test]
    /// fn pass() {
    ///     tryexpand::expand(["tests/pass/*.rs"])
    ///         .expect_pass();
    /// }
    /// ```
    pub fn expect_pass(self) -> TestSuitePass {
        self.0.expect_pass()
    }

    /// Expects all tests to fail (macro expansion produces errors).
    ///
    /// This consumes the test suite and executes all tests, expecting macro expansion
    /// to fail and produce error output that matches the snapshots.
    ///
    /// # Panics
    ///
    /// Panics if any test unexpectedly passes or if error snapshots don't match.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #[test]
    /// fn fail() {
    ///     tryexpand::expand(["tests/fail/*.rs"])
    ///         .expect_fail();
    /// }
    /// ```
    pub fn expect_fail(self) -> TestSuiteFail {
        self.0.expect_fail()
    }
}

/// A test suite builder for build/run/test operations created by
/// [`crate::check`], [`crate::run`], [`crate::run_tests`], or by calling
/// [`ExpandTestSuite::and_check`], [`ExpandTestSuite::and_run`], or
/// [`ExpandTestSuite::and_run_tests`].
///
/// This type provides methods to configure how tests are run before calling
/// [`expect_pass`](Self::expect_pass) or [`expect_fail`](Self::expect_fail) to execute the tests.
///
/// # Examples
///
/// ```no_run
/// #[test]
/// fn test_check() {
///     tryexpand::check(["tests/check/*.rs"])
///         .args(["--features", "test-feature"])
///         .expect_pass();
/// }
/// ```
pub struct BuildTestSuite(pub(crate) TestSuite);

impl BuildTestSuite {
    /// Adds a single argument to pass to the cargo command (`check`, `run`, or `test`).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::check(["tests/*.rs"])
    ///     .arg("--verbose")
    ///     .expect_pass();
    /// ```
    pub fn arg<T>(self, arg: T) -> Self
    where
        T: AsRef<str>,
    {
        Self(self.0.arg(arg))
    }

    /// Adds multiple arguments to pass to the cargo command.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::run(["tests/*.rs"])
    ///     .args(["--features", "test-feature"])
    ///     .expect_pass();
    /// ```
    pub fn args<T, I>(self, args: I) -> Self
    where
        T: AsRef<str>,
        I: IntoIterator<Item = T>,
    {
        Self(self.0.args(args))
    }

    /// Sets an environment variable for the test execution.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::run(["tests/*.rs"])
    ///     .env("MY_VAR", "value")
    ///     .expect_pass();
    /// ```
    pub fn env<K, V>(self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Self(self.0.env(key, value))
    }

    /// Sets multiple environment variables for the test execution.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::run_tests(["tests/*.rs"])
    ///     .envs([("VAR1", "value1"), ("VAR2", "value2")])
    ///     .expect_pass();
    /// ```
    pub fn envs<K, V, I>(self, envs: I) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
        I: IntoIterator<Item = (K, V)>,
    {
        Self(self.0.envs(envs))
    }

    /// Prevents overwriting existing snapshot files even when `TRYEXPAND=overwrite` is set.
    ///
    /// This is useful when you want to preserve specific snapshots while allowing
    /// others to be updated during test runs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::check(["tests/*.rs"])
    ///     .skip_overwrite()
    ///     .expect_pass();
    /// ```
    pub fn skip_overwrite(self) -> Self {
        Self(self.0.skip_overwrite())
    }

    /// Applies a regex filter to normalize stdout output before snapshotting.
    ///
    /// This is useful for removing non-deterministic content like timestamps or paths
    /// from the program output.
    ///
    /// # Panics
    ///
    /// Panics if the regex pattern is invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::run(["tests/*.rs"])
    ///     .filter_stdout(r"\d{4}-\d{2}-\d{2}", "YYYY-MM-DD")
    ///     .expect_pass();
    /// ```
    pub fn filter_stdout<P, R>(self, pattern: P, replacement: R) -> Self
    where
        P: AsRef<str>,
        R: AsRef<str>,
    {
        Self(self.0.filter_stdout(pattern, replacement))
    }

    /// Applies a regex filter to normalize stderr output before snapshotting.
    ///
    /// This is useful for removing non-deterministic content like timestamps or paths
    /// from error messages.
    ///
    /// # Panics
    ///
    /// Panics if the regex pattern is invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// tryexpand::check(["tests/*.rs"])
    ///     .filter_stderr(r"thread '\S+'", "thread 'test'")
    ///     .expect_pass();
    /// ```
    pub fn filter_stderr<P, R>(self, pattern: P, replacement: R) -> Self
    where
        P: AsRef<str>,
        R: AsRef<str>,
    {
        Self(self.0.filter_stderr(pattern, replacement))
    }

    /// Expects all tests to pass.
    ///
    /// This consumes the test suite and executes all tests, expecting the cargo
    /// command to succeed and snapshots to match.
    ///
    /// # Panics
    ///
    /// Panics if any test fails or if snapshots don't match.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #[test]
    /// fn pass() {
    ///     tryexpand::check(["tests/pass/*.rs"])
    ///         .expect_pass();
    /// }
    /// ```
    pub fn expect_pass(self) -> TestSuitePass {
        self.0.expect_pass()
    }

    /// Expects all tests to fail.
    ///
    /// This consumes the test suite and executes all tests, expecting the cargo
    /// command to fail and produce error output that matches the snapshots.
    ///
    /// # Panics
    ///
    /// Panics if any test unexpectedly passes or if error snapshots don't match.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #[test]
    /// fn fail() {
    ///     tryexpand::check(["tests/fail/*.rs"])
    ///         .expect_fail();
    /// }
    /// ```
    pub fn expect_fail(self) -> TestSuiteFail {
        self.0.expect_fail()
    }
}

#[derive(Debug)]
pub(crate) struct TestSuite {
    pub(crate) project: Project,
    pub(crate) plan: TestPlan,
    pub(crate) tests: Vec<Test>,
    pub(crate) options: Options,
    pub(crate) call_site: String,
}

impl TestSuite {
    #[track_caller]
    pub(crate) fn new<I, P>(
        patterns: I,
        action: Action,
        location: &std::panic::Location,
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
                                !path.to_string_lossy().ends_with(crate::OUT_RS_FILE_SUFFIX)
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

        let crate_name = env::var("CARGO_PKG_NAME")
            .ok()
            .or_else(|| metadata.root_package().map(|pkg| pkg.name.to_string()))
            .ok_or(Error::CargoPkgName)?;

        let package = metadata
            .packages
            .iter()
            .find(|package| package.name.as_str() == crate_name)
            .ok_or(Error::CargoPackageNotFound)?;

        let target_dir = env::var("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| metadata.target_directory.as_std_path().to_owned());

        let tests = Self::tests_for(&crate_name, paths);

        let test_suite_id = test_suite_id_from_location(location);

        let project = Project::new(&metadata, package, &test_suite_id, &target_dir, &tests)
            .unwrap_or_else(|err| {
                panic!("Could not create test project: {:#?}", err);
            });

        let plan = TestPlan {
            action,
            post_action: None,
            behavior: test_behavior_from_env().unwrap(),
            expectation: TestStatus::Success,
        };

        let options = Options::default();

        let call_site = format!(
            "{file}:{line}:{column}",
            file = location.file(),
            line = location.line(),
            column = location.column(),
        );

        Ok(Self {
            project,
            plan,
            tests,
            options,
            call_site,
        })
    }

    pub(crate) fn arg<T>(self, arg: T) -> Self
    where
        T: AsRef<str>,
    {
        self.args([arg])
    }

    pub(crate) fn args<T, I>(mut self, args: I) -> Self
    where
        T: AsRef<str>,
        I: IntoIterator<Item = T>,
    {
        self.options
            .args
            .extend(args.into_iter().map(|str| str.as_ref().to_owned()));
        self
    }

    pub(crate) fn env<K, V>(self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.envs([(key, value)])
    }

    pub(crate) fn envs<K, V, I>(mut self, envs: I) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
        I: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in envs.into_iter() {
            let key = key.as_ref().to_owned();
            let value = value.as_ref().to_owned();
            self.options.envs.insert(key, value);
        }
        self
    }

    pub(crate) fn skip_overwrite(mut self) -> Self {
        self.options.skip_overwrite = true;
        self
    }

    pub(crate) fn filter_stdout<P, R>(self, pattern: P, replacement: R) -> Self
    where
        P: AsRef<str>,
        R: AsRef<str>,
    {
        self.add_filter(crate::options::FilterTarget::Stdout, pattern, replacement)
    }

    pub(crate) fn filter_stderr<P, R>(self, pattern: P, replacement: R) -> Self
    where
        P: AsRef<str>,
        R: AsRef<str>,
    {
        self.add_filter(crate::options::FilterTarget::Stderr, pattern, replacement)
    }

    fn add_filter<P, R>(
        mut self,
        target: crate::options::FilterTarget,
        pattern: P,
        replacement: R,
    ) -> Self
    where
        P: AsRef<str>,
        R: AsRef<str>,
    {
        use crate::options::RegexFilter;

        let regex = regex::Regex::new(pattern.as_ref())
            .unwrap_or_else(|e| panic!("Invalid regex pattern '{}': {}", pattern.as_ref(), e));

        self.options.filters.push(RegexFilter {
            target,
            pattern: regex,
            replacement: replacement.as_ref().to_owned(),
        });

        self
    }

    pub(crate) fn and_check(self) -> Self {
        self.and_post_check(Action::Check)
    }

    pub(crate) fn and_run(self) -> Self {
        self.and_post_check(Action::Run)
    }

    pub(crate) fn and_run_tests(self) -> Self {
        self.and_post_check(Action::Test)
    }

    pub(crate) fn expect_pass(self) -> TestSuitePass {
        TestSuitePass {
            test_suite: self.expect_result(TestStatus::Success),
        }
    }

    pub(crate) fn expect_fail(self) -> TestSuiteFail {
        TestSuiteFail {
            test_suite: self.expect_result(TestStatus::Failure),
        }
    }

    fn and_post_check(mut self, action: Action) -> Self {
        if let Some(existing_action) = &self.plan.post_action {
            let cmd = match existing_action {
                Action::Expand => panic!("unexpected `expand` as post-action"),
                Action::Check => "check",
                Action::Test => "test",
                Action::Run => "run",
            };
            panic!("Post-expand action already set to `cargo {cmd}`!");
        }

        self.plan.post_action = Some(action);
        self
    }

    fn expect_result(mut self, expectation: TestStatus) -> Self {
        self.plan.expectation = expectation;
        self
    }

    #[track_caller] // LOAD-BEARING, DO NOT REMOVE!
    pub(crate) fn try_run(&mut self) -> Result<()> {
        let TestSuite {
            project,
            plan,
            tests,
            options,
            call_site,
        } = self;

        let total_tests = tests.len();

        println!(
            "Running {tests} macro expansion tests from {suite} ...\n",
            tests = total_tests,
            suite = call_site
        );

        let mut failures = HashSet::new();

        let max_errors = 2;
        let mut command_errors = 0;

        for test in tests {
            let test_path = test.path.to_owned();
            let result = test.run(plan, project, options, &mut |outcome| {
                message::report_outcome(&test_path, &outcome);

                match outcome.as_status() {
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

            eprintln!();
            panic!("{}", message);
        }

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

impl Drop for TestSuite {
    fn drop(&mut self) {
        match self.try_run() {
            Ok(()) => {}
            Err(err) => {
                panic!("Test suite failed with error: {err:?}");
            }
        }
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
