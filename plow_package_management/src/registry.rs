pub mod in_memory;
pub mod index;
pub mod on_disk;
pub mod on_disk_git;

use crate::package::{PackageVersion, PackageVersionWithRegistryMetadata};

/// A registry fulfills all the duties of an Index (= providing lightweight metadata about packages,
/// like e.g. version, dependencies) and an Artifact store (= retrieval of packages).
pub trait Registry {
    // TODO: Write docs to trait methods
    fn all_available_versions_of_a_package(
        &self,
        package_namespace_and_name: String,
    ) -> Vec<PackageVersionWithRegistryMetadata>;
    fn get_package_version_metadata(
        &self,
        package_version: &PackageVersion,
    ) -> Result<PackageVersionWithRegistryMetadata, anyhow::Error>;

    fn retrieve_package(&self, package: &PackageVersion) -> Result<Vec<u8>, anyhow::Error>;

    fn submit_package(
        &self,
        file_contents: &str,
    ) -> Result<PackageVersionWithRegistryMetadata, anyhow::Error>;
}
