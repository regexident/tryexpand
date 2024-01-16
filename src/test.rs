use std::path::{Path, PathBuf};

use crate::{
    cargo::{self, CargoOutput},
    error::Result,
    utils, Options, Project,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum TestAction {
    Expand,
    ExpandAndCheck,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum TestBehavior {
    OverwriteFiles,
    ExpectFiles,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum TestStatus {
    Success,
    Failure,
}

impl std::ops::BitAnd for TestStatus {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (TestStatus::Success, TestStatus::Success) => TestStatus::Success,
            _ => TestStatus::Failure,
        }
    }
}

impl std::ops::BitOr for TestStatus {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (TestStatus::Success, _) | (_, TestStatus::Success) => TestStatus::Success,
            _ => TestStatus::Failure,
        }
    }
}

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
        source: String,
        stdout: String,
        stderr: Option<String>,
    },
    UnexpectedFailure {
        source: String,
        stdout: Option<String>,
        stderr: String,
    },
}

impl TestOutcome {
    pub(crate) fn as_result(&self) -> TestStatus {
        match self {
            Self::SnapshotMatch { .. } => TestStatus::Success,
            Self::SnapshotMismatch { .. } => TestStatus::Failure,
            Self::SnapshotCreated { .. } => TestStatus::Success,
            Self::SnapshotUpdated { .. } => TestStatus::Success,
            Self::SnapshotExpected { .. } => TestStatus::Failure,
            Self::SnapshotUnexpected { .. } => TestStatus::Failure,
            Self::UnexpectedSuccess { .. } => TestStatus::Failure,
            Self::UnexpectedFailure { .. } => TestStatus::Failure,
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
    pub action: TestAction,
    pub behavior: TestBehavior,
    pub expectation: TestStatus,
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
    ) -> Result<TestStatus> {
        let TestPlan {
            action,
            behavior,
            expectation,
        } = plan;

        match action {
            TestAction::Expand => {
                let output = cargo::expand(project, self, options)?;

                if output.evaluation != *expectation {
                    let input = String::from_utf8_lossy(&utils::read(&self.path)?).into_owned();
                    return self.report_unexpected(&input, &output, observe);
                }

                self.evaluate_expand(output, *behavior, observe)
            }
            TestAction::ExpandAndCheck => {
                let expand_output = cargo::expand(project, self, options)?;

                // It only makes sense to run `cargo check` if `cargo expand` was successful:
                let combined_output = if expand_output.evaluation == TestStatus::Success {
                    let check_output = cargo::check(project, self, options)?;

                    // The expansion was successful, so we take its output:
                    let stdout = expand_output.stdout.clone();
                    // The expansion was successful, so all we care about is the check's errors:
                    let stderr = check_output.stderr.clone();
                    // Either both succeeded, or nothing did:
                    let evaluation = expand_output.evaluation & check_output.evaluation;

                    CargoOutput {
                        stdout,
                        stderr,
                        evaluation,
                    }
                } else {
                    expand_output
                };

                if combined_output.evaluation != *expectation {
                    let input = String::from_utf8_lossy(&utils::read(&self.path)?).into_owned();
                    return self.report_unexpected(&input, &combined_output, observe);
                }

                self.evaluate_expand(combined_output, *behavior, observe)
            }
        }
    }

    fn evaluate_expand(
        &mut self,
        output: CargoOutput,
        behavior: TestBehavior,
        observe: &mut dyn FnMut(TestOutcome),
    ) -> Result<TestStatus> {
        let stdout_snapshot_path = self.expand_out_rs_snapshot_path();
        let stderr_snapshot_path = self.expand_err_txt_snapshot_path();

        let snapshots = match output.evaluation {
            TestStatus::Success => [
                (&stdout_snapshot_path, output.stdout),
                (&stderr_snapshot_path, None),
            ],
            TestStatus::Failure => [
                (&stdout_snapshot_path, output.stdout),
                (&stderr_snapshot_path, output.stderr),
            ],
        };

        for (snapshot_path, actual) in snapshots {
            let expected = if snapshot_path.exists() {
                Some(String::from_utf8_lossy(&utils::read(snapshot_path)?).into_owned())
            } else {
                None
            };

            match behavior {
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

        Ok(output.evaluation)
    }

    // fn report_unexpected_expand(
    //     &mut self,
    //     output: &CargoOutput,
    //     observe: &mut dyn FnMut(TestOutcome),
    // ) -> Result<TestStatus> {
    //     match output.evaluation {
    //         TestStatus::Success => {
    //             let Some(stdout) = output.stdout.clone() else {
    //                 return Err(crate::error::Error::UnexpectedEmptyStdOut);
    //             };
    //             observe(TestOutcome::UnexpectedSuccess {
    //                 stdout,
    //                 stderr: None,
    //             });
    //             Ok(TestStatus::Failure)
    //         }
    //         TestStatus::Failure => {
    //             let Some(stderr) = output.stderr.clone() else {
    //                 return Err(crate::error::Error::UnexpectedEmptyStdErr);
    //             };
    //             observe(TestOutcome::UnexpectedFailure {
    //                 stdout: None,
    //                 stderr: stderr.clone(),
    //             });
    //             Ok(TestStatus::Failure)
    //         }
    //     }
    // }

    fn report_unexpected(
        &mut self,
        input: &str,
        output: &CargoOutput,
        observe: &mut dyn FnMut(TestOutcome),
    ) -> Result<TestStatus> {
        match output.evaluation {
            TestStatus::Success => {
                let Some(stdout) = output.stdout.clone() else {
                    return Err(crate::error::Error::UnexpectedEmptyStdOut);
                };
                observe(TestOutcome::UnexpectedSuccess {
                    source: input.to_owned(),
                    stdout,
                    stderr: output.stderr.clone(),
                });
                Ok(TestStatus::Failure)
            }
            TestStatus::Failure => {
                let Some(stderr) = output.stderr.clone() else {
                    return Err(crate::error::Error::UnexpectedEmptyStdErr);
                };
                observe(TestOutcome::UnexpectedFailure {
                    source: input.to_owned(),
                    stdout: output.stdout.clone(),
                    stderr: stderr.clone(),
                });
                Ok(TestStatus::Failure)
            }
        }
    }

    fn evaluate_snapshot_overwriting_files(
        &mut self,
        expected: Option<String>,
        actual: Option<String>,
        snapshot_path: &Path,
        observe: &mut dyn FnMut(TestOutcome),
    ) -> Result<TestStatus> {
        let Some(actual) = actual else {
            return Ok(TestStatus::Success);
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

        Ok(TestStatus::Success)
    }

    fn evaluate_snapshot_expecting_files(
        &mut self,
        expected: Option<String>,
        actual: Option<String>,
        snapshot_path: &Path,
        observe: &mut dyn FnMut(TestOutcome),
    ) -> Result<TestStatus> {
        match (actual, expected) {
            (None, None) => Ok(TestStatus::Success),
            (None, Some(expected)) => {
                observe(TestOutcome::SnapshotUnexpected {
                    content: expected,
                    path: snapshot_path.to_owned(),
                });
                Ok(TestStatus::Failure)
            }
            (Some(actual), None) => {
                observe(TestOutcome::SnapshotExpected {
                    content: actual,
                    path: snapshot_path.to_owned(),
                });
                Ok(TestStatus::Failure)
            }
            (Some(actual), Some(expected)) => {
                let comparison = Self::compare(&actual, &expected);
                match comparison {
                    Comparison::Match => {
                        observe(TestOutcome::SnapshotMatch {
                            path: snapshot_path.to_owned(),
                        });
                        Ok(TestStatus::Success)
                    }
                    Comparison::Mismatch => {
                        observe(TestOutcome::SnapshotMismatch {
                            expected,
                            actual: actual.clone(),
                            path: snapshot_path.to_owned(),
                        });
                        Ok(TestStatus::Failure)
                    }
                }
            }
        }
    }

    fn expand_out_rs_snapshot_path(&self) -> PathBuf {
        self.path.with_extension(crate::EXPAND_OUT_RS_FILE_SUFFIX)
    }

    fn expand_err_txt_snapshot_path(&self) -> PathBuf {
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
