use std::collections::BTreeMap;

use cargo_metadata::{
    Dependency as SourceDependency, DependencyKind as SourceDependencyKind,
    Edition as SourceEdition, Metadata as SourceMetadata, Package as SourcePackage,
};
use cargo_toml::{
    Badges, Dependency, DependencyDetail, DepsSet, Edition, Inheritable, Manifest, Package,
    PatchSet, Product, Profiles, TargetDepsSet, Workspace,
};

use crate::{
    error::{Error, Result},
    project::Project,
    test::Test,
};

pub(crate) fn cargo_manifest<'a, I>(
    _source_metadata: &SourceMetadata,
    source_package: &SourcePackage,
    test_crate_name: &str,
    project: &Project,
    tests: I,
) -> Result<Manifest>
where
    I: IntoIterator<Item = &'a Test>,
{
    #![allow(clippy::field_reassign_with_default)]

    let mut package: Package = Package::new(test_crate_name.to_owned(), "0.0.0".to_owned());

    package.edition = match source_package.edition {
        SourceEdition::E2015 => Inheritable::Set(Edition::E2015),
        SourceEdition::E2018 => Inheritable::Set(Edition::E2018),
        SourceEdition::E2021 => Inheritable::Set(Edition::E2021),
        edition => {
            return Err(Error::UnsupportedRustEdition {
                edition: edition.to_string(),
            })
        }
    };

    package.rust_version = source_package
        .rust_version
        .as_ref()
        .map(|version| Inheritable::Set(version.to_string()));

    let mut dependencies = BTreeMap::default();
    let mut dev_dependencies = BTreeMap::default();
    let mut build_dependencies = BTreeMap::default();

    let dependency_path = project.manifest_dir.display().to_string();
    let mut dependency = DependencyDetail::default();
    dependency.path = Some(dependency_path);
    dependency.default_features = true;

    let dependency_name = source_package.name.clone();

    dependencies.insert(
        dependency_name.to_owned(),
        Dependency::Detailed(Box::new(dependency)),
    );

    for source_dependency in &source_package.dependencies {
        let SourceDependency {
            name,
            source: _,
            req,
            kind,
            optional,
            uses_default_features,
            features,
            target: _,
            rename,
            registry,
            path,
            ..
        } = source_dependency;

        let mut dependency = DependencyDetail::default();
        dependency.package = rename.clone();
        dependency.version = Some(req.to_string());
        dependency.optional = *optional;
        dependency.default_features = *uses_default_features;
        dependency.features = features.clone();
        dependency.registry = registry.clone();
        dependency.path = path.as_ref().map(|path| path.to_string());
        dependency.default_features = true;

        match kind {
            SourceDependencyKind::Normal => {
                dependencies.insert(name.to_owned(), Dependency::Detailed(Box::new(dependency)));
            }
            SourceDependencyKind::Development => {
                dev_dependencies
                    .insert(name.to_owned(), Dependency::Detailed(Box::new(dependency)));
            }
            SourceDependencyKind::Build => {
                build_dependencies
                    .insert(name.to_owned(), Dependency::Detailed(Box::new(dependency)));
            }
            _ => todo!(),
        }
    }

    let features = source_package
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

    let bin: Vec<_> = tests
        .into_iter()
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
        dev_dependencies,
        build_dependencies,
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

    Ok(manifest)
}
