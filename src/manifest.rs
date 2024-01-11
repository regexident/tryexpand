use std::{collections::BTreeMap, path::PathBuf};

use cargo_toml::{
    Badges, Dependency, DependencyDetail, DepsSet, Manifest, Package, PatchSet, Product, Profiles,
    TargetDepsSet, Workspace,
};

use crate::{error::Result, project::Project};

pub(crate) fn cargo_manifest(
    _workspace_manifest: Option<&Manifest>,
    package_manifest: &Manifest,
    test_crate_name: &str,
    project: &Project,
) -> Result<Manifest> {
    #![allow(clippy::field_reassign_with_default)]

    let source_package = package_manifest.package();

    let mut package: Package = Package::new(test_crate_name.to_owned(), "0.0.0".to_owned());
    package.edition = source_package.edition;
    package.rust_version = source_package.rust_version.clone();

    let mut dependencies = BTreeMap::default();

    let dependency_path = project.manifest_dir.display().to_string();
    let mut dependency = DependencyDetail::default();
    dependency.path = Some(dependency_path);
    dependency.default_features = true;

    let dependency_name = source_package.name();

    dependencies.insert(
        dependency_name.to_owned(),
        Dependency::Detailed(Box::new(dependency)),
    );

    let features = package_manifest
        .features
        .keys()
        .map(|feature| {
            (
                feature.clone(),
                vec![format!("{dependency_name}/{feature}")],
            )
        })
        .collect();

    let mut lib_product = Product::default();
    lib_product.name = Some(test_crate_name.replace('-', "_").to_owned());
    lib_product.path = Some("lib.rs".to_owned());

    let bin: Vec<_> = project
        .tests
        .iter()
        .map(|test| {
            let mut bin_product = Product::default();
            let test_path = std::env::current_dir().unwrap().join(&test.path);
            bin_product.name = Some(test.bin.to_owned());
            bin_product.path = Some(test_path.display().to_string());
            bin_product
        })
        .collect();

    let workspace = Workspace {
        members: vec![],
        default_members: vec![],
        package: None,
        exclude: vec![],
        metadata: None,
        resolver: None,
        dependencies: BTreeMap::default(),
    };

    #[allow(deprecated)]
    let manifest = Manifest {
        package: Some(package),
        workspace: Some(workspace),
        dependencies,
        dev_dependencies: DepsSet::default(),
        build_dependencies: DepsSet::default(),
        target: TargetDepsSet::default(),
        features,
        replace: DepsSet::default(),
        patch: PatchSet::default(),
        lib: Some(lib_product),
        profile: Profiles::default(),
        badges: Badges::default(),
        bin,
        bench: vec![],
        test: vec![],
        example: vec![],
    };
    // eprintln!("{:#?}", manifest);

    Ok(manifest)
}

pub(crate) fn workspace_manifest(root_manifest: &Manifest) -> Option<Manifest> {
    if root_manifest.workspace.is_some() {
        Some(root_manifest.clone())
    } else {
        None
    }
}

pub(crate) fn package_manifest(root_manifest: &Manifest, package_name: &str) -> Option<Manifest> {
    if let Some(package) = &root_manifest.package {
        if package.name() == package_name {
            return Some(root_manifest.clone());
        } else {
            return None;
        }
    }

    let Some(workspace) = &root_manifest.workspace else {
        return None;
    };

    workspace
        .members
        .iter()
        .map(PathBuf::from)
        .find_map(|member_path| {
            let file_name = member_path.file_name().map(|n| n.to_string_lossy());
            let manifest_path = if file_name.as_deref() == Some("Cargo.toml") {
                member_path
            } else {
                member_path.join("Cargo.toml")
            };
            Manifest::from_path(manifest_path)
                .ok()
                .filter(|manifest| manifest.package().name() == package_name)
        })
}
