use std::{
    fs,
    path::{Path, PathBuf},
};

use cargo_metadata::{Metadata, Package};

use crate::{
    cargo::{self},
    error::{Error, Result},
    manifest,
    test::Test,
    utils,
};

#[derive(Debug)]
pub(crate) struct Project {
    pub name: String,
    pub dir: PathBuf,
    pub manifest_dir: PathBuf,
    pub target_dir: PathBuf,
}

impl Project {
    pub(crate) fn new<'a, I>(
        metadata: &Metadata,
        package: &Package,
        test_suite_id: &str,
        target_dir: &Path,
        tests: I,
    ) -> Result<Project>
    where
        I: IntoIterator<Item = &'a Test>,
    {
        let manifest_path = package.manifest_path.as_std_path().to_owned();
        let manifest_dir = manifest_path.parent().unwrap().to_owned();

        let tests_dir = target_dir.join("tests");

        let crate_name = &package.name;
        let test_crate_name = format!("{crate_name}_{test_suite_id}");
        let dir = tests_dir.join(&test_crate_name);

        let target_dir = tests_dir.join("tryexpand");

        let name = test_crate_name.clone();

        let project = Project {
            dir,
            manifest_dir,
            target_dir,
            name,
        };

        let manifest =
            manifest::cargo_manifest(metadata, package, &test_crate_name, &project, tests)?;
        let manifest_toml =
            basic_toml::to_string(&manifest).map_err(Error::CargoManifestSerializationFailed)?;

        let config = cargo::make_config();
        let config_toml =
            basic_toml::to_string(&config).map_err(Error::CargoConfigSerializationFailed)?;

        if project.dir.exists() {
            // Remove remaining artifacts from previous runs if exist.
            // For example, if the user stops the test with Ctrl-C during a previous
            // run, the destructor of Project will not be called.
            utils::remove_dir_all(&project.dir)?;
        }

        utils::create_dir_all(project.dir.join(".cargo"))?;

        utils::write(project.dir.join(".cargo").join("config.toml"), config_toml)?;
        utils::write(project.dir.join("Cargo.toml"), manifest_toml)?;
        utils::write(project.dir.join("lib.rs"), b"\n")?;

        utils::create_dir_all(&project.target_dir)?;

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
