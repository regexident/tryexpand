use std::{fs, path::PathBuf};

use crate::{
    cargo::{self, Expansion},
    error::Result,
    Options, Project,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum TestBehavior {
    OverwriteFiles,
    ExpectFiles,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum Evaluation {
    Success,
    Failure,
}

pub(crate) type TestExpectation = Evaluation;
pub(crate) type TestResult = Evaluation;

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
pub(crate) struct Test {
    pub bin: String,
    pub path: PathBuf,
}

impl Test {
    pub fn run(
        &self,
        project: &Project,
        args: &Option<Options>,
        behavior: TestBehavior,
        expectation: TestExpectation,
        observe: &mut dyn FnMut(TestOutcome),
    ) -> Result<()> {
        let Expansion {
            stdout,
            stderr,
            evaluation,
        } = cargo::expand(project, self, args)?;

        // First we check for unexpected successes/failures and bail out right away:
        match (evaluation, expectation) {
            (Evaluation::Success, Evaluation::Failure) => {
                let Some(stdout) = stdout.clone() else {
                    return Err(crate::error::Error::UnexpectedEmptyStdOut);
                };
                observe(TestOutcome::UnexpectedSuccess { stdout });
                return Ok(());
            }
            (Evaluation::Failure, Evaluation::Success) => {
                let Some(stderr) = stderr.clone() else {
                    return Err(crate::error::Error::UnexpectedEmptyStdErr);
                };
                observe(TestOutcome::UnexpectedFailure {
                    stderr: stderr.clone(),
                });
                return Ok(());
            }
            (_, _) => {}
        }

        let stdout_snapshot_path = self.stdout_snapshot_path();
        let stderr_snapshot_path = self.stderr_snapshot_path();

        let snapshots = match evaluation {
            Evaluation::Success => [
                (&stdout_snapshot_path, stdout),
                (&stderr_snapshot_path, None),
            ],
            Evaluation::Failure => [
                (&stdout_snapshot_path, stdout),
                (&stderr_snapshot_path, stderr),
            ],
        };

        for (snapshot_path, current) in snapshots {
            let existing = if snapshot_path.exists() {
                Some(String::from_utf8_lossy(&fs::read(snapshot_path)?).into_owned())
            } else {
                None
            };

            match behavior {
                // We either create snapshots if the user requested so:
                TestBehavior::OverwriteFiles => {
                    let Some(actual) = current else {
                        continue;
                    };

                    if let Some(expected) = existing {
                        if actual != expected {
                            fs::write(snapshot_path, &actual)?;

                            observe(TestOutcome::SnapshotUpdated {
                                before: expected.clone(),
                                after: actual.clone(),
                                path: snapshot_path.clone(),
                            });
                        }
                    } else {
                        fs::write(snapshot_path, &actual)?;

                        observe(TestOutcome::SnapshotCreated {
                            after: actual.clone(),
                            path: snapshot_path.clone(),
                        });
                    }
                }
                // Or otherwise check for existing snapshots:
                TestBehavior::ExpectFiles => match (current, existing) {
                    (None, None) => continue,
                    (None, Some(expected)) => {
                        observe(TestOutcome::SnapshotUnexpected {
                            content: expected,
                            path: snapshot_path.clone(),
                        });
                    }
                    (Some(actual), None) => {
                        observe(TestOutcome::SnapshotExpected {
                            content: actual,
                            path: snapshot_path.clone(),
                        });
                    }
                    (Some(actual), Some(expected)) => {
                        let comparison = Self::compare(&actual, &expected);
                        match comparison {
                            Comparison::Match => {
                                observe(TestOutcome::SnapshotMatch {
                                    path: snapshot_path.clone(),
                                });
                            }
                            Comparison::Mismatch => {
                                observe(TestOutcome::SnapshotMismatch {
                                    expected,
                                    actual: actual.clone(),
                                    path: snapshot_path.clone(),
                                });
                            }
                        }
                    }
                },
            }
        }

        Ok(())
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
