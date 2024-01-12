use std::{ffi::OsStr, fs, path::PathBuf};

use crate::{
    cargo::{self, Expansion},
    error::Result,
    Project,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum TestBehavior {
    OverwriteFiles,
    ExpectFiles,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum TestExpectation {
    Success,
    Failure,
}

#[derive(Clone, Debug)]
pub(crate) enum TestResult {
    Success,
    Failure,
}

#[derive(Clone, Debug)]
pub(crate) enum TestOutcome {
    SnapshotMatch,
    SnapshotMismatch { actual: String, expected: String },
    SnapshotCreated { after: String },
    SnapshotUpdated { before: String, after: String },
    SnapshotMissing,
    UnexpectedSuccess { output: String },
    UnexpectedFailure { output: String },
    CommandFailure { output: String },
}

impl TestOutcome {
    pub(crate) fn as_result(&self) -> TestResult {
        match self {
            Self::SnapshotMatch => TestResult::Success,
            Self::SnapshotMismatch { .. } => TestResult::Failure,
            Self::SnapshotCreated { .. } => TestResult::Success,
            Self::SnapshotUpdated { .. } => TestResult::Success,
            Self::SnapshotMissing => TestResult::Failure,
            Self::UnexpectedSuccess { .. } => TestResult::Failure,
            Self::UnexpectedFailure { .. } => TestResult::Failure,
            Self::CommandFailure { .. } => TestResult::Failure,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum ComparisonOutcome {
    Match,
    Mismatch,
}

#[derive(Debug)]
pub(crate) struct Test {
    pub bin: String,
    pub path: PathBuf,
    pub expanded_path: PathBuf,
}

impl Test {
    pub fn run<I, S>(
        &self,
        project: &Project,
        args: &Option<I>,
        behavior: TestBehavior,
        expectation: TestExpectation,
    ) -> Result<TestOutcome>
    where
        I: IntoIterator<Item = S> + Clone,
        S: AsRef<OsStr>,
    {
        let expanded_path = self.expanded_path.as_path();

        let expansion = cargo::expand(project, self, args)?;

        let actual = match &expansion {
            cargo::Expansion::Success { stdout } => stdout,
            cargo::Expansion::Failure { stderr } => stderr,
        };

        if let (Expansion::Failure { .. }, TestExpectation::Success) = (&expansion, expectation) {
            return Ok(TestOutcome::UnexpectedFailure {
                output: actual.clone(),
            });
        }

        if let (Expansion::Success { .. }, TestExpectation::Failure) = (&expansion, expectation) {
            return Ok(TestOutcome::UnexpectedSuccess {
                output: actual.clone(),
            });
        }

        if !expanded_path.exists() {
            match behavior {
                TestBehavior::OverwriteFiles => {
                    // Write a .expanded.rs file contents with an newline character at the end
                    fs::write(expanded_path, actual)?;

                    return Ok(TestOutcome::SnapshotCreated {
                        after: actual.clone(),
                    });
                }
                TestBehavior::ExpectFiles => return Ok(TestOutcome::SnapshotMissing),
            }
        }

        let expected = String::from_utf8_lossy(&fs::read(expanded_path)?).into_owned();

        let outcome = match Self::compare(actual, &expected) {
            ComparisonOutcome::Match => Ok(TestOutcome::SnapshotMatch),
            ComparisonOutcome::Mismatch => {
                match behavior {
                    TestBehavior::OverwriteFiles => {
                        // Write a .expanded.rs file contents with an newline character at the end
                        fs::write(expanded_path, actual)?;

                        Ok(TestOutcome::SnapshotUpdated {
                            before: expected.clone(),
                            after: actual.clone(),
                        })
                    }
                    TestBehavior::ExpectFiles => Ok(TestOutcome::SnapshotMismatch {
                        expected,
                        actual: actual.clone(),
                    }),
                }
            }
        }
        .unwrap_or_else(|err| TestOutcome::CommandFailure { output: err });

        Ok(outcome)
    }

    fn compare(actual: &str, expected: &str) -> ComparisonOutcome {
        if actual.lines().eq(expected.lines()) {
            ComparisonOutcome::Match
        } else {
            ComparisonOutcome::Mismatch
        }
    }
}
