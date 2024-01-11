use std::path::Path;

use yansi::Paint;

use crate::{test::TestOutcome, TRYEXPAND_ENV_KEY, TRYEXPAND_ENV_VAL_OVERWRITE};

pub(crate) fn report_outcome(path: &Path, expanded_path: &Path, outcome: &TestOutcome) {
    match outcome {
        TestOutcome::SnapshotMatch => {
            ok(path, expanded_path);
        }
        TestOutcome::SnapshotMismatch { actual, expected } => {
            snapshot_mismatch(path, expanded_path, expected, actual);
        }
        TestOutcome::SnapshotCreated { after } => {
            snapshot_created(path, expanded_path, after);
        }
        TestOutcome::SnapshotUpdated { before, after } => {
            snapshot_updated(path, expanded_path, before, after);
        }
        TestOutcome::SnapshotMissing => {
            snapshot_missing(path, expanded_path);
        }
        TestOutcome::UnexpectedSuccess { output } => {
            unexpected_success(path, expanded_path, output);
        }
        TestOutcome::UnexpectedFailure { output } => {
            unexpected_failure(path, expanded_path, output);
        }
        TestOutcome::CommandFailure { output } => {
            command_failure(path, expanded_path, output);
        }
    }
}

pub(crate) fn ok(path: &Path, _expanded_path: &Path) {
    eprintln!("{path} - {}", Paint::green("ok"), path = path.display());
}

pub(crate) fn snapshot_mismatch(path: &Path, expanded_path: &Path, expected: &str, actual: &str) {
    eprintln!("{path} - {}", Paint::red("MISMATCH"), path = path.display());
    eprintln!("--------------------------");
    eprintln!(
        "Unexpected mismatch in file {path}:",
        path = expanded_path.display()
    );
    eprintln!();

    const NUM_CONTEXT_LINES: usize = 2;

    print_diff(expected, actual, NUM_CONTEXT_LINES);

    eprintln!();
    eprintln!(
        "{}",
        Paint::cyan(format!(
            "help: To update the file run your tests with `{key}={val}`.",
            key = TRYEXPAND_ENV_KEY,
            val = TRYEXPAND_ENV_VAL_OVERWRITE
        ))
    );
    eprintln!("--------------------------");
}

pub(crate) fn snapshot_created(path: &Path, expanded_path: &Path, after: &str) {
    eprintln!(
        "{path} - {}",
        Paint::yellow("created"),
        path = path.display()
    );
    eprintln!("--------------------------");
    eprintln!(
        "{}",
        Paint::green(format!(
            "File created at path {path}",
            path = expanded_path.display()
        ))
    );
    print_diff("", after, 5);
    eprintln!("--------------------------");
}

pub(crate) fn snapshot_updated(path: &Path, expanded_path: &Path, before: &str, after: &str) {
    eprintln!(
        "{path} - {}",
        Paint::yellow("updated"),
        path = path.display()
    );
    eprintln!("--------------------------");
    eprintln!(
        "{}",
        Paint::green(format!(
            "File updated at path {path}",
            path = expanded_path.display()
        ))
    );
    print_diff(before, after, 5);
    eprintln!("--------------------------");
}

pub(crate) fn snapshot_missing(path: &Path, expanded_path: &Path) {
    eprintln!("{path} - {}", Paint::red("MISSING"), path = path.display());
    eprintln!("--------------------------");
    eprintln!(
        "{}",
        Paint::red(format!(
            "Expected file at path {path}",
            path = expanded_path.display()
        ))
    );
    eprintln!();
    eprintln!(
        "{}",
        Paint::cyan(format!(
            "help: Run your tests with `{key}={val}` to have the files created automatically.",
            key = TRYEXPAND_ENV_KEY,
            val = TRYEXPAND_ENV_VAL_OVERWRITE
        ))
    );
    eprintln!("--------------------------");
}

pub(crate) fn unexpected_success(path: &Path, _expanded_path: &Path, output: &str) {
    eprintln!("{path} - {}", Paint::red("ERROR"), path = path.display());
    eprintln!("--------------------------");
    eprintln!("{}", Paint::red("Unexpected success:"));
    eprintln!();
    let lines: Vec<&str> = output.lines().collect();
    let trimmed_lines = trim_infix(&lines, 3, 3);
    for line in trimmed_lines {
        eprintln!("{}", Paint::red(line));
    }
    eprintln!("--------------------------");
}

pub(crate) fn unexpected_failure(path: &Path, _expanded_path: &Path, output: &str) {
    eprintln!("{path} - {}", Paint::red("ERROR"), path = path.display());
    eprintln!("--------------------------");

    eprintln!("{}", Paint::red("Unexpected failure:"));
    eprintln!();
    eprintln!("{}", Paint::red(output.trim()));

    eprintln!("--------------------------");
}

pub(crate) fn command_failure(path: &Path, _expanded_path: &Path, output: &str) {
    eprintln!("{path} - {}", Paint::red("ERROR"), path = path.display());
    eprintln!("--------------------------");

    eprintln!("{}", Paint::red("Command failure:"));
    eprintln!();
    eprintln!("{}", Paint::red(output.trim()));

    // No `cargo expand` subcommand installed, make a suggestion
    if output.contains("no such subcommand: `expand`") {
        eprintln!(
            "{}",
            Paint::cyan("help: Perhaps, `cargo expand` is not installed?")
        );
        eprintln!("{}", Paint::cyan("      Install it by running:"));
        eprintln!();
        eprintln!("{}", Paint::cyan("      $ cargo install cargo-expand"));
        eprintln!();
    }

    eprintln!("--------------------------");
}

fn print_diff(before: &str, after: &str, num_context_lines: usize) {
    let diff_lines = diff::lines(before, after);
    for diff_run in DiffRunsIterator::from(diff_lines.into_iter()) {
        match diff_run {
            diff::Result::Left(lines) => {
                for line in lines {
                    eprintln!("{}", Paint::red(format!("- {line}")));
                }
            }
            diff::Result::Both(lines, _) => {
                let lines = trim_infix(&lines, num_context_lines, num_context_lines);

                for line in lines {
                    eprintln!("{}", Paint::blue(format!("  {line}")));
                }
            }
            diff::Result::Right(lines) => {
                for line in lines {
                    eprintln!("{}", Paint::green(format!("+ {line}")));
                }
            }
        }
    }
}

fn trim_infix<'a>(
    lines: &'a [&'a str],
    max_prefix_len: usize,
    max_suffix_len: usize,
) -> Vec<&'a str> {
    let mut trimmed: Vec<&str> = vec![];
    let num_lens = lines.len();

    for (index, &line) in lines.iter().enumerate() {
        if index < max_prefix_len {
            trimmed.push(line);
        } else if index == max_prefix_len {
            if num_lens > (max_prefix_len + max_suffix_len) {
                trimmed.push("...");
            } else {
                trimmed.push(line);
            }
        } else if (num_lens - index) <= max_suffix_len {
            trimmed.push(line);
        }
    }

    trimmed
}

struct DiffRunsIterator<I>
where
    I: Iterator,
{
    iter: I,
    current: Option<I::Item>,
}

impl<T, I> From<I> for DiffRunsIterator<I>
where
    I: Iterator<Item = diff::Result<T>>,
{
    fn from(mut iter: I) -> Self {
        let current = iter.next();
        Self { iter, current }
    }
}

impl<T, I> Iterator for DiffRunsIterator<I>
where
    T: Clone,
    I: Iterator<Item = diff::Result<T>>,
{
    type Item = diff::Result<Vec<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        use diff::Result;

        let Some(current) = self.current.take() else {
            return None;
        };

        let mut result = match current {
            Result::Left(lhs) => Result::Left(vec![lhs]),
            Result::Both(lhs, rhs) => Result::Both(vec![lhs], vec![rhs]),
            Result::Right(rhs) => Result::Right(vec![rhs]),
        };

        for next in self.iter.by_ref() {
            match (&mut result, &next) {
                (Result::Left(run), Result::Left(lhs)) => {
                    run.push(lhs.clone());
                }
                (Result::Both(lhs_run, rhs_run), Result::Both(lhs, rhs)) => {
                    lhs_run.push(lhs.clone());
                    rhs_run.push(rhs.clone());
                }
                (Result::Right(run), Result::Right(rhs)) => {
                    run.push(rhs.clone());
                }
                (_, next) => {
                    self.current = Some(next.clone());
                    break;
                }
            };
        }

        Some(result)
    }
}
