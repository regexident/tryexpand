use std::{
    env,
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use syn::{punctuated::Punctuated, Item, Meta, Token};

use crate::{
    cargo,
    dependencies::{self, Dependency},
    error::{Error, Result},
    features,
    manifest::{Bin, Build, Config, Manifest, Name, Package, Workspace},
    message::{message_different, message_expansion_error},
    rustflags,
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
    overwrite: bool,
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
    run_tests(
        path,
        ExpansionBehavior::RegenerateFiles,
        Option::<Vec<String>>::None,
        false,
    );
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
    run_tests(
        path,
        ExpansionBehavior::RegenerateFiles,
        Option::<Vec<String>>::None,
        true,
    );
}

/// Same as [`expand`] but allows to pass additional arguments to `cargo-expand`.
///
/// [`expand`]: expand/fn.expand.html
pub fn expand_args<I, S>(path: impl AsRef<Path>, args: I)
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    run_tests(path, ExpansionBehavior::RegenerateFiles, Some(args), false);
}

/// Same as [`expand_fail`] but allows to pass additional arguments to `cargo-expand`.
///
/// [`expand_fail`]: expand/fn.expand_fail.html
pub fn expand_args_fail<I, S>(path: impl AsRef<Path>, args: I)
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    run_tests(path, ExpansionBehavior::RegenerateFiles, Some(args), true);
}

/// Attempts to expand macros in files that match glob pattern.
/// More strict version of [`expand`] function.
///
/// # Refresh behavior
///
/// If no matching `.expanded.rs` files present, it's considered a failed test.
///
/// # Panics
///
/// Will panic if no matching `.expanded.rs` file is present. Otherwise it will exhibit the same
/// behavior as in [`expand`].
///
/// [`expand`]: expand/fn.expand.html
pub fn expand_without_refresh(path: impl AsRef<Path>) {
    run_tests(
        path,
        ExpansionBehavior::ExpectFiles,
        Option::<Vec<String>>::None,
        false,
    );
}

/// Attempts to expand macros in files that match glob pattern and expects a failure.
/// More strict version of [`expand_fail`] function.
///
/// # Refresh behavior
///
/// If no matching `.expanded.rs` files present, it's considered a failed test.
///
/// # Panics
///
/// Will panic if no matching `.expanded.rs` file is present. Otherwise it will exhibit the same
/// behavior as in [`expand_fail`].
///
/// [`expand_fail`]: expand/fn.expand_fail.html
pub fn expand_without_refresh_fail(path: impl AsRef<Path>) {
    run_tests(
        path,
        ExpansionBehavior::ExpectFiles,
        Option::<Vec<String>>::None,
        true,
    );
}

/// Same as [`expand_without_refresh`] but allows to pass additional arguments to `cargo-expand`.
///
/// [`expand_without_refresh`]: expand/fn.expand_without_refresh.html
pub fn expand_without_refresh_args<I, S>(path: impl AsRef<Path>, args: I)
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    run_tests(path, ExpansionBehavior::ExpectFiles, Some(args), false);
}

/// Same as [`expand_without_refresh_fail`] but allows to pass additional arguments to `cargo-expand` and expects a failure.
///
/// [`expand_without_refresh_fail`]: expand/fn.expand_without_refresh_fail.html
pub fn expand_without_refresh_args_fail<I, S>(path: impl AsRef<Path>, args: I)
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    run_tests(path, ExpansionBehavior::ExpectFiles, Some(args), true);
}

#[derive(Debug, Copy, Clone)]
enum ExpansionBehavior {
    RegenerateFiles,
    ExpectFiles,
}

fn run_tests<I, S>(
    path: impl AsRef<Path>,
    expansion_behavior: ExpansionBehavior,
    args: Option<I>,
    expect_fail: bool,
) where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    let tests = expand_globs(&path)
        .into_iter()
        .filter(|t| !t.test.to_string_lossy().ends_with(EXPANDED_RS_SUFFIX))
        .collect::<Vec<_>>();

    let len = tests.len();
    println!("Running {} macro expansion tests", len);

    let project = prepare(&tests).unwrap_or_else(|err| {
        panic!("prepare failed: {:#?}", err);
    });

    let mut failures = 0;
    for test in tests {
        let path = test.test.display();
        let expanded_path = test.test.with_extension(EXPANDED_RS_SUFFIX);

        match test.run(&project, expansion_behavior, &args) {
            Ok(ExpansionOutcome { error, outcome }) => {
                if let Some(msg) = error {
                    if !expect_fail {
                        message_expansion_error(msg);
                        failures += 1;

                        continue;
                    }
                }

                match outcome {
                    ExpansionOutcomeKind::Same => {
                        let _ = writeln!(std::io::stdout(), "{} - ok", path);
                    }

                    ExpansionOutcomeKind::Different(a, b) => {
                        message_different(&path.to_string(), &a, &b);
                        failures += 1;
                    }

                    ExpansionOutcomeKind::Update(_) => {
                        let _ =
                            writeln!(std::io::stderr(), "{} - refreshed", expanded_path.display());
                    }

                    ExpansionOutcomeKind::NoExpandedFileFound => {
                        let _ = writeln!(
                            std::io::stderr(),
                            "{} is expected but not found",
                            expanded_path.display()
                        );
                        failures += 1;
                    }
                }
            }

            Err(e) => {
                eprintln!("Error: {:#?}", e);
                failures += 1;
            }
        }
    }

    if failures > 0 {
        eprintln!("\n\n");
        panic!("{} of {} tests failed", failures, len);
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

    let overwrite = match env::var_os("tryexpand") {
        Some(ref v) if v == "overwrite" => true,
        Some(v) => return Err(Error::UnrecognizedEnv(v)),
        None => false,
    };

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
        overwrite,
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
                path: project.source_dir.join(&expanded.test),
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

#[derive(Debug)]
struct ExpansionOutcome {
    error: Option<Vec<u8>>,
    outcome: ExpansionOutcomeKind,
}

impl ExpansionOutcome {
    pub fn new(error: Option<Vec<u8>>, outcome: ExpansionOutcomeKind) -> Self {
        Self { error, outcome }
    }
}

#[derive(Debug)]
enum ExpansionOutcomeKind {
    Same,
    Different(Vec<u8>, Vec<u8>),
    Update(Vec<u8>),
    NoExpandedFileFound,
}

struct ExpandedTest {
    name: Name,
    test: PathBuf,
    error: Option<Error>,
}

impl ExpandedTest {
    pub fn run<I, S>(
        &self,
        project: &Project,
        expansion_behavior: ExpansionBehavior,
        args: &Option<I>,
    ) -> Result<ExpansionOutcome>
    where
        I: IntoIterator<Item = S> + Clone,
        S: AsRef<OsStr>,
    {
        let (success, output_bytes) = cargo::expand(project, &self.name, args)?;

        let error = if success {
            None
        } else {
            Some(output_bytes.clone())
        };

        let file_stem = self
            .test
            .file_stem()
            .expect("no file stem")
            .to_string_lossy()
            .into_owned();
        let mut expanded = self.test.clone();
        expanded.pop();
        let expanded = &expanded.join(format!("{}.{}", file_stem, EXPANDED_RS_SUFFIX));

        let output = if success {
            normalize_expansion(&output_bytes, CARGO_EXPAND_SKIP_LINES_COUNT, project)
        } else {
            normalize_expansion(&output_bytes, CARGO_EXPAND_ERROR_SKIP_LINES_COUNT, project)
        };

        if !expanded.exists() {
            if let ExpansionBehavior::ExpectFiles = expansion_behavior {
                return Ok(ExpansionOutcome::new(
                    error,
                    ExpansionOutcomeKind::NoExpandedFileFound,
                ));
            }

            // Write a .expanded.rs file contents with an newline character at the end
            std::fs::write(expanded, output)?;

            return Ok(ExpansionOutcome::new(
                error,
                ExpansionOutcomeKind::Update(output_bytes),
            ));
        }

        let expected_expansion_bytes = std::fs::read(expanded)?;
        let expected_expansion = String::from_utf8_lossy(&expected_expansion_bytes);

        let same = output
            .trim_end_matches(['\n', '\r'])
            .lines()
            .eq(expected_expansion.trim_end_matches(['\n', '\r']).lines());

        if !same && project.overwrite {
            if let ExpansionBehavior::ExpectFiles = expansion_behavior {
                return Ok(ExpansionOutcome::new(
                    error,
                    ExpansionOutcomeKind::NoExpandedFileFound,
                ));
            }

            // Write a .expanded.rs file contents with an newline character at the end
            std::fs::write(expanded, output)?;

            return Ok(ExpansionOutcome::new(
                error,
                ExpansionOutcomeKind::Update(output_bytes),
            ));
        }

        Ok(if same {
            ExpansionOutcome::new(error, ExpansionOutcomeKind::Same)
        } else {
            let output_bytes = output.into_bytes(); // Use normalized text for a message
            ExpansionOutcome::new(
                error,
                ExpansionOutcomeKind::Different(expected_expansion_bytes, output_bytes),
            )
        })
    }
}

// `cargo expand` does always produce some fixed amount of lines that should be ignored
const CARGO_EXPAND_SKIP_LINES_COUNT: usize = 5;
const CARGO_EXPAND_ERROR_SKIP_LINES_COUNT: usize = 1;

/// Removes specified number of lines and removes some unnecessary or non-determenistic cargo output
fn normalize_expansion(input: &[u8], num_lines_to_skip: usize, project: &Project) -> String {
    // These prefixes are non-deterministic and project-dependent
    // These prefixes or the whole line shall be removed
    let project_path_prefix = format!(" --> {}/", project.source_dir.to_string_lossy());
    let proj_name_prefix = format!("    Checking {} v0.0.0", project.name);
    let blocking_prefix = "    Blocking waiting for file lock on package cache";

    let code = String::from_utf8_lossy(input);
    let lines = code
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

    if !lines.ends_with("\n\n") {
        format!("{}\n", lines)
    } else {
        lines
    }
}

fn expand_globs(path: impl AsRef<Path>) -> Vec<ExpandedTest> {
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
        .as_ref()
        .file_stem()
        .expect("no file stem")
        .to_string_lossy()
        .to_string();
    let mut expanded = ExpandedTest {
        name: Name(name),
        test: path.as_ref().to_path_buf(),
        error: None,
    };

    if let Some(utf8) = path.as_ref().to_str() {
        if utf8.contains('*') {
            match glob(utf8) {
                Ok(paths) => {
                    for path in paths {
                        vec.push(ExpandedTest {
                            name: bin_name(vec.len()),
                            test: path,
                            error: None,
                        });
                    }
                }
                Err(error) => expanded.error = Some(error),
            }
        } else {
            vec.push(expanded);
        }
    }

    vec
}
