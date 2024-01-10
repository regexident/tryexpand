use std::ffi::OsStr;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

use crate::{
    error::{Error, Result},
    expand::Project,
    manifest::Name,
    rustflags,
};

#[derive(Deserialize)]
pub struct Metadata {
    pub target_directory: PathBuf,
    pub workspace_root: PathBuf,
}

fn raw_cargo() -> Command {
    Command::new(option_env!("CARGO").unwrap_or("cargo"))
}

fn cargo(project: &Project) -> Command {
    let mut cmd = raw_cargo();
    cmd.current_dir(&project.dir);
    cmd.env("CARGO_TARGET_DIR", &project.inner_target_dir);
    rustflags::set_env(&mut cmd);
    cmd
}

pub(crate) fn metadata() -> Result<Metadata> {
    let output = raw_cargo()
        .arg("metadata")
        .arg("--format-version=1")
        .output()
        .map_err(Error::Cargo)?;

    serde_json::from_slice(&output.stdout).map_err(Error::CargoMetadata)
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum Expansion {
    Success { stdout: String },
    Failure { stderr: String },
}

pub(crate) fn expand<I, S>(project: &Project, name: &Name, args: &Option<I>) -> Result<Expansion>
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    let mut cargo = cargo(project);
    let cargo = cargo
        .arg("expand")
        .arg("--bin")
        .arg(name.as_ref())
        .arg("--theme")
        .arg("none");

    if let Some(args) = args {
        cargo.args(args.clone());
    }

    let output = cargo
        .output()
        .map_err(|e| Error::CargoExpandExecution(e.to_string()))?;

    let status = output.status;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    // Sometimes the `cargo expand` command returns a success status,
    // despite an error having occurred, so we need to look for those:
    let has_errors = stderr.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("error: ") || line.starts_with("ERROR: ")
    });

    let is_success = status.success() && !output.stdout.is_empty() && !has_errors;

    if is_success {
        Ok(Expansion::Success { stdout })
    } else {
        Ok(Expansion::Failure { stderr })
    }
}

/// Builds dependencies for macro expansion and pipes `cargo` output to `STDOUT`.
/// Tries to expand macros in `main.rs` and intentionally filters the result.
/// This function is called before macro expansions to speed them up and
/// for dependencies build process to be visible for user.
pub(crate) fn build_dependencies(project: &Project) -> Result<()> {
    use std::io::Write;

    let stdout = cargo(project)
        .arg("expand")
        .arg("--bin")
        .arg(project.name.clone())
        .arg("--theme")
        .arg("none")
        .stdout(std::process::Stdio::piped())
        .spawn()?
        .stdout
        .ok_or(Error::CargoFail)?;

    let reader = std::io::BufReader::new(stdout);

    // Filter ignored lines and main.rs content
    reader
        .lines()
        .map_while(std::io::Result::ok)
        .filter(|line| !line.starts_with("fn main() {}"))
        .filter(|line| !line_should_be_ignored(line))
        .for_each(|line| {
            let _ = writeln!(std::io::stdout(), "{}", line);
        });

    Ok(())
}

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
