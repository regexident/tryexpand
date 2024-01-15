use std::{env, ffi::OsString, io::BufRead, process::Command};

use serde::Serialize;

use crate::{
    error::{Error, Result},
    normalization::{failure_stderr, failure_stdout, success_stderr, success_stdout},
    project::Project,
    test::{Evaluation, Test},
    Options,
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
pub(crate) struct Expansion {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub evaluation: Evaluation,
}

pub(crate) fn expand(
    project: &Project,
    test: &Test,
    options: &Option<Options>,
) -> Result<Expansion> {
    let mut cargo = cargo(project);
    let cargo = cargo
        .arg("expand")
        .arg("--bin")
        .arg(&test.bin)
        .arg("--theme")
        .arg("none");

    if let Some(options) = options {
        cargo.args(&options.args);

        for (key, value) in &options.env {
            cargo.env(key, value);
        }
    }

    let output = cargo
        .output()
        .map_err(|err| Error::CargoExpandExecution(err.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    let is_success = {
        // Unfortunately `cargo expand` will sometimes return a success status,
        // despite the expansion having produced errors in the log:
        let found_errors = stderr.lines().any(|line| line.starts_with("error:"));

        output.status.success() && !found_errors
    };

    if is_success {
        Ok(Expansion {
            stdout: success_stdout(stdout, project, test),
            stderr: success_stderr(stderr, project, test),
            evaluation: Evaluation::Success,
        })
    } else {
        Ok(Expansion {
            stdout: failure_stdout(stdout, project, test),
            stderr: failure_stderr(stderr, project, test),
            evaluation: Evaluation::Failure,
        })
    }
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

    println!("\n");

    let stdout = cargo(project)
        .arg("expand")
        .arg("--lib")
        .arg("--theme")
        .arg("none")
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
