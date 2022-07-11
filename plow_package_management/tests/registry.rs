mod common;

use crate::common::tests_filepath;
use plow_package_management::package::PackageVersion;
use plow_package_management::registry::on_disk::OnDiskRegistry;
use plow_package_management::registry::Registry;
use tempdir::TempDir;

#[test]
fn verify_integrity() {
    let registry_path = tests_filepath("../../data/example_registries/simple_hierarchy_with_ns");
    let registry = OnDiskRegistry::new(registry_path);
    assert!(registry.is_ok());
    assert!(registry.unwrap().verify_integrity().is_ok());
}

#[test]
fn simple_hierarchy_registry() -> Result<(), anyhow::Error> {
    let registry_path = tests_filepath("../../data/example_registries/simple_hierarchy_with_ns");
    let registry = OnDiskRegistry::new(registry_path)?;

    let package_version = PackageVersion {
        package_name: "@namespace/top_level".to_string(),
        version: "0.0.1".to_string(),
    };
    let package_metadata = registry.get_package_version_metadata(&package_version);
    assert!(package_metadata.is_ok());

    let package_artifact = registry.retrieve_package(&package_version);
    assert!(package_artifact.is_ok());

    Ok(())
}

#[test]
fn submit_top_level() -> Result<(), anyhow::Error> {
    let temp_dir = TempDir::new("submit_top_level")?;
    let registry = OnDiskRegistry::new(temp_dir.path())?;

    let ontology_package = std::fs::read_to_string(tests_filepath(
        "../../data/example_registries/simple_hierarchy_with_ns/artifacts/topLevel_0_0_1",
    ))
    .unwrap();

    registry
        .submit_package(&ontology_package)
        .expect("Should successfully submit package");

    let package_version = PackageVersion {
        package_name: "@namespace/top_level".to_string(),
        version: "0.0.1".to_string(),
    };
    let package_metadata = registry.get_package_version_metadata(&package_version);
    assert!(package_metadata.is_ok());

    let package_artifact = registry.retrieve_package(&package_version);
    assert!(package_artifact.is_ok());

    Ok(())
}
