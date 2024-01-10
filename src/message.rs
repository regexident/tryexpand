use std::path::Path;
use yansi::Paint;

use crate::{TRYEXPAND_ENV_KEY, TRYEXPAND_ENV_VAL_OVERWRITE};

/// Prints the difference of the two snippets of expanded code.
pub(crate) fn mismatch(path: &Path, expanded_path: &Path, a: &[u8], b: &[u8]) {
    let a = String::from_utf8_lossy(a);
    let b = String::from_utf8_lossy(b);

    let diff_lines = diff::lines(&a, &b);

    let diff_runs = DiffRunsIterator::from(diff_lines.into_iter());

    const NUM_CONTEXT_LINES: usize = 2;

    eprintln!("{path} - {}", Paint::red("MISMATCH"), path = path.display());
    eprintln!("--------------------------");
    eprintln!(
        "Unexpected mismatch in file {path}:",
        path = expanded_path.display()
    );
    eprintln!();

    for diff_run in diff_runs {
        let run_len = diff_run.len();
        for (index, change) in diff_run.into_iter().enumerate() {
            match change {
                diff::Result::Left(lhs) => {
                    eprintln!("{}", Paint::red(format!("- {lhs}")));
                }
                diff::Result::Both(both, _) => {
                    if index < NUM_CONTEXT_LINES {
                        eprintln!("{}", Paint::blue(format!("  {both}")));
                    } else if index == NUM_CONTEXT_LINES {
                        if run_len > (2 * NUM_CONTEXT_LINES) {
                            eprintln!("{}", Paint::blue("  ..."));
                        } else {
                            eprintln!("{}", Paint::blue(format!("  {both}")));
                        }
                    } else if (run_len - index) <= NUM_CONTEXT_LINES {
                        eprintln!("{}", Paint::blue(format!("  {both}")));
                    }
                }
                diff::Result::Right(rhs) => {
                    eprintln!("{}", Paint::green(format!("+ {rhs}")));
                }
            }
        }
    }

    eprintln!("--------------------------");
}

pub(crate) fn ok(path: &Path, _expanded_path: &Path) {
    eprintln!("{path} - {}", Paint::green("ok"), path = path.display());
}

pub(crate) fn unexpected_failure(path: &Path, _expanded_path: &Path, err: &str) {
    eprintln!("{path} - {}", Paint::red("ERROR"), path = path.display());
    eprintln!("--------------------------");

    eprintln!("{}", Paint::red("Unexpected failure:"));
    eprintln!("{}", Paint::red(err));

    // No `cargo expand` subcommand installed, make a suggestion
    if err.contains("no such subcommand: `expand`") {
        eprintln!(
            "{}",
            Paint::cyan("help: Perhaps, `cargo expand` is not installed?")
        );
        eprintln!("{}", Paint::cyan("Install it by running:"));
        eprintln!();
        eprintln!("{}", Paint::cyan("\tcargo install cargo-expand"));
        eprintln!();
    }

    // No nightly installed, make a suggestion
    if err.starts_with("error: toolchain '") && err.ends_with("is not installed") {
        eprintln!("{}", Paint::cyan("You have `cargo expand` installed but it requires *nightly* compiler to be installed as well."));
        eprintln!("{}", Paint::cyan("To install it via rustup, run:"));
        eprintln!();
        eprintln!("{}", Paint::cyan("\trustup toolchain install nightly"));
        eprintln!();
    }

    eprintln!("--------------------------");
    eprintln!("--------------------------");
}

pub(crate) fn expected_failure(path: &Path, _expanded_path: &Path) {
    eprintln!("{path} - {}", Paint::red("ERROR"), path = path.display());
    eprintln!("--------------------------");
    eprintln!("{}", Paint::red("Expected failure"));
    eprintln!("--------------------------");
}

pub(crate) fn missing(path: &Path, expanded_path: &Path) {
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
            "Hint: Set the `{key}={val}` environment value when running the tests to create the files for your automatically.",
            key = TRYEXPAND_ENV_KEY,
            val = TRYEXPAND_ENV_VAL_OVERWRITE
        ))
    );
    eprintln!("--------------------------");
}

pub(crate) fn created(path: &Path, expanded_path: &Path) {
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
    eprintln!("--------------------------");
}

pub(crate) fn updated(path: &Path, expanded_path: &Path) {
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
    eprintln!("--------------------------");
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
    type Item = Vec<diff::Result<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        use diff::Result;

        let Some(mut current) = self.current.take() else {
            return None;
        };

        let mut run = vec![];

        for next in self.iter.by_ref() {
            let is_match = matches!(
                (&current, &next),
                (Result::Left(_), Result::Left(_))
                    | (Result::Both(_, _), Result::Both(_, _))
                    | (Result::Right(_), Result::Right(_))
            );

            run.push(current);

            if is_match {
                current = next;
            } else {
                self.current = Some(next);
                break;
            }
        }

        Some(run)
    }
}
