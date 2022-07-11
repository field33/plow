use crate::{
    package::PackageVersionWithRegistryMetadata, resolve::Dependency, version::SemanticVersion,
};
use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexContents {
    pub versions: Vec<IndexedPackageVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedPackageVersion {
    pub name: String,
    pub version: String,
    /// Sha256 hash of the main artifact
    pub cksum: String,
    pub ontology_iri: Option<String>,
    pub deps: Vec<IndexedPackageDependency>,
}

impl TryFrom<IndexedPackageVersion> for PackageVersionWithRegistryMetadata {
    type Error = Error;
    fn try_from(indexed_package_version: IndexedPackageVersion) -> Result<Self, Self::Error> {
        indexed_package_version
            .deps
            .into_iter()
            .map(std::convert::TryInto::try_into)
            .collect::<Result<Vec<Dependency<SemanticVersion>>, _>>()
            .map_or_else(Err, |dependencies| {
                Ok(Self {
                    package_name: indexed_package_version.name,
                    version: indexed_package_version.version.try_into()?,
                    ontology_iri: indexed_package_version.ontology_iri,
                    dependencies,
                    cksum: Some(indexed_package_version.cksum),
                })
            })
    }
}

impl TryFrom<PackageVersionWithRegistryMetadata> for IndexedPackageVersion {
    type Error = Error;

    fn try_from(original: PackageVersionWithRegistryMetadata) -> Result<Self, Self::Error> {
        let package_name = original.package_name.clone();
        Ok(Self {
            cksum: original.cksum.ok_or_else(|| {
                anyhow!(
                    "No cksum provided for package `{name}`",
                    name = &package_name
                )
            })?,
            name: original.package_name,
            version: original.version.to_string(),
            ontology_iri: original.ontology_iri,
            deps: original.dependencies.into_iter().map(Into::into).collect(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedPackageDependency {
    pub name: String,
    pub req: String,
}

impl From<Dependency<SemanticVersion>> for IndexedPackageDependency {
    fn from(dependency: Dependency<SemanticVersion>) -> Self {
        Self {
            name: dependency.full_name,
            req: dependency.version_requirement,
        }
    }
}
