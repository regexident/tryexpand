use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    path::PathBuf,
};

use cargo_toml::Manifest;

use crate::{
    cargo::{self},
    error::{Error, Result},
    manifest,
    test::Test,
};

#[derive(Debug)]
pub(crate) struct Project {
    pub dir: PathBuf,
    pub manifest_dir: PathBuf,
    /// Used for the inner runs of cargo()
    pub target_dir: PathBuf,
    pub name: String,
    pub tests: Vec<Test>,
}

impl Project {
    pub(crate) fn new<I>(crate_name: &str, test_suite_id: &str, paths: I) -> Result<Project>
    where
        I: IntoIterator<Item = PathBuf>,
    {
        let manifest_path = "Cargo.toml";

        let current_dir = std::env::current_dir().expect("current working directory");

        let root_manifest = Manifest::from_path(manifest_path).map_err(Error::CargoMetadata)?;
        let workspace_manifest = manifest::workspace_manifest(&root_manifest);
        // eprintln!("workspace_manifest: {workspace_manifest:#?}");
        let package_manifest =
            manifest::package_manifest(&root_manifest, crate_name).expect("package");
        // eprintln!("package_manifest: {package_manifest:#?}");

        let target_dir = env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| current_dir.join("target"));
        let manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .ok_or(Error::CargoManifestDir)?;

        let tests_dir = target_dir.join("tests");

        let test_crate_name = format!("{crate_name}_{test_suite_id}");
        let dir = tests_dir.join(&test_crate_name);

        let target_dir = tests_dir.join("tryexpand");

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
                    format!("{name}_{test_id}")
                };
                Test { bin, path }
            })
            .collect();

        tests.sort_by_cached_key(|test| test.path.clone());

        let project = Project {
            dir,
            manifest_dir,
            target_dir,
            name,
            tests,
        };

        let manifest = manifest::cargo_manifest(
            workspace_manifest.as_ref(),
            &package_manifest,
            &test_crate_name,
            &project,
        )?;
        let manifest_toml = basic_toml::to_string(&manifest)?;

        let config = cargo::make_config();
        let config_toml = basic_toml::to_string(&config)?;

        if project.dir.exists() {
            // Remove remaining artifacts from previous runs if exist.
            // For example, if the user stops the test with Ctrl-C during a previous
            // run, the destructor of Project will not be called.
            fs::remove_dir_all(&project.dir)?;
        }

        fs::create_dir_all(project.dir.join(".cargo"))?;
        fs::write(project.dir.join(".cargo").join("config"), config_toml)?;
        fs::write(project.dir.join("Cargo.toml"), manifest_toml)?;
        fs::write(project.dir.join("lib.rs"), b"\n")?;

        fs::create_dir_all(&project.target_dir)?;

        cargo::build_dependencies(&project)?;

        Ok(project)
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let should_keep_artifacts = match should_keep_artifacts() {
            Ok(should_keep_artifacts) => should_keep_artifacts,
            Err(err) => {
                eprintln!("warning: {err}");
                false
            }
        };

        if !should_keep_artifacts {
            // Remove artifacts from the run (on a best-effort basis):
            let _ = fs::remove_dir_all(&self.dir);
        }
    }
}

fn should_keep_artifacts() -> Result<bool> {
    let key = crate::TRYEXPAND_KEEP_ARTIFACTS_ENV_KEY;
    let Some(var) = std::env::var_os(key) else {
        return Ok(false);
    };
    let value = var.to_string_lossy().to_lowercase().to_owned();
    match value.as_str() {
        "1" | "yes" | "true" => Ok(true),
        "0" | "no" | "false" => Ok(false),
        _ => Err(Error::UnrecognizedEnv {
            key: key.to_owned(),
            value,
        }),
    }
}
