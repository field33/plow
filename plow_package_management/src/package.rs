use crate::resolve::Dependency;
use crate::version::SemanticVersion;
use crate::{metadata::OntologyMetadata, ORGANIZATION_NAME};
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;

/// A single version of a package with enough information to serve as input for the dependency resolution process.
///
/// Needs to be enriched to [`PackageVersionWithRegistryMetadata`] via a registry to fully resolve a dependency tree.
#[derive(Debug, Clone, Eq, Hash, PartialOrd, Ord, PartialEq, Deserialize, Serialize)]
pub struct PackageVersion {
    pub package_name: String,
    pub version: String,
}

impl PackageVersion {
    pub const fn new(package_name: String, version: String) -> Self {
        Self {
            package_name,
            version,
        }
    }
}
impl ToString for PackageVersion {
    fn to_string(&self) -> String {
        format!("{} {}", self.package_name, self.version)
    }
}
// This unusual From implementation is a convenience implementation
// when using it in resolver when working with iterators of maps.
//
// Having this makes the other part of the code more readable.
impl From<(&String, &SemanticVersion)> for PackageVersion {
    fn from(tuple: (&String, &SemanticVersion)) -> Self {
        Self {
            package_name: tuple.0.clone(),
            version: tuple.1.to_string(),
        }
    }
}

impl From<PackageVersionWithRegistryMetadata> for PackageVersion {
    fn from(metadata: PackageVersionWithRegistryMetadata) -> Self {
        Self {
            package_name: metadata.package_name,
            version: metadata.version.to_string(),
        }
    }
}

impl From<&PackageVersionWithRegistryMetadata> for PackageVersion {
    fn from(metadata: &PackageVersionWithRegistryMetadata) -> Self {
        Self {
            package_name: metadata.package_name.clone(),
            version: metadata.version.to_string(),
        }
    }
}

/// A single version of a package with enough information to be used in dependency resolution.
#[derive(Debug, Clone, Serialize)]
pub struct PackageVersionWithRegistryMetadata {
    pub package_name: String,
    pub version: SemanticVersion,
    pub ontology_iri: Option<String>,
    pub dependencies: Vec<Dependency<SemanticVersion>>,
    pub cksum: Option<String>,
    pub private: bool,
}

impl PartialEq for PackageVersionWithRegistryMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.package_name == other.package_name && self.version == other.version
    }
}

impl Eq for PackageVersionWithRegistryMetadata {}

impl std::fmt::Display for PackageVersionWithRegistryMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.package_name, self.version)
    }
}

/// A flat list of package versions (e.g. dependencies).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageSet {
    pub packages: Vec<PackageVersion>,
}

/// A single version of a package that has been retrieved (= downloaded).
#[derive(Debug, Clone)]
pub struct RetrievedPackageVersion {
    pub ontology_iri: String,
    pub package: PackageVersion,
    pub file_path: Utf8PathBuf,
}

#[derive(Debug, Clone)]
pub struct FieldMetadata {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub dependencies: Vec<Dependency<SemanticVersion>>,
}

impl FieldMetadata {
    pub fn new(
        namespace: String,
        name: String,
        version: String,
        dependencies: Vec<Dependency<SemanticVersion>>,
    ) -> Self {
        Self {
            namespace,
            name,
            version,
            dependencies,
        }
    }
}

/// A flat list of package versions (e.g. dependencies) tha has been retrieved (= downloaded).
#[derive(Debug, Clone)]
pub struct RetrievedPackageSet {
    pub packages: Vec<RetrievedPackageVersion>,
}

/// The type which resolver directly expects
#[derive(Debug)]
pub struct OrganizationToResolveFor {
    pub package_name: String,
    pub package_version: SemanticVersion,
    pub dependencies: Vec<Dependency<SemanticVersion>>,
}

impl From<FieldMetadata> for OrganizationToResolveFor {
    fn from(field_metadata: FieldMetadata) -> Self {
        Self {
            package_name: ORGANIZATION_NAME.to_owned(),
            package_version: SemanticVersion::default(),
            dependencies: field_metadata.dependencies,
        }
    }
}

impl From<OntologyMetadata> for OrganizationToResolveFor {
    fn from(ontology_metadata: OntologyMetadata) -> Self {
        Self {
            package_name: ORGANIZATION_NAME.to_owned(),
            package_version: SemanticVersion::default(),
            dependencies: ontology_metadata.dependencies,
        }
    }
}

impl From<PackageVersionWithRegistryMetadata> for OrganizationToResolveFor {
    fn from(metadata: PackageVersionWithRegistryMetadata) -> Self {
        Self {
            package_name: ORGANIZATION_NAME.to_owned(),
            package_version: SemanticVersion::default(),
            dependencies: metadata.dependencies,
        }
    }
}

impl From<Vec<Dependency<SemanticVersion>>> for OrganizationToResolveFor {
    fn from(dependencies: Vec<Dependency<SemanticVersion>>) -> Self {
        Self {
            package_name: ORGANIZATION_NAME.to_owned(),
            package_version: SemanticVersion::default(),
            dependencies,
        }
    }
}
