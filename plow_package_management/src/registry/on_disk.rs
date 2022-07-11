use crate::{
    metadata::OntologyMetadata,
    package::{PackageVersion, PackageVersionWithRegistryMetadata},
    registry::Registry,
};

use anyhow::{anyhow, bail, Context, Error};
use glob::glob;
use harriet::TurtleDocument;
use plow_ontology::PackageName;

use sha2::{Digest, Sha256};
use std::{
    convert::{TryFrom, TryInto},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use super::index::{IndexContents, IndexedPackageVersion};

const INDEX_DIRECTORY: &str = "index";
const ARTIFACTS_DIRECTORY: &str = "artifacts";

/// An on-disk version of the registry based on the following file layout:
///
/// -- <`REGISTRY_ROOT`> --- index --- @namespace1 -- foo.json
///                     |         |
///                     |         |- @namespace2 -- bar.json
///                     |
///                     |- artifacts -- <`ARTIFACT_HASH`>
#[derive(Debug, Clone)]
pub struct OnDiskRegistry {
    root_directory: PathBuf,
}

impl OnDiskRegistry {
    pub fn new<R: Into<PathBuf>>(root_directory: R) -> Result<Self, Error> {
        let root_directory: PathBuf = root_directory.into();

        if !root_directory.exists() {
            bail!(
                "Provided root directory {:?} for registry does not exist.",
                &root_directory
            );
        }

        Ok(Self { root_directory })
    }

    pub fn root_directory(&self) -> &Path {
        &self.root_directory
    }

    fn index_dir(&self) -> Result<PathBuf, Error> {
        let index_path = self.root_directory.join(INDEX_DIRECTORY);
        Ok(PathBuf::from(index_path.to_str().ok_or_else(|| {
            anyhow!("Unable to turn registry index path into string.")
        })?))
    }

    /// Iterates through all packages in the registry and checks that they are well-formed.
    ///
    /// This currently checks whether the metadata for all packages can be parsed and their artifacts
    /// are available under the listed cksum.
    ///
    /// Currently DOES NOT validate the cksum of the artifacts.
    pub fn verify_integrity(&self) -> Result<(), Error> {
        let index_dir = self.index_dir()?;

        let glob_pattern = format!("{index}/**/*.json", index = index_dir.to_string_lossy());
        for metadata_file in glob(&glob_pattern)? {
            let metadata_file = metadata_file?;

            let file_contents = fs::read_to_string(metadata_file)?;
            let package_contents: IndexContents = serde_json::from_str(&file_contents)?;

            for version in package_contents.versions {
                self.get_artifact_for_cksum(&version.cksum)?;
            }
        }
        Ok(())
    }

    fn package_metadata_path(&self, package_name: &str) -> Result<PathBuf, Error> {
        let package_name = PackageName::try_from(package_name.to_owned())?;
        let namespace = package_name.namespace();
        let name = package_name.package();
        let expected_package_path = self
            .root_directory
            .join(INDEX_DIRECTORY)
            .join(namespace)
            .join(format!("{name}.json", name = name));

        Ok(expected_package_path)
    }

    /// Tries to retrieve the the package metadata from the index.
    ///
    /// If no version of the package exists in the index it will return `None`.
    fn get_package_index_metadata(
        &self,
        package_name: &str,
    ) -> Result<Option<IndexContents>, Error> {
        let expected_package_path = self.package_metadata_path(package_name)?;

        if !expected_package_path.exists() {
            return Ok(None);
        }

        let file_contents = fs::read_to_string(expected_package_path)?;
        Ok(Some(serde_json::from_str(&file_contents)?))
    }

    /// Tries to retrieve the the package version metadata from the index.
    ///
    /// If the version of the package does not in the index it will return `None`.
    fn get_package_version_metadata_index(
        &self,
        package_version: &PackageVersion,
    ) -> Result<Option<IndexedPackageVersion>, Error> {
        let package_metadata = self.get_package_index_metadata(&package_version.package_name)?;

        let index_package_version = match package_metadata {
            None => None,
            Some(package_metadata) => Some(
                package_metadata
                    .versions
                    .into_iter()
                    .find(|index_package| index_package.version == package_version.version),
            ),
        }
        .flatten();

        Ok(index_package_version)
    }

    fn get_artifact_for_cksum(&self, cksum: &str) -> Result<Vec<u8>, Error> {
        let expected_package_path = self.root_directory.join(ARTIFACTS_DIRECTORY).join(cksum);

        if !expected_package_path.exists() {
            bail!(
                "No artifact found in registry for cksum `{cksum}`",
                cksum = cksum
            );
        }

        Ok(fs::read(expected_package_path)?)
    }

    /// Computes the SHA256 cksum for the provided file contents.
    fn compute_cksum(file_contents: &str) -> String {
        let mut sha256 = Sha256::new();
        sha256.update(file_contents.as_bytes());
        format!("{:X}", sha256.finalize()).to_lowercase()
    }

    /// Writes the provided ontology package to the artifact registry and returns its cksum.
    fn submit_package_artifact(&self, file_contents: &str) -> Result<String, Error> {
        let cksum = Self::compute_cksum(file_contents);

        let artifact_path = self.root_directory.join(ARTIFACTS_DIRECTORY).join(&cksum);

        if let Some(artifact_parent_path) = artifact_path.parent() {
            fs::create_dir_all(artifact_parent_path)?;
            let mut file = fs::File::create(artifact_path)?;
            file.write_all(file_contents.as_bytes())?;

            return Ok(cksum);
        }
        Err(anyhow!(
            "The parent directory for the artifact_path could not be retrieved."
        ))
    }

    /// Adds the package version metadata to the package metadata file (and creates it if necessary).
    fn submit_package_version_metadata(
        &self,
        package_version: &PackageVersionWithRegistryMetadata,
    ) -> Result<(), Error> {
        let package_metadata_path = self.package_metadata_path(&package_version.package_name)?;
        // Create metadata file if it doesn't exist
        if !package_metadata_path.exists() {
            if let Some(package_metadata_parent_path) = package_metadata_path.parent() {
                fs::create_dir_all(package_metadata_parent_path)?;
                let mut file = fs::File::create(&package_metadata_path)?;

                let initial_metadata = IndexContents { versions: vec![] };

                file.write_all(serde_json::to_string_pretty(&initial_metadata)?.as_bytes())?;
            } else {
                return Err(anyhow!(
                    "The parent directory for the package_metadata_path could not be retrieved."
                ));
            }
        }

        // Add new version to metadata
        let mut metadata: IndexContents =
            serde_json::from_str(&fs::read_to_string(&package_metadata_path)?)?;
        metadata.versions.push(package_version.clone().try_into()?);

        let mut file = fs::File::create(&package_metadata_path)?;
        file.write_all(serde_json::to_string_pretty(&metadata)?.as_bytes())?;

        Ok(())
    }

    fn package_version_exists(&self, package_version: &PackageVersion) -> Result<bool, Error> {
        Ok(self
            .get_package_version_metadata_index(package_version)?
            .is_some())
    }

    pub fn list_all_package_versions(
        &self,
    ) -> Result<Vec<PackageVersionWithRegistryMetadata>, Error> {
        let index_dir = self.index_dir()?;

        let mut package_versions = vec![];
        let glob_pattern = format!("{index}/**/*.json", index = index_dir.to_string_lossy());

        for metadata_file in glob(&glob_pattern)? {
            let metadata_file = metadata_file?;

            let file_contents = fs::read_to_string(metadata_file)?;

            let package_contents: IndexContents = serde_json::from_str(&file_contents)?;

            for version in package_contents.versions {
                package_versions.push(version.try_into()?);
            }
        }

        Ok(package_versions)
    }
}

impl Registry for OnDiskRegistry {
    fn all_available_versions_of_a_package(
        &self,
        package_namespace_and_name: String,
    ) -> Vec<PackageVersionWithRegistryMetadata> {
        self.list_all_package_versions()
            .unwrap_or_default()
            .into_iter()
            .filter(|package_version| package_version.package_name == package_namespace_and_name)
            .collect()
    }
    fn get_package_version_metadata(
        &self,
        package_version: &PackageVersion,
    ) -> Result<PackageVersionWithRegistryMetadata, Error> {
        self.get_package_version_metadata_index(package_version)
            .map(|n| n.ok_or_else(|| anyhow!("Unable to find package version metadata")))?
            .map(std::convert::TryInto::try_into)?
    }

    fn retrieve_package(&self, package: &PackageVersion) -> Result<Vec<u8>, Error> {
        let package_version_metadata_opt = self.get_package_version_metadata_index(package)?;
        let package_version_metadata = package_version_metadata_opt.ok_or_else(|| {
            anyhow!(
                "Unable to get metadata for package version `{:?}`",
                &package
            )
        })?;

        self.get_artifact_for_cksum(&package_version_metadata.cksum)
            .context(anyhow!(
                "Dependency for the following version not found in registry: {dependency_spec:?}",
                dependency_spec = package
            ))
    }

    /// Submits an ontology package to the registry.
    fn submit_package(
        &self,
        file_contents: &str,
    ) -> Result<PackageVersionWithRegistryMetadata, Error> {
        if let Ok(document) = TurtleDocument::parse_full(file_contents) {
            if let Ok(metadata) = OntologyMetadata::try_from(&document) {
                // TODO: run registry-required lints before submitting
                if self.package_version_exists(&metadata.clone().into())? {
                    return Err(anyhow!(
                        "Version `{version}` of package `{name}` is already present in registry.",
                        version = metadata.package_version,
                        name = metadata.package_name,
                    ));
                }
                let cksum = self.submit_package_artifact(file_contents)?;

                let new_package_version = PackageVersionWithRegistryMetadata {
                    package_name: metadata.package_name,
                    version: metadata.package_version,
                    ontology_iri: Some(metadata.root_prefix),
                    dependencies: metadata.dependencies,
                    cksum: Some(cksum),
                };

                self.submit_package_version_metadata(&new_package_version)?;

                return Ok(new_package_version);
            }
            return Err(anyhow!(
                "Couldn't get ontology metadata from turtle document."
            ));
        }
        Err(anyhow!("Couldn't parse turtle document."))
    }
}
