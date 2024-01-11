use core::panic;
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    env,
    ffi::OsStr,
    fmt::Write,
    fs,
    hash::{Hash, Hasher},
    iter::FromIterator,
    path::{Path, PathBuf},
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
    tests: Vec<ExpandedTest>,
}

macro_rules! run_tests {
    ($paths:expr, $args:expr, $expectation:expr) => {
        // IMPORTANT: This only works as lone as all functions between
        // the public API and this call are marked with `#[track_caller]`:
        let caller_location = ::std::panic::Location::caller();

        use std::hash::{Hash as _, Hasher as _};

        let mut hasher = ::std::collections::hash_map::DefaultHasher::default();
        caller_location.file().hash(&mut hasher);
        caller_location.line().hash(&mut hasher);
        caller_location.column().hash(&mut hasher);
        let test_suite_id = ::base62::encode(hasher.finish());

        match $crate::try_run_tests($paths, $args, &test_suite_id, $expectation) {
            Ok(()) => {}
            Err(err) => panic!("{}", err),
        }
    };
}

pub(crate) use run_tests;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum ExpansionBehavior {
    OverwriteFiles,
    ExpectFiles,
}

#[track_caller] // LOAD-BEARING, DO NOT REMOVE!
pub(crate) fn try_run_tests<Ip, P, Ia, A>(
    paths: Ip,
    args: Option<Ia>,
    test_suite_id: &str,
    expectation: Expectation,
) -> Result<()>
where
    Ip: IntoIterator<Item = P>,
    P: AsRef<Path>,
    Ia: IntoIterator<Item = A> + Clone,
    A: AsRef<OsStr>,
{
    let unique_paths: HashSet<PathBuf> = paths
        .into_iter()
        .filter_map(|path| expand_globs(path).ok())
        .flatten()
        .filter(|path| !path.to_string_lossy().ends_with(EXPANDED_RS_SUFFIX))
        .collect();

    let paths: Vec<PathBuf> = Vec::from_iter(unique_paths);
    let len = paths.len();

    let crate_name = env::var("CARGO_PKG_NAME").map_err(|_| Error::PkgName)?;

    let project = setup_project(&crate_name, test_suite_id, paths).unwrap_or_else(|err| {
        panic!("prepare failed: {:#?}", err);
    });

    let _guard = scopeguard::guard((), |_| {
        let _ = teardown_project(project.dir.clone());
    });

    let behavior = expansion_behavior()?;

    println!("Running {} macro expansion tests ...!", len);

    let mut failures = vec![];

    for test in &project.tests {
        let path = test.path.as_path();
        let expanded_path = test.expanded_path.as_path();

        let outcome = match test.run(&project, &args, behavior, expectation) {
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
            writeln!(&mut message, "    {}", failure.display()).unwrap();
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

fn setup_project<I>(crate_name: &str, test_suite_id: &str, paths: I) -> Result<Project>
where
    I: IntoIterator<Item = PathBuf>,
{
    let metadata = cargo::metadata()?;
    let target_dir = metadata.target_directory;
    let workspace = metadata.workspace_root;

    let source_dir = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok_or(Error::ManifestDir)?;

    let features = features::find();

    let tests_dir = target_dir.join("tests");

    let test_crate_name = make_name(crate_name, test_suite_id);
    let dir = tests_dir.join(&test_crate_name);

    let inner_target_dir = tests_dir.join("tryexpand");

    let name = test_crate_name.clone();

    let mut tests: Vec<_> = paths
        .into_iter()
        .map(|path| {
            let bin = {
                let mut hasher = DefaultHasher::default();
                path.hash(&mut hasher);
                let test_id = base62::encode(hasher.finish());
                make_name(&name, &test_id)
            };
            let expanded_path = path.with_extension(EXPANDED_RS_SUFFIX);
            ExpandedTest {
                bin,
                path,
                expanded_path,
                error: None,
            }
        })
        .collect();

    tests.sort_by_cached_key(|test| test.path.clone());

    let mut project = Project {
        dir,
        source_dir,
        inner_target_dir,
        name,
        features,
        workspace,
        tests,
    };

    let manifest = make_manifest(crate_name, &test_crate_name, &project)?;
    let manifest_toml = basic_toml::to_string(&manifest)?;

    let config = make_config();
    let config_toml = basic_toml::to_string(&config)?;

    if let Some(enabled_features) = &mut project.features {
        enabled_features.retain(|feature| manifest.features.contains_key(feature));
    }

    if project.dir.exists() {
        // Remove remaining artifacts from previous runs if exist.
        // For example, if the user stops the test with Ctrl-C during a previous
        // run, the destructor of Project will not be called.
        fs::remove_dir_all(&project.dir)?;
    }

    fs::create_dir_all(project.dir.join(".cargo"))?;
    fs::write(project.dir.join(".cargo").join("config"), config_toml)?;
    fs::write(project.dir.join("Cargo.toml"), manifest_toml)?;
    fs::write(project.dir.join("main.rs"), b"fn main() {}\n")?;

    fs::create_dir_all(&project.inner_target_dir)?;

    cargo::build_dependencies(&project)?;

    Ok(project)
}

fn teardown_project(project_dir: PathBuf) -> Result<()> {
    if project_dir.exists() {
        // Remove artifacts from the run (on a best-effort basis):
        fs::remove_dir_all(project_dir)?;
    }

    Ok(())
}

fn make_name(name: &str, id: &str) -> String {
    format!("{name}-{id}")
}

fn make_manifest(crate_name: &str, test_crate_name: &str, project: &Project) -> Result<Manifest> {
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
            name: test_crate_name.to_owned(),
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
        crate_name.to_owned(),
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

    for expanded in &project.tests {
        if expanded.error.is_none() {
            manifest.bins.push(Bin {
                name: Name(expanded.bin.clone()),
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
    bin: String,
    path: PathBuf,
    expanded_path: PathBuf,
    error: Option<Error>,
}

impl ExpandedTest {
    pub fn run<I, S>(
        &self,
        project: &Project,
        args: &Option<I>,
        behavior: ExpansionBehavior,
        expectation: Expectation,
    ) -> Result<TestOutcome>
    where
        I: IntoIterator<Item = S> + Clone,
        S: AsRef<OsStr>,
    {
        let expanded_path = self.expanded_path.as_path();

        let expansion = cargo::expand(project, &self.bin, args)?;

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
            match behavior {
                ExpansionBehavior::OverwriteFiles => {
                    // Write a .expanded.rs file contents with an newline character at the end
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
                match behavior {
                    ExpansionBehavior::OverwriteFiles => {
                        // Write a .expanded.rs file contents with an newline character at the end
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

fn expand_globs(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let path = path.as_ref();

    fn glob(pattern: &str) -> Result<Vec<PathBuf>> {
        let mut paths = glob::glob(pattern)?
            .map(|entry| entry.map_err(Error::from))
            .collect::<Result<Vec<PathBuf>>>()?;
        paths.sort();
        Ok(paths)
    }

    let path_string = path.as_os_str().to_string_lossy();

    Ok(glob(&path_string)?.into_iter().collect())
}
