use std::{borrow::Cow, env, ffi::OsString, io::BufRead, process::Command};

use serde::Serialize;

use crate::{
    error::{Error, Result},
    normalization,
    options::Options,
    project::Project,
    test::{Test, TestStatus},
    utils::should_debug_log,
};

const RUSTFLAGS_ENV_KEY: &str = "RUSTFLAGS";

fn raw_cargo() -> Command {
    Command::new(option_env!("CARGO").unwrap_or("cargo"))
}

fn cargo(project: &Project) -> Command {
    let mut cmd = raw_cargo();
    cmd.current_dir(&project.dir);
    cmd.env("CARGO_TARGET_DIR", &project.target_dir);
    cmd.env(RUSTFLAGS_ENV_KEY, make_rustflags_env());
    cmd
}

#[derive(Serialize, Debug)]
pub struct Config {
    pub build: Build,
}

#[derive(Serialize, Debug)]
pub struct Build {
    pub rustflags: Vec<String>,
}

pub(crate) fn make_config() -> Config {
    Config {
        build: Build {
            rustflags: tryexpand_rustflags(),
        },
    }
}

fn tryexpand_rustflags() -> Vec<String> {
    vec!["-Awarnings".to_owned()]
}

fn make_rustflags_env() -> OsString {
    let mut rustflags = match env::var_os(RUSTFLAGS_ENV_KEY) {
        Some(rustflags) => rustflags,
        None => OsString::new(),
    };

    for rustflag in tryexpand_rustflags() {
        rustflags.push(rustflag);
    }

    rustflags
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct CargoOutput {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub evaluation: TestStatus,
}

pub(crate) fn expand(project: &Project, test: &Test, options: &Options) -> Result<CargoOutput> {
    let mut cargo = cargo(project);

    cargo
        .arg("expand")
        .arg("--bin")
        .arg(&test.bin)
        .arg("--theme")
        .arg("none");

    let CargoOutput {
        stdout,
        stderr,
        evaluation,
    } = run_cargo_command(cargo, options)?;

    let stdout = stdout.and_then(|stdout| {
        normalization::expand_stdout(Cow::from(stdout), project, test).map(|cow| cow.into_owned())
    });
    let stderr = stderr.and_then(|stderr| {
        normalization::expand_stderr(Cow::from(stderr), project, test).map(|cow| cow.into_owned())
    });

    let evaluation = match evaluation {
        TestStatus::Success => {
            if let Some(stderr) = &stderr {
                // Unfortunately `cargo expand` will sometimes return a success status,
                // despite the expansion having produced errors in the log:
                let found_errors = stderr.lines().any(line_is_error);
                TestStatus::failure(found_errors)
            } else {
                TestStatus::Success
            }
        }
        TestStatus::Failure => evaluation,
    };

    Ok(CargoOutput {
        stdout,
        stderr,
        evaluation,
    })
}

pub(crate) fn check(project: &Project, test: &Test, options: &Options) -> Result<CargoOutput> {
    let mut cargo = cargo(project);

    cargo
        .arg("check")
        .arg("--bin")
        .arg(&test.bin)
        .arg("--color")
        .arg("never");

    let CargoOutput {
        stdout,
        stderr,
        evaluation,
    } = run_cargo_command(cargo, options)?;

    let stdout = stdout.and_then(|stdout| {
        normalization::check_stdout(Cow::from(stdout), project, test).map(|cow| cow.into_owned())
    });
    let stderr = stderr.and_then(|stderr| {
        normalization::check_stderr(Cow::from(stderr), project, test).map(|cow| cow.into_owned())
    });

    Ok(CargoOutput {
        stdout,
        stderr,
        evaluation,
    })
}

pub(crate) fn test(project: &Project, test: &Test, options: &Options) -> Result<CargoOutput> {
    let mut cargo = cargo(project);

    cargo
        .arg("test")
        .arg("--bin")
        .arg(&test.bin)
        .arg("--color")
        .arg("never")
        .arg("--quiet");

    // We don't want a backtrace to dilute our snapshots (or make them instable):
    cargo.env("RUST_BACKTRACE", "0");

    let CargoOutput {
        stdout,
        stderr,
        evaluation,
    } = run_cargo_command(cargo, options)?;

    let stdout = stdout.and_then(|stdout| {
        normalization::test_stdout(Cow::from(stdout), project, test).map(|cow| cow.into_owned())
    });
    let stderr = stderr.and_then(|stderr| {
        normalization::test_stderr(Cow::from(stderr), project, test).map(|cow| cow.into_owned())
    });

    let evaluation = match evaluation {
        TestStatus::Success => {
            if let Some(stderr) = &stderr {
                // Unfortunately `cargo test` will return a success status,
                // despite the expansion having produced errors in the log:
                let found_errors = stderr
                    .lines()
                    .any(|line| line.starts_with("test result: FAILED."));
                TestStatus::failure(found_errors)
            } else {
                TestStatus::Success
            }
        }
        TestStatus::Failure => evaluation,
    };

    Ok(CargoOutput {
        stdout,
        stderr,
        evaluation,
    })
}

pub(crate) fn run(project: &Project, test: &Test, options: &Options) -> Result<CargoOutput> {
    let mut cargo = cargo(project);

    cargo
        .arg("run")
        .arg("--bin")
        .arg(&test.bin)
        .arg("--color")
        .arg("never")
        .arg("--quiet");

    // We don't want a backtrace to dilute our snapshots (or make them instable):
    cargo.env("RUST_BACKTRACE", "0");

    let CargoOutput {
        stdout,
        stderr,
        evaluation,
    } = run_cargo_command(cargo, options)?;

    let stdout = stdout.and_then(|stdout| {
        normalization::run_stdout(Cow::from(stdout), project, test).map(|cow| cow.into_owned())
    });
    let stderr = stderr.and_then(|stderr| {
        normalization::run_stderr(Cow::from(stderr), project, test).map(|cow| cow.into_owned())
    });

    Ok(CargoOutput {
        stdout,
        stderr,
        evaluation,
    })
}

fn run_cargo_command(mut cargo: Command, options: &Options) -> Result<CargoOutput> {
    cargo.args(&options.args);

    for (key, value) in &options.envs {
        cargo.env(key, value);
    }

    if should_debug_log().unwrap_or(false) {
        println!("Command: {:?}", cargo);
        println!("Environment: {:?}", options.envs);
        println!();
    }

    let output = cargo
        .output()
        .map_err(|err| Error::CargoExpandExecution(err.to_string()))?;

    let stdout = Some(String::from_utf8_lossy(&output.stdout).into_owned());
    let stderr = Some(String::from_utf8_lossy(&output.stderr).into_owned());

    if should_debug_log().unwrap_or(false) {
        println!("Stdout:\n{}", stdout.as_deref().unwrap_or("None"));
        println!();
        println!("Stderr:\n{}", stderr.as_deref().unwrap_or("None"));
        println!();
    }

    let evaluation = if output.status.success() {
        TestStatus::Success
    } else {
        TestStatus::Failure
    };

    Ok(CargoOutput {
        stdout,
        stderr,
        evaluation,
    })
}

/// Builds dependencies for macro expansion and pipes `cargo` output to `STDOUT`.
/// Tries to expand macros in `main.rs` and intentionally filters the result.
/// This function is called before macro expansions to speed them up and
/// for dependencies build process to be visible for user.
pub(crate) fn build_dependencies(project: &Project) -> Result<()> {
    use std::io::Write;

    const IGNORED_LINES: [&str; 5] = [
        "#![feature(prelude_import)]",
        "#[prelude_import]",
        "use std::prelude::",
        "#[macro_use]",
        "extern crate std;",
    ];

    fn line_should_be_ignored(line: &str) -> bool {
        for check in IGNORED_LINES.iter() {
            if line.starts_with(check) {
                return true;
            }
        }

        false
    }

    let _ = writeln!(std::io::stdout());
    let _ = writeln!(std::io::stdout());

    let stdout = cargo(project)
        .arg("check")
        .arg("--lib")
        .arg("--color")
        .arg("never")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(Error::SpawningProcessFailed)?
        .stdout
        .ok_or(Error::StdOutUnavailable)?;

    let reader = std::io::BufReader::new(stdout);

    // Filter ignored lines and lib.rs content
    reader
        .lines()
        .map_while(std::io::Result::ok)
        .filter(|line| !line_should_be_ignored(line))
        .for_each(|line| {
            let _ = writeln!(std::io::stdout(), "{}", line);
        });

    Ok(())
}

pub(crate) fn line_should_be_omitted(line: &str) -> bool {
    let line = line.trim();

    if line.starts_with("error: could not compile") && line.ends_with("due to previous error") {
        return true;
    }

    false
}

pub(crate) fn line_is_warning(line: &str) -> bool {
    let line = line.trim();

    if line.starts_with("warning:") {
        return true;
    }

    false
}

pub(crate) fn line_is_error(line: &str) -> bool {
    let line = line.trim();

    // Sometimes the `cargo expand` command returns a success status,
    // despite an error having occurred, so we need to look for those:

    // Example:
    // ```
    // ...
    // error[E0433]: failed to resolve: use of undeclared crate or module `pwyxfa`
    // --> /tests/expand/fail/test.rs:1:1
    // |
    // 1 | pwyxfa::skpwbd! {
    // | ^^^^^^ use of undeclared crate or module `pwyxfa`
    // For more information about this error, try `rustc --explain E0433`.
    // error: could not compile `<CRATE>` (bin "<BIN>") due to previous error
    // ...
    // ```
    if line.starts_with("error[") {
        return true;
    }

    // ```
    // error: expected item, found `1234`
    // --> /tests/expand/fail/test.rs:1:1
    // |
    // 1 | 1234
    // | ^^^^ expected item
    // |
    // ...
    // ```
    if line.starts_with("error:") {
        return true;
    }

    false
}
