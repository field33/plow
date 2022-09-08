use std::collections::HashSet;
use std::fs::create_dir_all;

use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use plow_linter::lints::field_manifest_lints;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use sha2::{Digest, Sha256};

use crate::config::PlowConfig;
use crate::error::CliError;
use crate::error::WorkspaceInitializationError::*;
use crate::manifest::FieldManifest;
use crate::subcommand::lint::lint_file_fail_on_failure;

// For comparisons later.
fn hash_file_with_name(path: &Utf8Path) -> Option<String> {
    if let Ok(mut file) = std::fs::File::open(path) {
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher).ok()?;
        if let Some(name) = path.file_name() {
            hasher.update(name.as_bytes());
        }
        let result = hasher.finalize();
        return Some(format!("{:x}", result));
    }
    None
}

#[derive(Default, Clone, Eq, PartialOrd, Ord)]
pub struct FieldPath {
    inner: Utf8PathBuf,
}
impl FieldPath {
    #[allow(clippy::missing_const_for_fn)]
    pub fn into_inner(self) -> Utf8PathBuf {
        self.inner
    }
    pub fn as_path(&self) -> &Utf8Path {
        self.inner.as_path()
    }
    pub fn as_path_buf(&self) -> Utf8PathBuf {
        self.inner.clone()
    }
    pub fn update_path(&mut self, path: Utf8PathBuf) {
        self.inner = path;
    }
}

impl std::fmt::Display for FieldPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::fmt::Debug for FieldPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.inner)
    }
}

impl From<Utf8PathBuf> for FieldPath {
    fn from(path: Utf8PathBuf) -> Self {
        Self { inner: path }
    }
}

impl From<&Utf8PathBuf> for FieldPath {
    fn from(path: &Utf8PathBuf) -> Self {
        Self {
            inner: path.clone(),
        }
    }
}

impl From<&Utf8Path> for FieldPath {
    fn from(path: &Utf8Path) -> Self {
        Self {
            inner: path.to_path_buf(),
        }
    }
}

impl PartialEq for FieldPath {
    fn eq(&self, other: &Self) -> bool {
        if let Ok(names_are_same) = crate::utils::file_names_are_same(&self.inner, &other.inner) {
            if let Ok(files_are_same) = crate::utils::files_are_same(&self.inner, &other.inner) {
                return names_are_same && files_are_same;
            }
            return false;
        }
        false
    }
}

pub struct FieldsDirectory {
    pub children: Vec<FieldPath>,
    pub path: Utf8PathBuf,
}

impl FieldsDirectory {
    pub fn create_empty_at(root: &Utf8Path) -> Self {
        Self {
            children: vec![],
            path: root.join("fields").to_path_buf(),
        }
    }

    // This is not performant but we're focusing to the functionality first now.
    pub fn dedup(&mut self) {
        self.children.sort();
        let mut container = HashSet::new();
        let mut all: Vec<FieldPath> = vec![];
        for child in &self.children {
            if let Some(hash) = hash_file_with_name(child.as_path()) {
                if container.contains(&hash) {
                    continue;
                }
                container.insert(hash);
                all.push(child.clone());
            }
        }
        self.children = all;
    }

    // This will never fail if used right.
    // Currently only use with backed up path.
    // We can restrict it with the type system later.
    #[allow(clippy::unwrap_in_result)]
    #[allow(clippy::unwrap_used)]
    pub fn fill_from_backup(root: &Utf8Path) -> Result<Self, CliError> {
        let mut fields_dir = Self::create_empty_at(root.parent().unwrap());
        fields_dir.children = crate::utils::list_files(root, "ttl")
            .map_err(|err| FailedRecursiveListingFields {
                reason: err.to_string(),
            })?
            .iter()
            .map(std::convert::Into::into)
            .collect();
        Ok(fields_dir)
    }

    pub fn fill_from_root(root: &Utf8Path) -> Result<Self, CliError> {
        let mut fields_dir = Self::create_empty_at(root);
        fields_dir.children = crate::utils::list_files(root, "ttl")
            .map_err(|err| FailedRecursiveListingFields {
                reason: err.to_string(),
            })?
            .iter()
            .map(std::convert::Into::into)
            .collect();
        Ok(fields_dir)
    }

    #[allow(clippy::unwrap_used)]
    pub fn extend_from_root_excluding_fields_dir(
        &mut self,
        root: &Utf8Path,
    ) -> Result<(), CliError> {
        self.children = crate::utils::list_files(root, "ttl")
            .map_err(|err| FailedRecursiveListingFields {
                reason: err.to_string(),
            })?
            .iter()
            .filter_map(|path| {
                if !path
                    .canonicalize_utf8()
                    .unwrap()
                    .components()
                    .contains(&camino::Utf8Component::Normal("fields"))
                {
                    return Some(path.clone());
                }
                None
            })
            .map(std::convert::Into::into)
            .collect();
        Ok(())
    }

    pub fn exists_in_filesystem(&self) -> bool {
        self.path.exists()
    }

    pub fn lint_all_children(&self) -> Option<(Vec<String>, CliError)> {
        // Lint all fields in the directory and collect failures if there are some.
        let failed_field_paths_on_linting = self
            .children
            .par_iter()
            .filter_map(|child| {
                // TODO: Field manifest or all lints?
                if let Err(err) =
                    lint_file_fail_on_failure(child.as_path().as_ref(), field_manifest_lints())
                {
                    Some(err)
                } else {
                    None
                }
            })
            .filter_map(|err| match err {
                CliError::LintSubcommand(
                    crate::error::LintSubcommandError::FailedToParseField { field_path }
                    | crate::error::LintSubcommandError::SingleLintContainsFailure { field_path }
                    | crate::error::LintSubcommandError::FailedToReadField { field_path, .. },
                ) => Some(field_path),
                _ => None,
            })
            .collect::<Vec<_>>();

        // If there are failures, prepare the error to inform the user later.
        if !failed_field_paths_on_linting.is_empty() {
            return Some((
                failed_field_paths_on_linting.clone(),
                crate::error::LintSubcommandError::LintsContainFailures {
                    field_paths: failed_field_paths_on_linting,
                }
                .into(),
            ));
        }
        None
    }

    #[allow(clippy::unwrap_in_result)]
    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::unwrap_used)]
    pub fn write_with_children(&mut self) -> Result<(), CliError> {
        std::fs::create_dir_all(&self.path)
            .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;

        let mut copy_number = 1;
        for child in &mut self.children {
            // We're safe here, we've linted before.
            if let Ok(full_name) = FieldManifest::quick_extract_field_full_name(&child.as_path()) {
                let full_name: Vec<&str> = full_name.split('/').collect();
                let namespace = full_name[0];
                let name = full_name[1];

                std::fs::create_dir_all(self.path.join(&namespace).join(&name))
                    .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;

                let new_file_name = child.as_path().file_name().unwrap();
                let mut new_field_destination = self
                    .path
                    .join(&namespace)
                    .join(&name)
                    // Checked before
                    .join(new_file_name);

                // If there are same named files just suffix them as copies.
                // Let the user sort it out later.
                if new_field_destination.exists() {
                    new_field_destination = new_field_destination
                        .with_file_name(format!("{new_file_name}.copy_{copy_number}"));
                    copy_number += 1;
                }

                std::fs::copy(child.as_path(), &new_field_destination)
                    .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;

                child.update_path(new_field_destination);
            }
        }
        Ok(())
    }

    // Explodes the fields directory back to the workspace root.
    pub fn explode(path_to_fields_dir: &Utf8Path, config: &PlowConfig) -> Result<(), CliError> {
        if path_to_fields_dir.exists() {
            let fields = crate::utils::list_files(path_to_fields_dir, "ttl")
                .map_err(|err| FailedToReadFieldsDirectory(err.to_string()))?;
            for field in fields {
                std::fs::copy(
                    &field,
                    &config
                        .working_dir
                        .path
                        .join(field.file_name().ok_or_else(|| {
                            FailedToCreateFieldsDirectory("Found field is not a file.".to_owned())
                        })?),
                )
                .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;
            }
            std::fs::remove_dir_all(&path_to_fields_dir)
                .map_err(|err| FailedToRemoveFieldsDirectory(err.to_string()))?;
            return Ok(());
        }
        Err(FailedToReadFieldsDirectory("Fields directory does not exist".to_owned()).into())
    }

    // Returns the path to the tmp dir
    pub fn backup_if_already_exists(
        path_to_fields_dir: &Utf8Path,
        config: &PlowConfig,
    ) -> Result<Option<Utf8PathBuf>, CliError> {
        if path_to_fields_dir.exists() {
            let fields = crate::utils::list_files(path_to_fields_dir, "ttl")
                .map_err(|err| FailedToReadFieldsDirectory(err.to_string()))?;

            let backup_dir = config
                .working_dir
                .path
                .join(format!(".plow_{}", uuid::Uuid::new_v4()));

            create_dir_all(&backup_dir)
                .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;

            for field in fields {
                std::fs::copy(
                    &field,
                    &backup_dir.join(field.file_name().ok_or_else(|| {
                        FailedToCreateFieldsDirectory("Found field is not a file.".to_owned())
                    })?),
                )
                .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;
            }
            std::fs::remove_dir_all(&path_to_fields_dir)
                .map_err(|err| FailedToRemoveFieldsDirectory(err.to_string()))?;

            return Ok(Some(backup_dir));
        }
        Ok(None)
    }

    pub fn remove(&self) -> Result<(), CliError> {
        Ok(std::fs::remove_dir_all(&self.path)
            .map_err(|err| FailedToRemoveFieldsDirectory(err.to_string()))?)
    }

    pub fn write_empty(&self) -> Result<(), CliError> {
        Ok(std::fs::create_dir_all(&self.path)
            .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?)
    }
}
