mod dependency;
mod index;
mod version;

use anyhow::anyhow;
pub use dependency::Dependency;
use pubgrub::version::Version;
use serde::Deserialize;
use std::str::FromStr;
use ustr::Ustr;

use crate::manifest::FieldManifest;

pub use self::version::SemanticVersion;

/// A single line in the index representing a single version of a field.
#[derive(Deserialize)]
pub struct IndexedFieldVersion {
    name: Ustr,
    version: SemanticVersion,
    ontology_iri: Ustr,
    cksum: Ustr,
    dependencies: Vec<IndexedDependencySpec>,
    // TODO:  If `true`, Plow will skip this version when resolving.
    // yanked: bool,
}

impl IndexedFieldVersion {
    pub fn full_name(&self) -> &str {
        &self.name
    }
    pub fn version(&self) -> &SemanticVersion {
        &self.version
    }
    pub fn version_string(&self) -> String {
        self.version.to_string()
    }
    pub fn ontology_iri(&self) -> &str {
        &self.ontology_iri
    }
    pub fn cksum(&self) -> &str {
        &self.cksum
    }
    pub fn dependencies(&self) -> &[IndexedDependencySpec] {
        &self.dependencies
    }
    // pub fn yanked(&self) -> bool {
    //     self.yanked
    // }
    pub fn namespace_and_name(&self) -> (&str, &str) {
        self.name.split_once('/').unwrap()
    }
}

impl From<&FieldManifest<'_>> for IndexedFieldVersion {
    fn from(manifest: &FieldManifest) -> Self {
        Self {
            name: manifest.full_name(),
            version: *manifest.version(),
            ontology_iri: manifest.ontology_iri().into(),
            cksum: manifest.cksum().into(),
            dependencies: manifest
                .dependencies()
                .iter()
                .map(|dep| IndexedDependencySpec {
                    name: dep.full_name().into(),
                    req: dep.version_requirement().into(),
                })
                .collect(),
            // TODO: Yanking..
            // yanked: false,
        }
    }
}

/// A dependency as encoded in the index JSON.
#[derive(Deserialize)]
pub struct IndexedDependencySpec {
    name: Ustr,
    req: Ustr,
}

impl IndexedDependencySpec {
    pub fn full_name(&self) -> &str {
        &self.name
    }
    pub fn req(&self) -> &str {
        &self.req
    }
    pub fn namespace_and_name(&self) -> (&str, &str) {
        self.name.split_once('/').unwrap()
    }
}

impl FromStr for IndexedDependencySpec {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (full_name, req) = input.split_once(' ').ok_or_else(|| {
            anyhow!(
                "Input does not contain enough information to produce an `IndexedDependencySpec`."
            )
        })?;

        Dependency::<SemanticVersion>::split_and_validate_full_field_name(full_name)?;
        Dependency::<SemanticVersion>::split_and_validate_semantic_version_requirement_literal(
            req,
        )?;

        Ok(Self {
            name: full_name.into(),
            req: req.into(),
        })
    }
}

impl ToString for IndexedDependencySpec {
    fn to_string(&self) -> String {
        format!("{} {}", self.name, self.req)
    }
}

impl<V> From<Dependency<V>> for IndexedDependencySpec
where
    V: Version + From<SemanticVersion>,
{
    fn from(dep: Dependency<V>) -> Self {
        Self {
            name: dep.full_name().into(),
            req: dep.version_requirement().into(),
        }
    }
}

// TODO: On the other hand it is not that simple
// pub trait Registry {
//     // @attention Like our get all available versions of a package
//     /// Attempt to find the packages that match a dependency request.
//     fn query(
//         &mut self,
//         dep: &Dependency,
//         kind: QueryKind,
//         f: &mut dyn FnMut(Summary),
//     ) -> Poll<CargoResult<()>>;

//     // @attention Probably this will always return a vec I mean queries
//     fn query_vec(&mut self, dep: &Dependency, kind: QueryKind) -> Poll<CargoResult<Vec<Summary>>> {
//         let mut ret = Vec::new();
//         self.query(dep, kind, &mut |s| ret.push(s)).map_ok(|()| ret)
//     }

//     fn describe_source(&self, source: SourceId) -> String;
//     fn is_replaced(&self, source: SourceId) -> bool;

//     // @attention this stands for our retrieve package
//     /// Block until all outstanding Poll::Pending requests are Poll::Ready.
//     fn block_until_ready(&mut self) -> CargoResult<()>;

//     //@attention our submit package is not needed imo

//     //@attention TODO: Get package version metadata
// }

// A registry fulfills all the duties of an Index (= providing lightweight metadata about packages,
// like e.g. version, dependencies) and an Artifact store (= retrieval of packages).
// pub trait Registry {
//     // TODO: Write docs to trait methods
//     fn all_available_versions_of_a_package(
//         &self,
//         package_namespace_and_name: String,
//     ) -> Vec<PackageVersionWithRegistryMetadata>;
//     fn get_package_version_metadata(
//         &self,
//         package_version: &PackageVersion,
//     ) -> Result<PackageVersionWithRegistryMetadata, anyhow::Error>;

//     fn retrieve_package(&self, package: &PackageVersion) -> Result<Vec<u8>, anyhow::Error>;

//     fn submit_package(
//         &self,
//         file_contents: &str,
//     ) -> Result<PackageVersionWithRegistryMetadata, anyhow::Error>;
// }
