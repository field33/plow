#![allow(clippy::use_self)]

use crate::field::OrganizationToResolveFor;
use anyhow::bail;
use camino::{Utf8Path, Utf8PathBuf};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{
    field::{fieldVersion, Fieldset},
    registry::Registry,
    resolve::Resolver,
};

/// Common name between all lock files for ontologies.
pub const LOCK_FILE_NAME: &str = "Plow.lock";

/// A runtime representation of a lock file.
#[derive(Debug, Clone, Default)]
pub struct LockFile {
    // Maybe necessary later.
    _path: Option<PathBuf>,
    contents: Vec<FieldSummary>,
}

impl LockFile {
    /// Writes the conceptual lock file to the given path.
    ///
    /// This mimics Cargo.lock and produces a similar structure, like below.
    ///
    /// ```toml
    /// [[field]]
    /// name = "@namespace/name"
    /// # A complete bare version.
    /// version = "0.2.15"
    /// # Currently left empty
    /// source = ""
    /// cksum = "24606928a235e73cdef55a0c909719cadd72fce573e5713d58cb2952d8f5794c"
    /// # Set of dependencies with corresponding versions.
    /// dependencies = [
    ///  "@x/y 0.24.0",
    ///  "@a/b 1.15.0",
    /// ]
    /// ```
    pub fn write(
        workspace_root: Option<Utf8PathBuf>,
        field_set: &[FieldInLockFile],
    ) -> Result<Utf8PathBuf, anyhow::Error> {
        let fields = FieldsInLockFile {
            version: FileVersion::V1,
            fields: field_set.to_vec(),
        };

        let serialized = toml::to_string_pretty(&Fields)?;

        if let Some(workspace_root) = workspace_root {
            let lock_file_path = workspace_root.join(LOCK_FILE_NAME);
            let mut lock_file_contents = "# This file is automatically generated by Field33.\n# It is not intended for manual editing.\n\n".to_owned();
            lock_file_contents += &serialized;
            std::fs::write(&lock_file_path, lock_file_contents)?;

            return Ok(lock_file_path);
        }
        bail!("There is no workspace root to write the lock file.");
    }

    pub fn deserialize_lock_file(
        lock_file_path: &Utf8Path,
    ) -> Result<FieldsInLockFile, anyhow::Error> {
        let lock_file_contents = std::fs::read_to_string(lock_file_path)?;
        Ok(toml::from_str::<FieldsInLockFile>(&lock_file_contents)?)
    }

    pub fn previous_lock_file_exists(workspace_root: Option<Utf8PathBuf>) -> Option<Utf8PathBuf> {
        workspace_root.and_then(|workspace_root| {
            let lock_file_path_in_workspace_root = workspace_root.join(LOCK_FILE_NAME);
            if lock_file_path_in_workspace_root.exists() {
                return Some(lock_file_path_in_workspace_root);
            }
            None
        })
    }

    /// Starts locking operation, resolves dependencies and write the lock file.
    pub fn lock_with_registry(
        field_to_resolve: OrganizationToResolveFor,
        registry: &dyn Registry,
        workspace_root: Option<Utf8PathBuf>,
        respect_existing_lock_file: bool,
    ) -> Result<Self, anyhow::Error> {
        // TODO: Either this or another entry point will be expanded to support db based locks in the future.

        let (resolved_dependencies, _previously_locked_path) =
            if let Some(lock_file_path) = Self::previous_lock_file_exists(workspace_root) {
                if respect_existing_lock_file {
                    // With existing lock file input
                    let fields = Self::deserialize_lock_file(&lock_file_path)?
                        .fields
                        .iter()
                        .cloned()
                        // TODO:
                        // Currently filter the local resolutions out.
                        // Will be addressed soon.
                        .filter(|p| !p.root)
                        .collect::<Vec<_>>();

                    (
                        Into::<crate::resolve::VersionRequestResolver>::into(registry)
                            .resolve_dependencies(field_to_resolve, Some(&fields))?,
                        Some(lock_file_path),
                    )
                } else {
                    (
                        Into::<crate::resolve::VersionRequestResolver>::into(registry)
                            .resolve_dependencies(field_to_resolve, None)?,
                        None,
                    )
                }
            } else {
                // No lockfile input
                (
                    Into::<crate::resolve::VersionRequestResolver>::into(registry)
                        .resolve_dependencies(field_to_resolve, None)?,
                    None,
                )
            };

        // TODO: Do not write in this function!
        // let resolved_dependencies_with_metadata: Vec<fieldVersionWithRegistryMetadata> =
        //     resolved_dependencies
        //         .Fields
        //         .iter()
        //         .map(|field_version| registry.get_field_version_metadata(field_version))
        //         .collect::<Result<Vec<_>, _>>()?;

        // if !resolved_dependencies_with_metadata.is_empty() {
        //     // Do not write a lock file if there are no dependencies.
        //     let path = Self::write(selected_file, &resolved_dependencies_with_metadata)?;
        //     return Ok(Self {
        //         _path: Some(path),
        //         locked_dependencies: resolved_dependencies,
        //     });
        // }

        // if let Some(existing_lock_file_path) = previously_locked_path {
        //     if resolved_dependencies_with_metadata.is_empty() {
        //         // If there are no dependencies, but there is an existing lock file, remove it.
        //         std::fs::remove_file(existing_lock_file_path)?;
        //     }
        // }

        // Always re-writing the lock file even if it is the same,
        // I think it is harmless since we're not interested on the creation time.
        Ok(Self {
            _path: None,
            locked_dependencies: resolved_dependencies,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub enum FileVersion {
    #[serde(rename(serialize = "1", deserialize = "1"))]
    V1,
}
/// Set of Fields in the form to be serialized to or to be deserialized to or from the lock file.
#[derive(Serialize, Deserialize)]
#[serde(rename(serialize = "field", deserialize = "field"))]
pub struct FieldsInLockFile {
    version: FileVersion,
    #[serde(rename(serialize = "field", deserialize = "field"))]
    fields: Vec<FieldInLockFile>,
}

impl FieldsInLockFile {
    pub fn fields(&self) -> &[fieldInLockFile] {
        &self.fields
    }
}

/// A field in the form to be serialized to or to be deserialized to or from the lock file.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FieldInLockFile {
    pub root: bool,
    pub name: String,
    pub version: String,
    pub source: String,
    pub ontology_iri: Option<String>,
    pub cksum: String,
    pub dependencies: Vec<String>,
}
