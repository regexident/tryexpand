use std::{collections::BTreeMap as Map, ffi::OsStr, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    dependencies::{self, Dependency, Patch, RegistryPatch},
    error::Result,
    project::Project,
};

#[derive(Serialize, Debug)]
pub struct Manifest {
    pub package: Package,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub features: Map<String, Vec<String>>,
    pub dependencies: Map<String, Dependency>,
    #[serde(rename = "bin")]
    pub bins: Vec<Bin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<Workspace>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub patch: Map<String, RegistryPatch>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub replace: Map<String, Patch>,
}

impl Manifest {
    pub(crate) fn try_new(
        crate_name: &str,
        test_crate_name: &str,
        project: &Project,
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
            path: PathBuf::from("main.rs"),
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
}

#[derive(Serialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub edition: Edition,
    pub publish: bool,
}

// Do not use enum for edition for future-compatibility.
#[derive(Serialize, Deserialize, Debug)]
pub struct Edition(pub String);

#[derive(Serialize, Debug)]
pub struct Bin {
    pub name: Name,
    pub path: PathBuf,
}

#[derive(Serialize, Clone, Debug)]
pub struct Name(pub String);

#[derive(Serialize, Debug)]
pub struct Config {
    pub build: Build,
}

#[derive(Serialize, Debug)]
pub struct Build {
    pub rustflags: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct Workspace {}

impl Default for Edition {
    fn default() -> Self {
        Self("2021".into())
    }
}

impl AsRef<OsStr> for Name {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}
