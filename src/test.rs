use std::path::{Path, PathBuf};

use crate::{
    cargo::{self, Expansion},
    error::Result,
    utils, Options, Project,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum TestBehavior {
    OverwriteFiles,
    ExpectFiles,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum TestEvaluation {
    Success,
    Failure,
}

pub(crate) type TestExpectation = TestEvaluation;
pub(crate) type TestResult = TestEvaluation;

#[derive(Clone, Debug)]
pub(crate) enum TestOutcome {
    SnapshotMatch {
        path: PathBuf,
    },
    SnapshotMismatch {
        path: PathBuf,
        actual: String,
        expected: String,
    },
    SnapshotCreated {
        path: PathBuf,
        after: String,
    },
    SnapshotUpdated {
        path: PathBuf,
        before: String,
        after: String,
    },
    SnapshotExpected {
        path: PathBuf,
        content: String,
    },
    SnapshotUnexpected {
        path: PathBuf,
        content: String,
    },
    UnexpectedSuccess {
        stdout: String,
    },
    UnexpectedFailure {
        stderr: String,
    },
}

impl TestOutcome {
    pub(crate) fn as_result(&self) -> TestResult {
        match self {
            Self::SnapshotMatch { .. } => TestResult::Success,
            Self::SnapshotMismatch { .. } => TestResult::Failure,
            Self::SnapshotCreated { .. } => TestResult::Success,
            Self::SnapshotUpdated { .. } => TestResult::Success,
            Self::SnapshotExpected { .. } => TestResult::Failure,
            Self::SnapshotUnexpected { .. } => TestResult::Failure,
            Self::UnexpectedSuccess { .. } => TestResult::Failure,
            Self::UnexpectedFailure { .. } => TestResult::Failure,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Comparison {
    Match,
    Mismatch,
}

#[derive(Debug)]
pub(crate) struct TestPlan {
    pub behavior: TestBehavior,
    pub expectation: TestExpectation,
}

#[derive(Debug)]
pub(crate) struct Test {
    pub bin: String,
    pub path: PathBuf,
}

impl Test {
    pub fn run(
        &mut self,
        plan: &TestPlan,
        project: &Project,
        options: &Options,
        observe: &mut dyn FnMut(TestOutcome),
    ) -> Result<()> {
        self.expand(plan, project, options, observe).map(|_| ())
    }

    fn expand(
        &mut self,
        plan: &TestPlan,
        project: &Project,
        options: &Options,
        observe: &mut dyn FnMut(TestOutcome),
    ) -> Result<TestEvaluation> {
        let Expansion {
            stdout,
            stderr,
            evaluation,
        } = cargo::expand(project, self, options)?;

        // First we check for unexpected successes/failures and bail out right away:
        match (evaluation, plan.expectation) {
            (TestEvaluation::Success, TestEvaluation::Failure) => {
                let Some(stdout) = stdout.clone() else {
                    return Err(crate::error::Error::UnexpectedEmptyStdOut);
                };
                observe(TestOutcome::UnexpectedSuccess { stdout });
                return Ok(TestEvaluation::Failure);
            }
            (TestEvaluation::Failure, TestEvaluation::Success) => {
                let Some(stderr) = stderr.clone() else {
                    return Err(crate::error::Error::UnexpectedEmptyStdErr);
                };
                observe(TestOutcome::UnexpectedFailure {
                    stderr: stderr.clone(),
                });
                return Ok(TestEvaluation::Failure);
            }
            (_, _) => {}
        }

        let stdout_snapshot_path = self.stdout_snapshot_path();
        let stderr_snapshot_path = self.stderr_snapshot_path();

        let snapshots = match evaluation {
            TestEvaluation::Success => [
                (&stdout_snapshot_path, stdout),
                (&stderr_snapshot_path, None),
            ],
            TestEvaluation::Failure => [
                (&stdout_snapshot_path, stdout),
                (&stderr_snapshot_path, stderr),
            ],
        };

        for (snapshot_path, actual) in snapshots {
            let expected = if snapshot_path.exists() {
                Some(String::from_utf8_lossy(&utils::read(snapshot_path)?).into_owned())
            } else {
                None
            };

            match plan.behavior {
                // We either create snapshots if the user requested so:
                TestBehavior::OverwriteFiles => {
                    self.evaluate_snapshot_overwriting_files(
                        expected,
                        actual,
                        snapshot_path,
                        observe,
                    )?;
                }
                // Or otherwise check for existing snapshots:
                TestBehavior::ExpectFiles => {
                    self.evaluate_snapshot_expecting_files(
                        expected,
                        actual,
                        snapshot_path,
                        observe,
                    )?;
                }
            }
        }

        Ok(evaluation)
    }

    fn evaluate_snapshot_overwriting_files(
        &mut self,
        expected: Option<String>,
        actual: Option<String>,
        snapshot_path: &Path,
        observe: &mut dyn FnMut(TestOutcome),
    ) -> Result<TestEvaluation> {
        let Some(actual) = actual else {
            return Ok(TestEvaluation::Success);
        };

        if let Some(expected) = expected {
            if actual != expected {
                utils::write(snapshot_path, &actual)?;

                observe(TestOutcome::SnapshotUpdated {
                    before: expected.clone(),
                    after: actual.clone(),
                    path: snapshot_path.to_owned(),
                });
            }
        } else {
            utils::write(snapshot_path, &actual)?;

            observe(TestOutcome::SnapshotCreated {
                after: actual.clone(),
                path: snapshot_path.to_owned(),
            });
        }

        Ok(TestEvaluation::Success)
    }

    fn evaluate_snapshot_expecting_files(
        &mut self,
        expected: Option<String>,
        actual: Option<String>,
        snapshot_path: &Path,
        observe: &mut dyn FnMut(TestOutcome),
    ) -> Result<TestEvaluation> {
        match (actual, expected) {
            (None, None) => Ok(TestEvaluation::Success),
            (None, Some(expected)) => {
                observe(TestOutcome::SnapshotUnexpected {
                    content: expected,
                    path: snapshot_path.to_owned(),
                });
                Ok(TestEvaluation::Failure)
            }
            (Some(actual), None) => {
                observe(TestOutcome::SnapshotExpected {
                    content: actual,
                    path: snapshot_path.to_owned(),
                });
                Ok(TestEvaluation::Failure)
            }
            (Some(actual), Some(expected)) => {
                let comparison = Self::compare(&actual, &expected);
                match comparison {
                    Comparison::Match => {
                        observe(TestOutcome::SnapshotMatch {
                            path: snapshot_path.to_owned(),
                        });
                        Ok(TestEvaluation::Success)
                    }
                    Comparison::Mismatch => {
                        observe(TestOutcome::SnapshotMismatch {
                            expected,
                            actual: actual.clone(),
                            path: snapshot_path.to_owned(),
                        });
                        Ok(TestEvaluation::Failure)
                    }
                }
            }
        }
    }

    fn stdout_snapshot_path(&self) -> PathBuf {
        self.path.with_extension(crate::EXPAND_OUT_RS_FILE_SUFFIX)
    }

    fn stderr_snapshot_path(&self) -> PathBuf {
        self.path.with_extension(crate::EXPAND_ERR_TXT_FILE_SUFFIX)
    }

    fn compare(actual: &str, expected: &str) -> Comparison {
        if actual.lines().eq(expected.lines()) {
            Comparison::Match
        } else {
            Comparison::Mismatch
        }
    }
}
