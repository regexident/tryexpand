use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    path::PathBuf,
};

use crate::{
    cargo::{self},
    error::{Error, Result},
    features,
    manifest::Manifest,
    test::Test,
};

/// An extension for files containing `cargo expand` result.
const EXPANDED_RS_SUFFIX: &str = "expanded.rs";

#[derive(Debug)]
pub(crate) struct Project {
    pub(crate) dir: PathBuf,
    pub(crate) source_dir: PathBuf,
    /// Used for the inner runs of cargo()
    pub(crate) inner_target_dir: PathBuf,
    pub(crate) name: String,
    pub(crate) features: Option<Vec<String>>,
    pub(crate) workspace: PathBuf,
    pub(crate) tests: Vec<Test>,
}

pub(crate) fn setup_project<I>(crate_name: &str, test_suite_id: &str, paths: I) -> Result<Project>
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

    let test_crate_name = format!("{crate_name}-{test_suite_id}");
    let dir = tests_dir.join(&test_crate_name);

    let inner_target_dir = tests_dir.join("tryexpand");

    let name = test_crate_name.clone();

    let mut tests: Vec<_> = paths
        .into_iter()
        .map(|path| {
            let bin = {
                let mut hasher = DefaultHasher::default();
                path.hash(&mut hasher);
                // Taking the lower-case of a base62 hash leads to collisions
                // but the number of tests we're dealing with shouldn't
                // realistically cause any issues in practice:
                let test_id = base62::encode(hasher.finish()).to_lowercase();
                format!("{name}-{test_id}")
            };
            let expanded_path = path.with_extension(EXPANDED_RS_SUFFIX);
            Test {
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

    let manifest = Manifest::try_new(crate_name, &test_crate_name, &project)?;
    let manifest_toml = basic_toml::to_string(&manifest)?;

    let config = cargo::make_config();
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

pub(crate) fn teardown_project(project_dir: PathBuf) -> Result<()> {
    if project_dir.exists() {
        // Remove artifacts from the run (on a best-effort basis):
        fs::remove_dir_all(project_dir)?;
    }

    Ok(())
}
