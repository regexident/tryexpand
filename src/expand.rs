use std::{
    env,
    ffi::OsStr,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use syn::{punctuated::Punctuated, Item, Meta, Token};

use crate::{
    cargo::{self, Expansion},
    dependencies::{self, Dependency},
    error::{Error, Result},
    features,
    manifest::{Bin, Build, Config, Manifest, Name, Package, Workspace},
    message, rustflags, TRYEXPAND_ENV_KEY, TRYEXPAND_ENV_VAL_EXPECT, TRYEXPAND_ENV_VAL_OVERWRITE,
};

/// An extension for files containing `cargo expand` result.
const EXPANDED_RS_SUFFIX: &str = "expanded.rs";

#[derive(Debug)]
pub(crate) struct Project {
    pub dir: PathBuf,
    source_dir: PathBuf,
    /// Used for the inner runs of cargo()
    pub inner_target_dir: PathBuf,
    pub name: String,
    pub features: Option<Vec<String>>,
    workspace: PathBuf,
    behavior: ExpansionBehavior,
}

/// This `Drop` implementation will clean up the temporary crates when expansion is finished.
/// This is to prevent pollution of the filesystem with dormant files.
impl Drop for Project {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_dir_all(&self.dir) {
            eprintln!(
                "Failed to cleanup the directory `{}`: {}",
                self.dir.to_string_lossy(),
                e
            );
        }
    }
}

/// Attempts to expand macros in files that match glob pattern.
///
/// # Refresh behavior
///
/// If no matching `.expanded.rs` files present, they will be created and result of expansion
/// will be written into them.
///
/// # Panics
///
/// Will panic if matching `.expanded.rs` file is present, but has different expanded code in it.
pub fn expand(path: impl AsRef<Path>) {
    run_tests(path, Option::<Vec<String>>::None, Expectation::Success);
}

/// Attempts to expand macros in files that match glob pattern and expects the expansion to fail.
///
/// # Refresh behavior
///
/// If no matching `.expanded.rs` files present, they will be created and result (error) of expansion
/// will be written into them.
///
/// # Panics
///
/// Will panic if matching `.expanded.rs` file is present, but has different expanded code in it.
pub fn expand_fail(path: impl AsRef<Path>) {
    run_tests(path, Option::<Vec<String>>::None, Expectation::Failure);
}

/// Same as [`expand`] but allows to pass additional arguments to `cargo-expand`.
///
/// [`expand`]: expand/fn.expand.html
pub fn expand_args<I, S>(path: impl AsRef<Path>, args: I)
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    run_tests(path, Some(args), Expectation::Success);
}

/// Same as [`expand_fail`] but allows to pass additional arguments to `cargo-expand`.
///
/// [`expand_fail`]: expand/fn.expand_fail.html
pub fn expand_args_fail<I, S>(path: impl AsRef<Path>, args: I)
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    run_tests(path, Some(args), Expectation::Failure);
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum ExpansionBehavior {
    OverwriteFiles,
    ExpectFiles,
}

fn run_tests<I, S>(path: impl AsRef<Path>, args: Option<I>, expectation: Expectation)
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    match try_run_tests(path, args, expectation) {
        Ok(()) => {}
        Err(err) => panic!("{}", err),
    }
}

fn try_run_tests<I, S>(
    path: impl AsRef<Path>,
    args: Option<I>,
    expectation: Expectation,
) -> Result<()>
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    let tests = expand_globs(&path)?
        .into_iter()
        .filter(|t| !t.path.to_string_lossy().ends_with(EXPANDED_RS_SUFFIX))
        .collect::<Vec<_>>();

    let len = tests.len();
    println!("Running {} macro expansion tests", len);

    let project = prepare(&tests).unwrap_or_else(|err| {
        panic!("prepare failed: {:#?}", err);
    });

    let mut failures = vec![];

    for test in tests {
        let path = test.path.as_path();
        let expanded_path = test.expanded_path.as_path();

        let outcome = match test.run(&project, &args, expectation) {
            Ok(outcome) => outcome,
            Err(err) => {
                eprintln!("Error: {err:#?}");
                failures.push(path.to_owned());
                continue;
            }
        };

        let is_success = match outcome {
            TestOutcome::Ok => {
                message::ok(path, expanded_path);
                true
            }
            TestOutcome::SnapshotMismatch { actual, expected } => {
                message::snapshot_mismatch(path, expanded_path, &expected, &actual);
                false
            }
            TestOutcome::SnapshotCreated { after } => {
                message::snapshot_created(path, expanded_path, &after);
                true
            }
            TestOutcome::SnapshotUpdated { before, after } => {
                message::snapshot_updated(path, expanded_path, &before, &after);
                true
            }
            TestOutcome::SnapshotMissing => {
                message::snapshot_missing(path, expanded_path);
                false
            }
            TestOutcome::UnexpectedSuccess { output } => {
                message::unexpected_success(path, expanded_path, &output);
                false
            }
            TestOutcome::UnexpectedFailure { output } => {
                message::unexpected_failure(path, expanded_path, &output);
                false
            }
        };

        if !is_success {
            failures.push(path.to_owned());
        }
    }

    if !failures.is_empty() {
        let mut message = String::new();

        writeln!(&mut message).unwrap();
        writeln!(&mut message, "{} of {} tests failed:", failures.len(), len).unwrap();
        writeln!(&mut message).unwrap();

        for failure in failures {
            writeln!(&mut message, "- {}", failure.display()).unwrap();
        }

        panic!("{}", message);
    }

    Ok(())
}

fn expansion_behavior() -> Result<ExpansionBehavior> {
    let Some(var) = std::env::var_os(TRYEXPAND_ENV_KEY) else {
        return Ok(ExpansionBehavior::ExpectFiles);
    };

    match var.as_os_str().to_string_lossy().as_ref() {
        TRYEXPAND_ENV_VAL_EXPECT => Ok(ExpansionBehavior::ExpectFiles),
        TRYEXPAND_ENV_VAL_OVERWRITE => Ok(ExpansionBehavior::OverwriteFiles),
        _ => Err(Error::UnrecognizedEnv(var)),
    }
}

fn prepare(tests: &[ExpandedTest]) -> Result<Project> {
    let metadata = cargo::metadata()?;
    let target_dir = metadata.target_directory;
    let workspace = metadata.workspace_root;

    let crate_name = env::var("CARGO_PKG_NAME").map_err(|_| Error::PkgName)?;

    let source_dir = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok_or(Error::ManifestDirError)?;

    let features = features::find();
    let behavior = expansion_behavior()?;

    static COUNT: AtomicUsize = AtomicUsize::new(0);
    // Use unique string for the crate dir to
    // prevent conflicts when running parallel tests.
    let unique_string: String = format!("tryexpand{:03}", COUNT.fetch_add(1, Ordering::SeqCst));
    let dir = path!(target_dir / "tests" / crate_name / unique_string);
    if dir.exists() {
        // Remove remaining artifacts from previous runs if exist.
        // For example, if the user stops the test with Ctrl-C during a previous
        // run, the destructor of Project will not be called.
        fs::remove_dir_all(&dir)?;
    }

    let inner_target_dir = path!(target_dir / "tests" / "tryexpand");

    let mut project = Project {
        dir,
        source_dir,
        inner_target_dir,
        name: format!("{}-tests", crate_name),
        features,
        workspace,
        behavior,
    };

    let manifest = make_manifest(crate_name, &project, tests)?;
    let manifest_toml = basic_toml::to_string(&manifest)?;

    let config = make_config();
    let config_toml = basic_toml::to_string(&config)?;

    if let Some(enabled_features) = &mut project.features {
        enabled_features.retain(|feature| manifest.features.contains_key(feature));
    }

    fs::create_dir_all(path!(project.dir / ".cargo"))?;
    fs::write(path!(project.dir / ".cargo" / "config"), config_toml)?;
    fs::write(path!(project.dir / "Cargo.toml"), manifest_toml)?;
    fs::write(path!(project.dir / "main.rs"), b"fn main() {}\n")?;

    fs::create_dir_all(&project.inner_target_dir)?;

    cargo::build_dependencies(&project)?;

    Ok(project)
}

fn make_manifest(
    crate_name: String,
    project: &Project,
    tests: &[ExpandedTest],
) -> Result<Manifest> {
    let source_manifest = dependencies::get_manifest(&project.source_dir);
    let workspace_manifest = dependencies::get_workspace_manifest(&project.workspace);

    let features = source_manifest
        .features
        .keys()
        .map(|feature| {
            let enable = format!("{}/{}", crate_name, feature);
            (feature.clone(), vec![enable])
        })
        .collect();

    let mut manifest = Manifest {
        package: Package {
            name: project.name.clone(),
            version: "0.0.0".to_owned(),
            edition: source_manifest.package.edition,
            publish: false,
        },
        features,
        dependencies: std::collections::BTreeMap::new(),
        bins: Vec::new(),
        workspace: Some(Workspace {}),
        // Within a workspace, only the [patch] and [replace] sections in
        // the workspace root's Cargo.toml are applied by Cargo.
        patch: workspace_manifest.patch,
        replace: workspace_manifest.replace,
    };

    manifest.dependencies.extend(source_manifest.dependencies);
    manifest
        .dependencies
        .extend(source_manifest.dev_dependencies);
    manifest.dependencies.insert(
        crate_name,
        Dependency {
            version: None,
            path: Some(project.source_dir.clone()),
            default_features: false,
            features: Vec::new(),
            rest: std::collections::BTreeMap::new(),
        },
    );

    manifest.bins.push(Bin {
        name: Name(project.name.to_owned()),
        path: Path::new("main.rs").to_owned(),
    });

    for expanded in tests {
        if expanded.error.is_none() {
            manifest.bins.push(Bin {
                name: expanded.name.clone(),
                path: project.source_dir.join(&expanded.path),
            });
        }
    }

    Ok(manifest)
}

fn make_config() -> Config {
    Config {
        build: Build {
            rustflags: rustflags::make_vec(),
        },
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum Expectation {
    Success,
    Failure,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum ComparisonOutcome {
    Match,
    Mismatch,
}

#[derive(Clone, Debug)]
pub(crate) enum TestOutcome {
    Ok,
    SnapshotMismatch { actual: String, expected: String },
    SnapshotCreated { after: String },
    SnapshotUpdated { before: String, after: String },
    SnapshotMissing,
    UnexpectedSuccess { output: String },
    UnexpectedFailure { output: String },
}

#[derive(Debug)]
struct ExpandedTest {
    name: Name,
    path: PathBuf,
    expanded_path: PathBuf,
    error: Option<Error>,
}

impl ExpandedTest {
    pub fn run<I, S>(
        &self,
        project: &Project,
        args: &Option<I>,
        expectation: Expectation,
    ) -> Result<TestOutcome>
    where
        I: IntoIterator<Item = S> + Clone,
        S: AsRef<OsStr>,
    {
        let expanded_path = self.expanded_path.as_path();
        let expansion = cargo::expand(project, &self.name, args)?;

        let actual = match &expansion {
            cargo::Expansion::Success { stdout } => {
                normalize_expansion(stdout.as_str(), CARGO_EXPAND_SKIP_LINES_COUNT, project)
            }
            cargo::Expansion::Failure { stderr } => normalize_expansion(
                stderr.as_str(),
                CARGO_EXPAND_ERROR_SKIP_LINES_COUNT,
                project,
            ),
        };

        if let (Expansion::Failure { .. }, Expectation::Success) = (&expansion, expectation) {
            return Ok(TestOutcome::UnexpectedFailure { output: actual });
        }

        if let (Expansion::Success { .. }, Expectation::Failure) = (&expansion, expectation) {
            return Ok(TestOutcome::UnexpectedSuccess { output: actual });
        }

        if !expanded_path.exists() {
            match project.behavior {
                ExpansionBehavior::OverwriteFiles => {
                    // Write a .expanded.rs file contents with an newline character at the end
                    // panic!("create: {expanded_path:?}");
                    fs::write(expanded_path, &actual)?;

                    return Ok(TestOutcome::SnapshotCreated { after: actual });
                }
                ExpansionBehavior::ExpectFiles => return Ok(TestOutcome::SnapshotMissing),
            }
        }

        let expected = String::from_utf8_lossy(&fs::read(expanded_path)?).into_owned();

        match compare(&actual, &expected) {
            ComparisonOutcome::Match => Ok(TestOutcome::Ok),
            ComparisonOutcome::Mismatch => {
                match project.behavior {
                    ExpansionBehavior::OverwriteFiles => {
                        // Write a .expanded.rs file contents with an newline character at the end
                        // panic!("update: {expanded_path:?}");
                        fs::write(expanded_path, &actual)?;

                        Ok(TestOutcome::SnapshotUpdated {
                            before: expected.clone(),
                            after: actual,
                        })
                    }
                    ExpansionBehavior::ExpectFiles => {
                        Ok(TestOutcome::SnapshotMismatch { expected, actual })
                    }
                }
            }
        }
    }
}

fn compare(actual: &str, expected: &str) -> ComparisonOutcome {
    if actual.lines().eq(expected.lines()) {
        ComparisonOutcome::Match
    } else {
        ComparisonOutcome::Mismatch
    }
}

// `cargo expand` does always produce some fixed amount of lines that should be ignored
const CARGO_EXPAND_SKIP_LINES_COUNT: usize = 5;
const CARGO_EXPAND_ERROR_SKIP_LINES_COUNT: usize = 1;

/// Removes specified number of lines and removes some unnecessary or non-determenistic cargo output
fn normalize_expansion(input: &str, num_lines_to_skip: usize, project: &Project) -> String {
    // These prefixes are non-deterministic and project-dependent
    // These prefixes or the whole line shall be removed
    let project_path_prefix = format!(" --> {}/", project.source_dir.to_string_lossy());
    let proj_name_prefix = format!("    Checking {} v0.0.0", project.name);
    let blocking_prefix = "    Blocking waiting for file lock on package cache";

    let lines = input
        .lines()
        .skip(num_lines_to_skip)
        .filter(|line| !line.starts_with(&proj_name_prefix))
        .map(|line| line.strip_prefix(&project_path_prefix).unwrap_or(line))
        .map(|line| line.strip_prefix(blocking_prefix).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");

    let mut syntax_tree = match syn::parse_file(&lines) {
        Ok(syntax_tree) => syntax_tree,
        Err(_) => return lines,
    };

    // Strip the following:
    //
    //     #![feature(prelude_import)]
    //
    syntax_tree.attrs.retain(|attr| {
        if let Meta::List(meta) = &attr.meta {
            if meta.path.is_ident("feature") {
                if let Ok(list) =
                    meta.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                {
                    if list.len() == 1 {
                        if let Meta::Path(inner) = &list.first().unwrap() {
                            if inner.is_ident("prelude_import") {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    });

    // Strip the following:
    //
    //     #[prelude_import]
    //     use std::prelude::$edition::*;
    //
    //     #[macro_use]
    //     extern crate std;
    //
    syntax_tree.items.retain(|item| {
        if let Item::Use(item) = item {
            if let Some(attr) = item.attrs.first() {
                if attr.path().is_ident("prelude_import") && attr.meta.require_path_only().is_ok() {
                    return false;
                }
            }
        }
        if let Item::ExternCrate(item) = item {
            if item.ident == "std" {
                return false;
            }
        }
        true
    });

    let lines = prettyplease::unparse(&syntax_tree);
    if cfg!(windows) {
        format!("{}\n\r", lines.trim_end_matches("\n\r"))
    } else {
        format!("{}\n", lines.trim_end_matches('\n'))
    }
}

fn expand_globs(path: impl AsRef<Path>) -> Result<Vec<ExpandedTest>> {
    let path = path.as_ref();

    fn glob(pattern: &str) -> Result<Vec<PathBuf>> {
        let mut paths = glob::glob(pattern)?
            .map(|entry| entry.map_err(Error::from))
            .collect::<Result<Vec<PathBuf>>>()?;
        paths.sort();
        Ok(paths)
    }

    fn bin_name(i: usize) -> Name {
        Name(format!("tryexpand{:03}", i))
    }

    let mut vec = Vec::new();

    let name = path
        .file_stem()
        .expect("no file stem")
        .to_string_lossy()
        .to_string();

    let path_string = path.as_os_str().to_string_lossy();

    let paths = glob(&path_string)?;
    for path in paths {
        let expanded_path = path.with_extension(EXPANDED_RS_SUFFIX);
        vec.push(ExpandedTest {
            name: bin_name(vec.len()),
            path,
            expanded_path,
            error: None,
        });
    }
    // if path_string.contains('*') {
    //     let paths = glob(&path_string)?;
    //     for path in paths {
    //         let expanded_path = path.with_extension(EXPANDED_RS_SUFFIX);
    //         vec.push(ExpandedTest {
    //             name: bin_name(vec.len()),
    //             path,
    //             expanded_path,
    //             error: None,
    //         });
    //     }
    // } else {
    //     let mut expanded = ExpandedTest {
    //         name: Name(name),
    //         path: path.to_path_buf(),
    //         expanded_path: path.with_extension(EXPANDED_RS_SUFFIX),
    //         error: None,
    //     };
    //     vec.push(expanded);
    // }

    Ok(vec)
}
