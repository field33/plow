use crate::{
    package::{PackageVersion, PackageVersionWithRegistryMetadata},
    registry::Registry,
};

use anyhow::{anyhow, Error};
use std::{clone::Clone, collections::HashMap};

/// An in-memory version of a registry that allows for explicit definition of packages and their relations.
#[derive(Debug, Default)]
pub struct InMemoryRegistry {
    /// Map of package versions to ontology file contents.
    packages: HashMap<PackageVersion, String>,
    /// Map of packages to their metadata (= index).
    packages_metadata: HashMap<PackageVersion, PackageVersionWithRegistryMetadata>,
}

impl InMemoryRegistry {
    pub fn insert(
        &mut self,
        package_version: PackageVersion,
        metadata: PackageVersionWithRegistryMetadata,
        ontology_file_content: String,
    ) {
        self.packages
            .insert(package_version.clone(), ontology_file_content);
        self.packages_metadata.insert(package_version, metadata);
    }
}

impl Registry for InMemoryRegistry {
    fn all_available_versions_of_a_package(
        &self,
        package_namespace_and_name: String,
    ) -> Vec<PackageVersionWithRegistryMetadata> {
        self.packages_metadata
            .iter()
            .filter(|(package, _)| package.package_name == package_namespace_and_name)
            .map(|(_, metadata)| metadata.clone())
            .collect()
    }
    fn get_package_version_metadata(
        &self,
        package_version: &PackageVersion,
    ) -> Result<PackageVersionWithRegistryMetadata, Error> {
        self.packages_metadata
            .get(package_version)
            .ok_or_else(|| {
                anyhow!(
                    "Unable to retrieve package registry metadata for package {:?}",
                    &package_version
                )
            })
            .map(Clone::clone)
    }

    fn retrieve_package(&self, package: &PackageVersion) -> Result<Vec<u8>, Error> {
        self.packages.get(package).map(|contents| contents.clone().into_bytes()).ok_or_else(|| anyhow!("Dependency for the following version not found in registry: {dependency_spec:?}", dependency_spec = package))
    }

    fn submit_package(
        &self,
        _file_contents: &str,
    ) -> Result<PackageVersionWithRegistryMetadata, Error> {
        unimplemented!()
    }
}
