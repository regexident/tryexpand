use std::path::Path;

use yansi::Paint;

use crate::{test::TestOutcome, TRYEXPAND_ENV_KEY, TRYEXPAND_ENV_VAL_OVERWRITE};

pub(crate) fn report_outcome(source_path: &Path, outcome: &TestOutcome) {
    match outcome {
        TestOutcome::SnapshotMatch { path } => {
            snapshot_match(source_path, path);
        }
        TestOutcome::SnapshotMismatch {
            actual,
            expected,
            path,
        } => {
            snapshot_mismatch(source_path, path, expected, actual);
        }
        TestOutcome::SnapshotCreated { after, path } => {
            snapshot_created(source_path, path, after);
        }
        TestOutcome::SnapshotUpdated {
            before,
            after,
            path,
        } => {
            snapshot_updated(source_path, path, before, after);
        }
        TestOutcome::SnapshotExpected { path, content } => {
            snapshot_expected(source_path, path, content);
        }
        TestOutcome::SnapshotUnexpected { path, content } => {
            snapshot_unexpected(source_path, path, content);
        }
        TestOutcome::UnexpectedSuccess {
            source,
            expanded,
            output,
            error,
        } => {
            unexpected_success(
                source_path,
                source,
                expanded.as_deref(),
                output.as_deref(),
                error.as_deref(),
            );
        }
        TestOutcome::UnexpectedFailure {
            source,
            expanded,
            output,
            error,
        } => {
            unexpected_failure(
                source_path,
                source,
                expanded.as_deref(),
                output.as_deref(),
                error.as_deref(),
            );
        }
    }
}

pub(crate) fn snapshot_match(path: &Path, _snapshot_path: &Path) {
    eprintln!("{path} - {}", Paint::green("ok"), path = path.display());
}

pub(crate) fn snapshot_mismatch(path: &Path, snapshot_path: &Path, expected: &str, actual: &str) {
    eprintln!("{path} - {}", Paint::red("MISMATCH"), path = path.display());
    eprintln!("--------------------------");

    eprintln!(
        "Unexpected mismatch in file {path}:",
        path = snapshot_path.display()
    );

    print_snapshot_diff(expected, actual);

    print_overwrite_hint();

    eprintln!("--------------------------");
}

pub(crate) fn snapshot_created(path: &Path, snapshot_path: &Path, snapshot: &str) {
    eprintln!(
        "{path} - {}",
        Paint::yellow("created"),
        path = path.display()
    );
    eprintln!("--------------------------");

    eprintln!(
        "{}",
        Paint::green(format!(
            "Snapshot created at path {path}",
            path = snapshot_path.display()
        ))
    );

    print_valid_snapshot(snapshot);

    eprintln!("--------------------------");
}

pub(crate) fn snapshot_updated(path: &Path, snapshot_path: &Path, before: &str, after: &str) {
    eprintln!(
        "{path} - {}",
        Paint::yellow("updated"),
        path = path.display()
    );
    eprintln!("--------------------------");

    eprintln!(
        "{}",
        Paint::green(format!(
            "Snapshot updated at path {path}",
            path = snapshot_path.display()
        ))
    );

    print_snapshot_diff(before, after);

    eprintln!("--------------------------");
}

pub(crate) fn snapshot_expected(path: &Path, snapshot_path: &Path, snapshot: &str) {
    eprintln!("{path} - {}", Paint::red("MISSING"), path = path.display());
    eprintln!("--------------------------");

    eprintln!(
        "{}",
        Paint::red(format!(
            "Expected snapshot at {path}",
            path = snapshot_path.display()
        ))
    );

    print_invalid_snapshot(snapshot);

    print_overwrite_hint();

    eprintln!("--------------------------");
}

pub(crate) fn snapshot_unexpected(path: &Path, snapshot_path: &Path, snapshot: &str) {
    eprintln!("{path} - {}", Paint::red("ERROR"), path = path.display());
    eprintln!("--------------------------");

    eprintln!(
        "{}",
        Paint::red(format!(
            "Unexpected snapshot at {path}",
            path = snapshot_path.display()
        ))
    );

    print_invalid_snapshot(snapshot);

    print_remove_hint(snapshot_path);

    eprintln!("--------------------------");
}

pub(crate) fn unexpected_success(
    path: &Path,
    source: &str,
    expanded: Option<&str>,
    output: Option<&str>,
    error: Option<&str>,
) {
    eprintln!("{path} - {}", Paint::red("ERROR"), path = path.display());
    eprintln!("--------------------------");

    eprintln!("{}", Paint::red("Unexpected success!"));

    print_source(source);

    if let Some(expanded) = expanded {
        print_expanded_snapshot(expanded);
    }
    if let Some(output) = output {
        print_output_snapshot(output);
    }
    if let Some(error) = error {
        print_error_snapshot(error);
    }

    eprintln!("--------------------------");
}

pub(crate) fn unexpected_failure(
    path: &Path,
    source: &str,
    expanded: Option<&str>,
    output: Option<&str>,
    error: Option<&str>,
) {
    eprintln!("{path} - {}", Paint::red("ERROR"), path = path.display());
    eprintln!("--------------------------");

    eprintln!("{}", Paint::red("Unexpected failure!"));

    print_source(source);

    if let Some(expanded) = expanded {
        print_expanded_snapshot(expanded);
    }
    if let Some(output) = output {
        print_output_snapshot(output);
    }
    if let Some(error) = error {
        print_error_snapshot(error);
    }

    eprintln!("--------------------------");
}

pub(crate) fn command_failure(path: &Path, error: &str) {
    eprintln!("{path} - {}", Paint::red("ERROR"), path = path.display());
    eprintln!("--------------------------");

    eprintln!("{}", Paint::red("Command failure!"));

    print_error_snapshot(error);

    if error.contains("no such subcommand: `expand`") {
        print_install_cargo_expand_hint();
    }

    eprintln!("--------------------------");
}

pub(crate) fn command_abortion(num_errors: usize) {
    eprintln!(
        "{}",
        Paint::red(format!("Aborting due to {num_errors} previous errors."))
    );
    eprintln!();
}

fn print_source(source: &str) {
    eprintln!();
    eprintln!("SOURCE:");
    eprintln!();
    print_lines(source, Paint::blue);
}

fn print_snapshot_diff(expected: &str, actual: &str) {
    eprintln!();
    eprintln!("DIFF:");
    eprintln!();
    print_diff(expected, actual, 2);
}

fn print_expanded_snapshot(expanded: &str) {
    eprintln!();
    eprintln!("EXPANDED:");
    eprintln!();
    print_lines(expanded, Paint::blue);
}

fn print_output_snapshot(output: &str) {
    eprintln!();
    eprintln!("OUTPUT:");
    eprintln!();
    print_lines(output, Paint::blue);
}

fn print_error_snapshot(error: &str) {
    eprintln!();
    eprintln!("ERROR:");
    eprintln!();
    print_lines(error, Paint::red);
}

fn print_valid_snapshot(snapshot: &str) {
    eprintln!();
    eprintln!("SNAPSHOT:");
    eprintln!();
    print_lines(snapshot, Paint::blue);
}

fn print_invalid_snapshot(snapshot: &str) {
    eprintln!();
    eprintln!("SNAPSHOT:");
    eprintln!();
    print_lines(snapshot, Paint::red);
}

fn print_install_cargo_expand_hint() {
    eprintln!();
    eprintln!(
        "{}",
        Paint::cyan("help: Perhaps, `cargo expand` is not installed?")
    );
    eprintln!("{}", Paint::cyan("      Install it by running:"));
    eprintln!();
    eprintln!("{}", Paint::cyan("      $ cargo install cargo-expand"));
}

fn print_overwrite_hint() {
    eprintln!();
    eprintln!(
        "{}",
        Paint::cyan(format!(
            "help: Overwrite the snapshot file by running your tests with `{key}={val}`.",
            key = TRYEXPAND_ENV_KEY,
            val = TRYEXPAND_ENV_VAL_OVERWRITE
        ))
    );
}

fn print_remove_hint(path: &Path) {
    let path = match std::env::current_dir() {
        Ok(directory) => directory.join(path),
        Err(_) => path.to_owned(),
    };

    let path_display = path.display();

    eprintln!();
    eprintln!(
        "{}",
        Paint::cyan(format!("help: Remove the snapshot file at {path_display}.",))
    );
}

fn print_lines<F>(string: &str, f: F)
where
    F: Fn(String) -> Paint<String>,
{
    let lines: Vec<&str> = string.lines().collect();
    for line in lines {
        eprintln!("{}", f(line.to_owned()));
    }
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
