use std::{
    ops::BitAnd,
    path::{Path, PathBuf},
};

use crate::{
    cargo::{self, CargoOutput},
    error::Result,
    options::Options,
    project::Project,
    utils,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum PostExpandAction {
    Check,
    Test,
    Run,
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

impl TestStatus {
    pub(crate) fn success(is_success: bool) -> Self {
        match is_success {
            true => Self::Success,
            false => Self::Failure,
        }
    }

    pub(crate) fn failure(is_failure: bool) -> Self {
        Self::success(!is_failure)
    }
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

impl std::ops::BitAndAssign for TestStatus {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.bitand(rhs);
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

impl std::ops::BitOrAssign for TestStatus {
    fn bitor_assign(&mut self, rhs: Self) {
        use std::ops::BitOr;

        *self = self.bitor(rhs);
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct TestReport {
    pub expand: CargoOutput,
    pub post_expand: Option<PostExpandOutput>,
}

impl TestReport {
    pub fn post_expand_output(&self) -> Option<&CargoOutput> {
        self.post_expand
            .as_ref()
            .map(|post_expand| post_expand.output())
    }

    pub fn expanded(&self) -> Option<String> {
        self.expand.stdout.clone()
    }

    pub fn output(&self) -> Option<String> {
        if self.expand.evaluation == TestStatus::Failure {
            return None;
        }

        self.post_expand_output()
            .and_then(|output| output.stdout.clone())
    }

    pub fn error(&self) -> Option<String> {
        if self.expand.evaluation == TestStatus::Failure {
            return None;
        }

        self.post_expand_output()
            .and_then(|output| output.stderr.clone())
    }

    pub fn evaluation(&self) -> TestStatus {
        let mut evaluation: TestStatus = TestStatus::Success;

        evaluation &= self.expand.evaluation;

        if let Some(output) = self.post_expand_output() {
            evaluation &= output.evaluation;
        }

        evaluation
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum PostExpandOutput {
    Check(CargoOutput),
    Test(CargoOutput),
    Run(CargoOutput),
}

impl PostExpandOutput {
    fn output(&self) -> &CargoOutput {
        match self {
            Self::Check(output) => output,
            Self::Test(output) => output,
            Self::Run(output) => output,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
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
        expanded: Option<String>,
        output: Option<String>,
        error: Option<String>,
    },
    UnexpectedFailure {
        source: String,
        expanded: Option<String>,
        output: Option<String>,
        error: Option<String>,
    },
}

impl TestOutcome {
    pub(crate) fn as_status(&self) -> TestStatus {
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
    pub post_expand: Option<PostExpandAction>,
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
            post_expand: post_expand_action,
            behavior,
            expectation,
        } = plan;

        let behavior = if options.skip_overwrite {
            // If the `skip_overwrite` flag is set we just check files,
            // instead of overwriting. The main purpose of this behavior
            // is to allow for our own unit tests to run with `#[should_panic]`
            // on the same directory (just flipping `pass/` with `fail/` directories)
            // without it emitting snapshots that would then make the non-inverted
            // tests fail and vice versa:
            TestBehavior::ExpectFiles
        } else {
            *behavior
        };

        let expand = cargo::expand(project, self, options)?;

        let post_expand = if expand.evaluation == TestStatus::Success {
            if let Some(action) = post_expand_action {
                Some(match action {
                    PostExpandAction::Check => {
                        PostExpandOutput::Check(cargo::check(project, self, options)?)
                    }
                    PostExpandAction::Test => {
                        PostExpandOutput::Test(cargo::test(project, self, options)?)
                    }
                    PostExpandAction::Run => {
                        PostExpandOutput::Run(cargo::run(project, self, options)?)
                    }
                })
            } else {
                None
            }
        } else {
            None
        };

        let report = TestReport {
            expand,
            post_expand,
        };

        let source = String::from_utf8_lossy(&utils::read(&self.path)?).into_owned();

        let evaluation = match (report.evaluation(), expectation) {
            (TestStatus::Success, TestStatus::Failure) => {
                self.report_unexpected_success(&source, &report, observe);
                TestStatus::Failure
            }
            (TestStatus::Failure, TestStatus::Success) => {
                self.report_unexpected_failure(&source, &report, observe);
                TestStatus::Failure
            }
            (TestStatus::Success, TestStatus::Success)
            | (TestStatus::Failure, TestStatus::Failure) => {
                self.process_snapshots(&report, behavior, observe)?
            }
        };

        Ok(evaluation)
    }

    fn process_snapshots(
        &mut self,
        report: &TestReport,
        behavior: TestBehavior,
        observe: &mut dyn FnMut(TestOutcome),
    ) -> Result<TestStatus> {
        let expanded_snapshot_path = self.path.with_extension(crate::OUT_RS_FILE_SUFFIX);
        let output_snapshot_path = self.path.with_extension(crate::OUT_TXT_FILE_SUFFIX);
        let error_snapshot_path = self.path.with_extension(crate::ERR_TXT_FILE_SUFFIX);

        let mut snapshots = vec![
            // We always want the expansion outputs:
            (&expanded_snapshot_path, report.expand.stdout.clone()),
        ];

        match report.expand.evaluation {
            TestStatus::Failure => {
                snapshots.push((&error_snapshot_path, report.expand.stderr.clone()));
            }
            TestStatus::Success => {
                if let Some(post_expand) = &report.post_expand {
                    match &post_expand {
                        PostExpandOutput::Check(output) => {
                            snapshots.push((&error_snapshot_path, output.stderr.clone()));
                        }
                        PostExpandOutput::Test(output) => {
                            snapshots.push((&output_snapshot_path, output.stdout.clone()));
                            snapshots.push((&error_snapshot_path, output.stdout.clone()));
                        }
                        PostExpandOutput::Run(output) => {
                            snapshots.push((&output_snapshot_path, output.stdout.clone()));
                            snapshots.push((&error_snapshot_path, output.stderr.clone()));
                        }
                    }
                }
            }
        }

        self.evaluate_snapshots(snapshots, behavior, observe)?;

        Ok(report.evaluation())
    }

    fn report_unexpected_success(
        &mut self,
        source: &str,
        report: &TestReport,
        observe: &mut dyn FnMut(TestOutcome),
    ) {
        observe(TestOutcome::UnexpectedSuccess {
            source: source.to_owned(),
            expanded: report.expanded().clone(),
            output: report.output().clone(),
            error: report.error().clone(),
        });
    }

    fn report_unexpected_failure(
        &mut self,
        source: &str,
        output: &TestReport,
        observe: &mut dyn FnMut(TestOutcome),
    ) {
        observe(TestOutcome::UnexpectedFailure {
            source: source.to_owned(),
            expanded: output.expanded().clone(),
            output: output.output().clone(),
            error: output.error().clone(),
        });
    }

    fn evaluate_snapshots(
        &mut self,
        snapshots: Vec<(&PathBuf, Option<String>)>,
        behavior: TestBehavior,
        observe: &mut dyn FnMut(TestOutcome),
    ) -> Result<TestStatus> {
        let mut outcomes = vec![];

        for (snapshot_path, actual) in snapshots {
            let expected = if snapshot_path.exists() {
                Some(String::from_utf8_lossy(&utils::read(snapshot_path)?).into_owned())
            } else {
                None
            };

            let outcome = match behavior {
                // We either create snapshots if the user requested so:
                TestBehavior::OverwriteFiles => {
                    self.evaluate_snapshot_overwriting_files(expected, actual, snapshot_path)?
                }
                // Or otherwise check for existing snapshots:
                TestBehavior::ExpectFiles => {
                    self.evaluate_snapshot_expecting_files(expected, actual, snapshot_path)?
                }
            };

            if let Some(outcome) = outcome {
                outcomes.push(outcome);
            }
        }

        let (successes, failures): (Vec<_>, Vec<_>) = outcomes
            .into_iter()
            .partition(|outcome| outcome.as_status() == TestStatus::Success);

        if !failures.is_empty() {
            for outcome in failures {
                observe(outcome);
            }
            return Ok(TestStatus::Failure);
        }

        for outcome in successes {
            observe(outcome);
        }

        Ok(TestStatus::Success)
    }

    fn evaluate_snapshot_overwriting_files(
        &mut self,
        expected: Option<String>,
        actual: Option<String>,
        snapshot_path: &Path,
    ) -> Result<Option<TestOutcome>> {
        let Some(actual) = actual else {
            return Ok(None);
        };

        if let Some(expected) = expected {
            if actual == expected {
                return Ok(None);
            }

            utils::write(snapshot_path, &actual)?;

            Ok(Some(TestOutcome::SnapshotUpdated {
                before: expected.clone(),
                after: actual.clone(),
                path: snapshot_path.to_owned(),
            }))
        } else {
            utils::write(snapshot_path, &actual)?;

            Ok(Some(TestOutcome::SnapshotCreated {
                after: actual.clone(),
                path: snapshot_path.to_owned(),
            }))
        }
    }

    fn evaluate_snapshot_expecting_files(
        &mut self,
        expected: Option<String>,
        actual: Option<String>,
        snapshot_path: &Path,
    ) -> Result<Option<TestOutcome>> {
        match (actual, expected) {
            (None, None) => Ok(Some(TestOutcome::SnapshotMatch {
                path: snapshot_path.to_owned(),
            })),
            (None, Some(expected)) => Ok(Some(TestOutcome::SnapshotUnexpected {
                content: expected,
                path: snapshot_path.to_owned(),
            })),
            (Some(actual), None) => Ok(Some(TestOutcome::SnapshotExpected {
                content: actual,
                path: snapshot_path.to_owned(),
            })),
            (Some(actual), Some(expected)) => {
                let comparison = Self::compare(&actual, &expected);
                match comparison {
                    Comparison::Match => Ok(Some(TestOutcome::SnapshotMatch {
                        path: snapshot_path.to_owned(),
                    })),
                    Comparison::Mismatch => Ok(Some(TestOutcome::SnapshotMismatch {
                        expected,
                        actual: actual.clone(),
                        path: snapshot_path.to_owned(),
                    })),
                }
            }
        }
    }

    fn compare(actual: &str, expected: &str) -> Comparison {
        if actual.lines().eq(expected.lines()) {
            Comparison::Match
        } else {
            Comparison::Mismatch
        }
    }
}
